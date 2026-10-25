#!/usr/bin/env python3
"""Nezávislá referenční implementace zabezpečeného kanálu (spec/PROTOCOL.md §14).

Generuje spec/test-vectors/channel.json. Node (Rust) se musí shodovat bajt po bajtu.
Použití: python3 spec/reference/channel.py > spec/test-vectors/channel.json
Potřebuje balík `cryptography`. node_id se bere z identity.json (blake3 tu není potřeba).
"""
import base64, hashlib, json, os, struct, sys

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey, X25519PublicKey
from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
from cryptography.hazmat.primitives.kdf.hkdf import HKDF
from cryptography.hazmat.primitives import hashes, serialization

HERE = os.path.dirname(os.path.abspath(__file__))
VECTORS = os.path.join(HERE, "..", "test-vectors")


def b64u(b: bytes) -> str:
    return base64.urlsafe_b64encode(b).rstrip(b"=").decode()


def unb64u(s: str) -> bytes:
    return base64.urlsafe_b64decode(s + "=" * (-len(s) % 4))


def raw_pub(k) -> bytes:
    return k.public_key().public_bytes(serialization.Encoding.Raw, serialization.PublicFormat.Raw)


def ed_sign(seed: bytes, domain: str, payload: bytes) -> bytes:
    return Ed25519PrivateKey.from_private_bytes(seed).sign(domain.encode() + b"\x00" + payload)


def hkdf64(ikm: bytes, salt: bytes, info: bytes) -> bytes:
    return HKDF(algorithm=hashes.SHA256(), length=64, salt=salt, info=info).derive(ikm)


def pad(data: bytes) -> bytes:
    total = max(256, -(-(4 + len(data)) // 256) * 256)
    return struct.pack(">I", len(data)) + data + b"\x00" * (total - 4 - len(data))


# --- HChaCha20 / XChaCha20-Poly1305 (draft-irtf-cfrg-xchacha) ---
def _rotl(v, n):
    return ((v << n) & 0xFFFFFFFF) | (v >> (32 - n))


def hchacha20(key: bytes, nonce16: bytes) -> bytes:
    s = list(struct.unpack("<4I", b"expand 32-byte k")) + list(struct.unpack("<8I", key)) + list(struct.unpack("<4I", nonce16))

    def qr(a, b, c, d):
        s[a] = (s[a] + s[b]) & 0xFFFFFFFF; s[d] = _rotl(s[d] ^ s[a], 16)
        s[c] = (s[c] + s[d]) & 0xFFFFFFFF; s[b] = _rotl(s[b] ^ s[c], 12)
        s[a] = (s[a] + s[b]) & 0xFFFFFFFF; s[d] = _rotl(s[d] ^ s[a], 8)
        s[c] = (s[c] + s[d]) & 0xFFFFFFFF; s[b] = _rotl(s[b] ^ s[c], 7)

    for _ in range(10):
        qr(0, 4, 8, 12); qr(1, 5, 9, 13); qr(2, 6, 10, 14); qr(3, 7, 11, 15)
        qr(0, 5, 10, 15); qr(1, 6, 11, 12); qr(2, 7, 8, 13); qr(3, 4, 9, 14)
    return struct.pack("<8I", *(s[0:4] + s[12:16]))


def xchacha_seal(key: bytes, nonce24: bytes, aad: bytes, pt: bytes) -> bytes:
    sub = hchacha20(key, nonce24[:16])
    return ChaCha20Poly1305(sub).encrypt(b"\x00" * 4 + nonce24[16:], pt, aad)


def seed_of(n: int) -> bytes:
    return bytes([n]) * 32


def main():
    identity = json.load(open(os.path.join(VECTORS, "identity.json")))
    node_ids = {c["seed"]: c["node_id"] for c in identity["cases"]}
    pubs = {c["seed"]: unb64u(c["pubkey"]) for c in identity["cases"]}

    # ---- klient <-> node (§14.1)
    node_seed = seed_of(1)
    node_id = node_ids[b64u(node_seed)]
    npk = pubs[b64u(node_seed)]
    challenge = hashlib.sha256(b"chan-challenge").digest()
    ce_sk = X25519PrivateKey.from_private_bytes(hashlib.sha256(b"chan-ce").digest())
    ne_sk = X25519PrivateKey.from_private_bytes(hashlib.sha256(b"chan-ne").digest())
    ce, ne = raw_pub(ce_sk), raw_pub(ne_sk)
    th = hashlib.sha256(b"ancord/v1/chan\x00" + challenge + ce + ne + npk + node_id.encode()).digest()
    node_sig = ed_sign(node_seed, "ancord/v1/chan-node", th)
    shared = ce_sk.exchange(X25519PublicKey.from_public_bytes(ne))
    assert shared == ne_sk.exchange(X25519PublicKey.from_public_bytes(ce))
    okm = hkdf64(shared, th, b"ancord/v1/chan/keys")
    k_c2n, k_n2c = okm[:32], okm[32:]
    ek_seed = seed_of(2)
    client_sig = ed_sign(ek_seed, "ancord/v1/chan-client", th)

    def frame(key, counter, text):
        pt = pad(text.encode())
        nonce = b"\x00" * 4 + struct.pack(">Q", counter)
        return {"counter": counter, "plaintext": text, "frame": b64u(ChaCha20Poly1305(key).encrypt(nonce, pt, b"ancord/v1/chan/frame" + th))}

    chan = {
        "node_seed": b64u(node_seed), "node_id": node_id, "npk": b64u(npk),
        "challenge": b64u(challenge),
        "ce_secret": b64u(hashlib.sha256(b"chan-ce").digest()), "ne_secret": b64u(hashlib.sha256(b"chan-ne").digest()),
        "ce": b64u(ce), "ne": b64u(ne),
        "th": b64u(th), "node_sig": b64u(node_sig), "shared": b64u(shared),
        "k_c2n": b64u(k_c2n), "k_n2c": b64u(k_n2c),
        "ek_seed": b64u(ek_seed), "ek": b64u(pubs[b64u(ek_seed)]), "client_sig": b64u(client_sig),
        "c2n_frames": [frame(k_c2n, 0, '{"t":"auth","ticket":"x.y","sig":"z"}'), frame(k_c2n, 1, '{"t":"ping"}')],
        "n2c_frames": [frame(k_n2c, 0, '{"t":"ready"}')],
    }

    # ---- node <-> node (§14.2)
    i_seed, r_seed = seed_of(3), seed_of(1)
    ipk, rpk = pubs[b64u(i_seed)], pubs[b64u(r_seed)]
    req_nonce = hashlib.sha256(b"n2n-request-nonce").digest()[:16]
    ice = X25519PrivateKey.from_private_bytes(hashlib.sha256(b"n2n-ce").digest())
    rne = X25519PrivateKey.from_private_bytes(hashlib.sha256(b"n2n-ne").digest())
    n_ce, n_ne = raw_pub(ice), raw_pub(rne)
    sid = hashlib.sha256(b"n2n-sid").digest()[:16]
    n_th = hashlib.sha256(b"ancord/v1/n2n\x00" + ipk + rpk + req_nonce + n_ce + n_ne + sid).digest()
    n_sig = ed_sign(r_seed, "ancord/v1/n2n-resp", n_th)
    n_shared = ice.exchange(X25519PublicKey.from_public_bytes(n_ne))
    n_okm = hkdf64(n_shared, n_th, b"ancord/v1/n2n/keys")
    k_i2r, k_r2i = n_okm[:32], n_okm[32:]
    rq_nonce = hashlib.sha256(b"n2n-xnonce-req").digest()[:24]
    rs_nonce = hashlib.sha256(b"n2n-xnonce-res").digest()[:24]
    rq_pt = b'{"op":"get","ts":1760000000,"body":{"mailbox_id":"AA","limit":10}}'
    rs_pt = b'{"status":200,"body":{"messages":[]}}'
    n2n = {
        "initiator_seed": b64u(i_seed), "responder_seed": b64u(r_seed),
        "initiator_pk": b64u(ipk), "responder_pk": b64u(rpk),
        "request_nonce": b64u(req_nonce),
        "ce_secret": b64u(hashlib.sha256(b"n2n-ce").digest()), "ne_secret": b64u(hashlib.sha256(b"n2n-ne").digest()),
        "ce": b64u(n_ce), "ne": b64u(n_ne), "sid": b64u(sid),
        "th": b64u(n_th), "resp_sig": b64u(n_sig), "shared": b64u(n_shared),
        "k_i2r": b64u(k_i2r), "k_r2i": b64u(k_r2i),
        "request": {"nonce": b64u(rq_nonce), "plaintext": rq_pt.decode(),
                    "c": b64u(xchacha_seal(k_i2r, rq_nonce, b"ancord/v1/n2n/req" + sid, pad(rq_pt)))},
        "response": {"nonce": b64u(rs_nonce), "plaintext": rs_pt.decode(),
                     "c": b64u(xchacha_seal(k_r2i, rs_nonce, b"ancord/v1/n2n/res" + sid + rq_nonce, pad(rs_pt)))},
    }

    out = {
        "description": "Zabezpečený kanál (spec §14): klient↔node a node↔node. Vygenerováno spec/reference/channel.py.",
        "padding": [{"len": n, "padded": len(pad(b"a" * n))} for n in (0, 1, 251, 252, 253, 508, 1000)],
        "hchacha20": {"key": b64u(bytes(range(32))), "nonce16": b64u(bytes.fromhex("000000090000004a0000000031415927")),
                      "out": b64u(hchacha20(bytes(range(32)), bytes.fromhex("000000090000004a0000000031415927")))},
        "client_node": chan,
        "node_node": n2n,
    }
    json.dump(out, sys.stdout, indent=2)
    print()


if __name__ == "__main__":
    main()
