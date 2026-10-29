#!/usr/bin/env python3
"""Referenční výpočet pro spec §15 (profil, X3DH, obálka, schránka, bezpečnostní číslo). Nezávislý na Rustu.

Přegeneruje spec/test-vectors/pairwise.json:
`uv run --no-project --with cryptography --with blake3 python3 spec/reference/pairwise.py > spec/test-vectors/pairwise.json`
Vyžaduje balíky `cryptography` a `blake3`.
"""
import base64, hashlib, json, struct
import blake3
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey, X25519PublicKey
from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
from cryptography.hazmat.primitives.hashes import SHA256
from cryptography.hazmat.primitives.kdf.hkdf import HKDF
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat


def b64u(b): return base64.urlsafe_b64encode(b).rstrip(b"=").decode()
def raw(pub): return pub.public_bytes(Encoding.Raw, PublicFormat.Raw)
def sha(*parts): return hashlib.sha256(b"".join(parts)).digest()
def hkdf(salt, ikm, info, n): return HKDF(algorithm=SHA256(), length=n, salt=salt, info=info).derive(ikm)
def x(priv, pub): return priv.exchange(X25519PublicKey.from_public_bytes(pub))
def xpub(priv): return raw(priv.public_key())
def xkey(seed): return X25519PrivateKey.from_private_bytes(seed)
def push_id(seed): return base64.b32encode(blake3.blake3(raw(Ed25519PrivateKey.from_private_bytes(seed).public_key())).digest()).decode().lower().rstrip("=")


def solve(nonce, pk, bits):
    c = 0
    while True:
        h = hashlib.sha256(nonce + pk + struct.pack("<Q", c)).digest()
        if int.from_bytes(h, "big") >> (256 - bits) == 0:
            return c
        c += 1


def profile(uk, ik, spk, otks, push, ts, bits):
    pk = raw(uk.public_key())
    payload = xpub(ik) + xpub(spk) + bytes([len(otks)]) + b"".join(xpub(o) for o in otks) + struct.pack(">Q", ts) + push.encode()
    sig = uk.sign(b"ancord/v1/profile\0" + payload)
    return {
        "pk": b64u(pk), "bk": b64u(xpub(ik)), "spk": b64u(xpub(spk)), "otks": [b64u(xpub(o)) for o in otks],
        "push_id": push, "ts": ts, "pow": solve(sha(b"ancord/v1/profile-pow\0", payload), pk, bits), "sig": b64u(sig),
    }


def x3dh(dhs, pk_i, pk_r, ek, spk, otk):
    ikm = b"\xff" * 32 + b"".join(dhs)
    return hkdf(b"ancord/v1/x3dh", ikm, pk_i + pk_r + ek + spk + (otk or b""), 32)


BITS, TS = 8, 1700000000
seed = lambda n: bytes([n] * 32)
a_uk, a_ik = Ed25519PrivateKey.from_private_bytes(seed(1)), xkey(seed(2))
b_uk, b_ik, b_spk = Ed25519PrivateKey.from_private_bytes(seed(3)), xkey(seed(4)), xkey(seed(5))
b_otks = [xkey(seed(6)), xkey(seed(7))]
ek = xkey(seed(8))
a_pk, b_pk = raw(a_uk.public_key()), raw(b_uk.public_key())

b_profile = profile(b_uk, b_ik, b_spk, b_otks, push_id(seed(10)), TS, BITS)
a_profile = profile(a_uk, a_ik, xkey(seed(9)), [], push_id(seed(11)), TS, BITS)

# iniciátor (Alice) použije druhý jednorázový prekey
otk = xpub(b_otks[1])
dhs = [x(a_ik, xpub(b_spk)), x(ek, xpub(b_ik)), x(ek, xpub(b_spk)), x(ek, otk)]
secret = x3dh(dhs, a_pk, b_pk, xpub(ek), xpub(b_spk), otk)
# příjemce (Bob) dojde ke stejnému
dhs_b = [x(b_spk, xpub(a_ik)), x(b_ik, xpub(ek)), x(b_spk, xpub(ek)), x(b_otks[1], xpub(ek))]
assert secret == x3dh(dhs_b, a_pk, b_pk, xpub(ek), xpub(b_spk), otk)

inbox = sha(b"ancord/v1/inbox\0", b_pk)
spk_id = sha(xpub(b_spk))[:8]
okm = hkdf(b"ancord/v1/envelope", x(ek, xpub(b_spk)), inbox + xpub(ek) + spk_id, 44)
aad = b"ancord/v1/envelope" + inbox + xpub(ek) + spk_id
envelope = b"\x01" + spk_id + xpub(ek) + ChaCha20Poly1305(okm[:32]).encrypt(okm[32:], a_pk + otk, aad)

# push klíče mailboxů konverzace (§15.11)
def mailbox(ss, d): return sha(b"ancord/v1/mailbox\0", ss, b"\0", d.encode())
push = {}
for d in ("i2r", "r2i"):
    m = mailbox(secret, d)
    ps = hkdf(b"ancord/v1/push", secret, m, 32)
    push[d] = {"mailbox_id": b64u(m), "push_seed": b64u(ps), "push_id": push_id(ps)}

lo, hi = sorted([a_pk, b_pk])
d = sha(b"ancord/v1/safety\0", lo, hi)
safety = " ".join("%05d" % (int.from_bytes(d[i:i + 2], "big") % 100000) for i in range(0, 24, 2))

print(json.dumps({
    "description": "Profil, X3DH, obálka, schránka, push klíče a bezpečnostní číslo (spec §15), vygenerováno spec/reference/pairwise.py (Python, cryptography)",
    "pow_bits": BITS,
    "now": TS,
    "alice": {"uk_seed": b64u(seed(1)), "ik_seed": b64u(seed(2)), "push_seed": b64u(seed(11)), "profile": a_profile},
    "bob": {
        "uk_seed": b64u(seed(3)), "ik_seed": b64u(seed(4)), "spk_seed": b64u(seed(5)), "push_seed": b64u(seed(10)),
        "otk_seeds": [b64u(seed(6)), b64u(seed(7))], "profile": b_profile, "inbox_id": b64u(inbox),
    },
    "handshake": {
        "ek_seed": b64u(seed(8)), "otk_index": 1, "secret": b64u(secret), "envelope": b64u(envelope),
    },
    "mailbox_push": push,
    "safety_number": safety,
}, indent=2))
