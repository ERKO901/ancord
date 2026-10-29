//! Sdílené testovací vektory z `spec/test-vectors/` (jediné, co node sdílí s Workerem).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use ancord_node::canonical::canonical_json;
use ancord_node::encoding::{b64u_decode, b64u_decode_array, b64u_encode, hex_encode};
use ancord_node::hashcash;
use ancord_node::identity::{hostname_of, node_id_of, Identity};
use ancord_node::sign;
use serde_json::Value;

fn load(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../spec/test-vectors").join(name);
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).unwrap()
}

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or_else(|| panic!("missing {key}"))
}

fn identity_of(seed: &str) -> Identity {
    Identity::from_seed(b64u_decode_array(seed).unwrap())
}

#[test]
fn identity() {
    let v = load("identity.json");
    for case in v["cases"].as_array().unwrap() {
        let id = identity_of(s(case, "seed"));
        assert_eq!(id.pubkey_b64(), s(case, "pubkey"));
        assert_eq!(id.node_id(), s(case, "node_id"));
        assert_eq!(node_id_of(&b64u_decode_array(s(case, "pubkey")).unwrap()), s(case, "node_id"));
        assert_eq!(hostname_of(id.node_id(), s(&v, "zone")), s(case, "hostname"));
    }
}

#[test]
fn canonical() {
    let v = load("canonical-json.json");
    for case in v["cases"].as_array().unwrap() {
        assert_eq!(canonical_json(&case["input"]).unwrap(), s(case, "canonical"));
    }
}

#[test]
fn register_signature_and_hashcash() {
    let v = load("register.json");
    let id = identity_of(s(&v, "seed"));
    let nonce: [u8; 32] = b64u_decode_array(s(&v, "nonce")).unwrap();
    let mut payload = nonce.to_vec();
    payload.extend_from_slice(s(&v, "hostname").as_bytes());
    payload.extend_from_slice(&id.pubkey());
    assert_eq!(hex_encode(&sign::signing_input(sign::DOMAIN_REGISTER, &payload)), s(&v, "signing_input_hex"));
    assert_eq!(b64u_encode(&id.sign(sign::DOMAIN_REGISTER, &payload)), s(&v, "signature"));

    let hc = &v["hashcash"];
    let bits = hc["bits"].as_u64().unwrap() as u32;
    let counter = hc["counter"].as_u64().unwrap();
    assert!(hashcash::valid(&nonce, &id.pubkey(), counter, bits));
    assert!(counter == 0 || !hashcash::valid(&nonce, &id.pubkey(), counter - 1, bits));
    assert_eq!(hashcash::solve(&nonce, &id.pubkey(), bits, 4), counter);
}

#[test]
fn request_auth() {
    let v = load("request-auth.json");
    let id = identity_of(s(&v, "seed"));
    let t = v["t"].as_u64().unwrap();
    let body = s(&v, "body").as_bytes();
    let payload = sign::request_payload(s(&v, "method"), s(&v, "path_and_query"), t, s(&v, "n"), body);
    assert_eq!(String::from_utf8(payload).unwrap(), s(&v, "payload"));

    let nonce: [u8; 16] = b64u_decode_array(s(&v, "n")).unwrap();
    let header = sign::auth_header(id.signing_key(), s(&v, "method"), s(&v, "path_and_query"), body, t, &nonce);
    assert_eq!(header, s(&v, "authorization"));

    let parsed = sign::verify_request(&header, s(&v, "method"), s(&v, "path_and_query"), body, t + 5, 60).unwrap();
    assert_eq!(parsed.pubkey, id.pubkey());
    assert!(sign::verify_request(&header, "GET", s(&v, "path_and_query"), body, t, 60).is_none());
    assert!(sign::verify_request(&header, s(&v, "method"), s(&v, "path_and_query"), body, t + 61, 60).is_none());
}

#[test]
fn heartbeat() {
    let v = load("heartbeat.json");
    let id = identity_of(s(&v, "seed"));
    let canon = canonical_json(&v["message"]).unwrap();
    assert_eq!(canon, s(&v, "canonical"));
    assert_eq!(b64u_encode(&id.sign(sign::DOMAIN_HEARTBEAT, canon.as_bytes())), s(&v, "sig"));
}

#[test]
fn proof() {
    let v = load("proof.json");
    let id = identity_of(s(&v, "seed"));
    let c = b64u_decode(s(&v, "challenge")).unwrap();
    let mut payload = c;
    payload.extend_from_slice(s(&v, "hostname").as_bytes());
    payload.extend_from_slice(&id.pubkey());
    assert_eq!(b64u_encode(&id.sign(sign::DOMAIN_PROOF, &payload)), s(&v, "signature"));
}

#[test]
fn hashchain() {
    let v = load("hashchain.json");
    let seed: [u8; 32] = b64u_decode_array(s(&v, "seed")).unwrap();
    let got = ancord_node::challenge::cpu::compute_checkpoints(
        &seed,
        v["lanes"].as_u64().unwrap() as u32,
        v["iters"].as_u64().unwrap(),
        v["every"].as_u64().unwrap(),
    )
    .unwrap();
    let want: Vec<Vec<String>> = serde_json::from_value(v["checkpoints"].clone()).unwrap();
    assert_eq!(got, want);
}

#[test]
fn space() {
    let v = load("space.json");
    let seed: [u8; 32] = b64u_decode_array(s(&v, "seed")).unwrap();
    let t = ancord_node::challenge::space::SpaceTable::build(seed, v["leaves"].as_u64().unwrap(), None).unwrap();
    assert_eq!(b64u_encode(&t.root()), s(&v, "root"));
    for o in v["openings"].as_array().unwrap() {
        let got = t.opening(o["i"].as_u64().unwrap()).unwrap();
        assert_eq!(&got, o);
    }
}

#[test]
fn ticket() {
    use ancord_node::ticket::{verify_ticket, TicketError, TicketGuard};
    let v = load("ticket.json");
    let key: [u8; 32] = b64u_decode_array(s(&v, "worker_pubkey")).unwrap();
    let valid_at = v["valid_at"].as_u64().unwrap();
    let body = verify_ticket(&key, s(&v, "ticket"), s(&v, "node_id"), valid_at).unwrap();
    assert_eq!(body.ek, s(&v, "ek"));
    assert_eq!(body.n, s(&v, "nonce"));
    assert_eq!(verify_ticket(&key, s(&v, "ticket"), s(&v, "node_id"), v["expired_at"].as_u64().unwrap()), Err(TicketError::Expired));
    assert_eq!(verify_ticket(&key, s(&v, "ticket"), "other", valid_at), Err(TicketError::WrongNode));
    let mut forged = s(&v, "ticket").to_owned();
    forged.insert(3, 'A');
    assert!(verify_ticket(&key, &forged, s(&v, "node_id"), valid_at).is_err());
    let guard = TicketGuard::default();
    assert!(guard.consume(&body, valid_at).is_ok());
    assert_eq!(guard.consume(&body, valid_at), Err(TicketError::Replayed));
}

#[test]
fn directory() {
    let v = load("directory.json");
    let key: [u8; 32] = b64u_decode_array(s(&v, "worker_pubkey")).unwrap();
    let envelope = serde_json::json!({"body": v["body"], "sig": v["sig"], "kid": v["kid"]});
    let dir = ancord_node::ticket::verify_directory(&key, &envelope).unwrap();
    assert_eq!(dir.epoch, 41);
    assert_eq!(dir.nodes.len(), 3);
    let mut tampered = envelope.clone();
    tampered["body"]["epoch"] = serde_json::json!(42);
    assert!(ancord_node::ticket::verify_directory(&key, &tampered).is_none());
}

#[test]
fn space_file_backed_matches_memory() {
    use ancord_node::challenge::space::SpaceTable;
    let dir = tempfile::tempdir().unwrap();
    let seed = [7u8; 32];
    let mem = SpaceTable::build(seed, 1 << 12, None).unwrap();
    let file = SpaceTable::build(seed, 1 << 12, Some((dir.path(), "storage"))).unwrap();
    assert_eq!(mem.root(), file.root());
    for i in [1u64, 2, 777, 4095] {
        assert_eq!(mem.opening(i).unwrap(), file.opening(i).unwrap());
    }
    assert!(file.opening(0).is_err());
    assert!(file.opening(4096).is_err());
    drop(file);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

fn arr<const N: usize>(v: &Value, key: &str) -> [u8; N] {
    b64u_decode_array(s(v, key)).unwrap_or_else(|| panic!("bad {key}"))
}

#[test]
fn channel_client_node() {
    use ancord_node::channel::{client_transcript, Channel, Ephemeral};
    use zeroize::Zeroizing;
    let all = load("channel.json");
    let v = &all["client_node"];
    let node = identity_of(s(v, "node_seed"));
    assert_eq!(node.node_id(), s(v, "node_id"));
    assert_eq!(node.pubkey_b64(), s(v, "npk"));

    let ce = Ephemeral::from_bytes(Zeroizing::new(arr(v, "ce_secret")));
    let ne = Ephemeral::from_bytes(Zeroizing::new(arr(v, "ne_secret")));
    assert_eq!(b64u_encode(&ce.public()), s(v, "ce"));
    assert_eq!(b64u_encode(&ne.public()), s(v, "ne"));
    let th = client_transcript(&arr(v, "challenge"), &ce.public(), &ne.public(), &node.pubkey(), node.node_id());
    assert_eq!(b64u_encode(&th), s(v, "th"));

    let node_sig = node.sign(sign::DOMAIN_CHAN_NODE, &th);
    assert_eq!(b64u_encode(&node_sig), s(v, "node_sig"));
    let ek = identity_of(s(v, "ek_seed"));
    assert_eq!(ek.pubkey_b64(), s(v, "ek"));
    assert_eq!(b64u_encode(&ek.sign(sign::DOMAIN_CHAN_CLIENT, &th)), s(v, "client_sig"));

    let (cpub, npub) = (ce.public(), ne.public());
    let shared = ce.diffie_hellman(&npub).unwrap();
    assert_eq!(b64u_encode(&*shared), s(v, "shared"));
    assert_eq!(*ne.diffie_hellman(&cpub).unwrap(), *shared);

    // zachycené rámce z referenční implementace se dešifrují a naše rámce jim odpovídají bajt po bajtu
    let mut node_side = Channel::for_node(&shared, &th);
    for f in v["c2n_frames"].as_array().unwrap() {
        let frame = b64u_decode(s(f, "frame")).unwrap();
        assert_eq!(node_side.recv.open(&frame).unwrap(), s(f, "plaintext").as_bytes());
    }
    let mut client_side = Channel::for_client(&shared, &th);
    for f in v["c2n_frames"].as_array().unwrap() {
        assert_eq!(b64u_encode(&client_side.send.seal(s(f, "plaintext").as_bytes()).unwrap()), s(f, "frame"));
    }
    let mut node_out = Channel::for_node(&shared, &th);
    for f in v["n2c_frames"].as_array().unwrap() {
        assert_eq!(b64u_encode(&node_out.send.seal(s(f, "plaintext").as_bytes()).unwrap()), s(f, "frame"));
        let frame = b64u_decode(s(f, "frame")).unwrap();
        assert_eq!(client_side.recv.open(&frame).unwrap(), s(f, "plaintext").as_bytes());
    }
}

#[test]
fn channel_padding() {
    let all = load("channel.json");
    for case in all["padding"].as_array().unwrap() {
        let len = case["len"].as_u64().unwrap() as usize;
        assert_eq!(ancord_node::channel::padded_len(len), case["padded"].as_u64().unwrap() as usize);
    }
}

#[test]
fn channel_node_node() {
    use ancord_node::channel::{n2n_transcript, open_request, open_response, seal_request, seal_response, Ephemeral, SessionKeys};
    use zeroize::Zeroizing;
    let all = load("channel.json");
    let v = &all["node_node"];
    let (initiator, responder) = (identity_of(s(v, "initiator_seed")), identity_of(s(v, "responder_seed")));
    assert_eq!(initiator.pubkey_b64(), s(v, "initiator_pk"));
    assert_eq!(responder.pubkey_b64(), s(v, "responder_pk"));
    let ce = Ephemeral::from_bytes(Zeroizing::new(arr(v, "ce_secret")));
    let ne = Ephemeral::from_bytes(Zeroizing::new(arr(v, "ne_secret")));
    assert_eq!((b64u_encode(&ce.public()), b64u_encode(&ne.public())), (s(v, "ce").to_owned(), s(v, "ne").to_owned()));
    let sid: [u8; 16] = arr(v, "sid");
    let th = n2n_transcript(&initiator.pubkey(), &responder.pubkey(), &arr(v, "request_nonce"), &ce.public(), &ne.public(), &sid);
    assert_eq!(b64u_encode(&th), s(v, "th"));
    assert_eq!(b64u_encode(&responder.sign(sign::DOMAIN_N2N_RESP, &th)), s(v, "resp_sig"));
    let npub = ne.public();
    let shared = ce.diffie_hellman(&npub).unwrap();
    assert_eq!(b64u_encode(&*shared), s(v, "shared"));
    let keys = SessionKeys::derive(&shared, &th);
    assert_eq!(b64u_encode(&*keys.i2r), s(v, "k_i2r"));
    assert_eq!(b64u_encode(&*keys.r2i), s(v, "k_r2i"));

    let rq = &v["request"];
    let rq_nonce: [u8; 24] = arr(rq, "nonce");
    let sealed = seal_request(&keys.i2r, &sid, &rq_nonce, s(rq, "plaintext").as_bytes()).unwrap();
    assert_eq!(b64u_encode(&sealed), s(rq, "c"));
    assert_eq!(open_request(&keys.i2r, &sid, &rq_nonce, &sealed).unwrap(), s(rq, "plaintext").as_bytes());
    let rs = &v["response"];
    let rs_nonce: [u8; 24] = arr(rs, "nonce");
    let sealed = seal_response(&keys.r2i, &sid, &rq_nonce, &rs_nonce, s(rs, "plaintext").as_bytes()).unwrap();
    assert_eq!(b64u_encode(&sealed), s(rs, "c"));
    assert_eq!(open_response(&keys.r2i, &sid, &rq_nonce, &rs_nonce, &sealed).unwrap(), s(rs, "plaintext").as_bytes());
}

#[test]
fn hrw() {
    let v = load("hrw.json");
    let mailbox: [u8; 32] = b64u_decode_array(s(&v, "mailbox_id")).unwrap();
    let ids: Vec<String> = serde_json::from_value(v["node_ids"].clone()).unwrap();
    for case in v["cases"].as_array().unwrap() {
        let epoch = case["epoch"].as_u64().unwrap();
        for w in case["weights"].as_array().unwrap() {
            assert_eq!(hex_encode(&ancord_node::hrw::weight(&mailbox, s(w, "id"), epoch)), s(w, "weight_hex"));
        }
        let got = ancord_node::hrw::anchors(&mailbox, ids.iter().map(String::as_str), epoch, case["r"].as_u64().unwrap() as usize);
        let want: Vec<String> = serde_json::from_value(case["anchors"].clone()).unwrap();
        assert_eq!(got, want);
    }
}

/// Deterministický zdroj klíčů: každé použití vrátí další zárodek ze seznamu (stejné pořadí jako v referenčním Pythonu).
struct SeedRng(std::collections::VecDeque<[u8; 32]>);

impl rand::RngCore for SeedRng {
    fn next_u32(&mut self) -> u32 {
        unreachable!("only fill_bytes is used")
    }
    fn next_u64(&mut self) -> u64 {
        unreachable!("only fill_bytes is used")
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let seed = self.0.pop_front().expect("the vector provides enough key seeds");
        dest.copy_from_slice(&seed);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rand::CryptoRng for SeedRng {}

#[test]
fn ratchet() {
    use ancord_node::ratchet::{bootstrap_secret, mailbox_id, Direction, Ratchet};
    use x25519_dalek::{PublicKey, StaticSecret};
    let v = load("ratchet.json");
    let ss: [u8; 32] = b64u_decode_array(s(&v, "shared_secret")).unwrap();
    assert_eq!(v["max_skip"].as_u64().unwrap(), u64::from(ancord_node::ratchet::MAX_SKIP));
    assert_eq!(b64u_encode(&mailbox_id(&ss, Direction::InitiatorToResponder)), s(&v["mailboxes"], "i2r"));
    assert_eq!(b64u_encode(&mailbox_id(&ss, Direction::ResponderToInitiator)), s(&v["mailboxes"], "r2i"));
    assert_eq!(b64u_encode(&bootstrap_secret(&ss).to_bytes()), s(&v, "bootstrap_seed"));
    let seeds = |who: &str| -> std::collections::VecDeque<[u8; 32]> {
        v["key_seeds"][who].as_array().unwrap().iter().map(|k| b64u_decode_array(k.as_str().unwrap()).unwrap()).collect()
    };
    let spk_secret = StaticSecret::from(b64u_decode_array::<32>(s(&v, "responder_ratchet_seed")).unwrap());

    for scenario in v["scenarios"].as_array().unwrap() {
        let name = s(scenario, "name");
        let (mut a_rng, mut b_rng) = (SeedRng(seeds("a")), SeedRng(seeds("b")));
        let responder_key = if name == "shared-secret" { bootstrap_secret(&ss) } else { spk_secret.clone() };
        let mut a = Ratchet::initiator_with_rng(&ss, &PublicKey::from(&responder_key).to_bytes(), &mut a_rng).unwrap();
        let mut b = Ratchet::responder(&ss, &responder_key);
        let mut sent: Vec<([u8; 16], String)> = Vec::new();
        for (i, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
            let (me, rng) = if s(step, "who") == "a" { (&mut a, &mut a_rng) } else { (&mut b, &mut b_rng) };
            let plaintext = b64u_decode(s(step, "plaintext_b64")).unwrap();
            if s(step, "op") == "send" {
                let id: [u8; 16] = b64u_decode_array(s(step, "id")).unwrap();
                // odesílatel musí vyrobit přesně stejné bajty jako referenční implementace
                assert_eq!(me.seal(&id, &plaintext).unwrap(), s(step, "ct"), "{name} step {i}");
                sent.push((id, s(step, "ct").to_owned()));
            } else {
                let (id, ct) = &sent[step["message"].as_u64().unwrap() as usize];
                let (seq, got) = me.open_numbered_with_rng(id, ct, rng).unwrap();
                assert_eq!(got, plaintext, "{name} step {i}");
                assert_eq!(u64::from(seq), step["seq"].as_u64().unwrap(), "{name} step {i}");
            }
        }
    }
}

#[test]
fn pairwise() {
    use ancord_node::profile::{profile_id, Profile};
    use ancord_node::ratchet::{mailbox_id, Direction};
    use ancord_node::user::{inbox_id, mailbox_push_key, safety_number, UserKeys};
    use ancord_node::wake::push_id_of;
    use x25519_dalek::StaticSecret;
    let v = load("pairwise.json");
    let bits = u32::try_from(v["pow_bits"].as_u64().unwrap()).unwrap();
    let now = v["now"].as_u64().unwrap();
    let seed = |x: &Value, k: &str| -> [u8; 32] { b64u_decode_array(s(x, k)).unwrap() };
    let (alice, bob) = (&v["alice"], &v["bob"]);
    let otk_seeds: Vec<[u8; 32]> = bob["otk_seeds"].as_array().unwrap().iter().map(|o| b64u_decode_array(o.as_str().unwrap()).unwrap()).collect();

    // Alice: žádný prekey nepotřebuje k odeslání, její profil ale musí projít ověřením a odpovídat klíčům
    let alice_keys = UserKeys::from_parts(seed(alice, "uk_seed"), seed(alice, "ik_seed"), [9; 32], now, &[], seed(alice, "push_seed"));
    let alice_profile = Profile::from_json(&alice["profile"]).unwrap();
    alice_profile.check(now, bits).unwrap();
    assert_eq!(alice_keys.profile(now, bits), alice_profile, "Ed25519 is deterministic, so the profile is byte-identical");

    let mut bob_keys = UserKeys::from_parts(seed(bob, "uk_seed"), seed(bob, "ik_seed"), seed(bob, "spk_seed"), now, &otk_seeds, seed(bob, "push_seed"));
    assert_eq!(bob_keys.push_id(), bob_profile_push_id(bob));
    let bob_profile = Profile::from_json(&bob["profile"]).unwrap();
    bob_profile.check(now, bits).unwrap();
    assert_eq!(bob_keys.profile(now, bits), bob_profile);
    assert_eq!(b64u_encode(&profile_id(&bob_keys.pk())), b64u_encode(&profile_id(&bob_profile.pk)));
    assert_eq!(b64u_encode(&inbox_id(&bob_keys.pk())), s(bob, "inbox_id"));

    // X3DH a obálka: stejné tajemství i stejné bajty obálky jako v nezávislém Pythonu
    let h = &v["handshake"];
    let ek = StaticSecret::from(seed(h, "ek_seed"));
    let index = usize::try_from(h["otk_index"].as_u64().unwrap()).unwrap();
    let init = alice_keys.initiate_with(&bob_profile, now, &ek, Some(index)).unwrap();
    assert_eq!(b64u_encode(&init.pairwise.secret), s(h, "secret"));
    assert_eq!(b64u_encode(&init.envelope), s(h, "envelope"));

    let opened = bob_keys.open_envelope(&b64u_decode(s(h, "envelope")).unwrap()).unwrap();
    assert_eq!(opened.peer_pk, alice_keys.pk());
    let accepted = bob_keys.accept(&opened, &alice_profile).unwrap();
    assert_eq!(b64u_encode(&accepted.secret), s(h, "secret"));
    assert_eq!(bob_keys.otk_count(), 1, "the used one-time prekey is deleted");

    assert_eq!(safety_number(&alice_keys.pk(), &bob_keys.pk()), s(&v, "safety_number"));

    // push klíče mailboxů konverzace (§15.11)
    for dir in [Direction::InitiatorToResponder, Direction::ResponderToInitiator] {
        let p = &v["mailbox_push"][dir.as_str()];
        let mailbox = mailbox_id(&accepted.secret, dir);
        assert_eq!(b64u_encode(&mailbox), s(p, "mailbox_id"));
        let key = mailbox_push_key(&accepted.secret, &mailbox);
        assert_eq!(b64u_encode(&key.to_bytes()), s(p, "push_seed"));
        assert_eq!(push_id_of(&key.verifying_key().to_bytes()), s(p, "push_id"));
    }
}

fn bob_profile_push_id(bob: &Value) -> &str {
    s(&bob["profile"], "push_id")
}
