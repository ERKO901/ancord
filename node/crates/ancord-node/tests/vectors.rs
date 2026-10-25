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

#[test]
fn client_auth() {
    let v = load("client-auth.json");
    let client = identity_of(s(&v, "client_seed"));
    assert_eq!(client.pubkey_b64(), s(&v, "ek"));
    let mut payload = b64u_decode(s(&v, "challenge")).unwrap();
    payload.extend_from_slice(s(&v, "node_id").as_bytes());
    let sig = client.sign(sign::DOMAIN_CLIENT_AUTH, &payload);
    assert_eq!(b64u_encode(&sig), s(&v, "signature"));
    assert!(sign::verify(&client.pubkey(), sign::DOMAIN_CLIENT_AUTH, &payload, &sig));
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

#[test]
fn ratchet() {
    use ancord_node::ratchet::{mailbox_id, Direction, Receiver, Sender};
    let v = load("ratchet.json");
    let ss: [u8; 32] = b64u_decode_array(s(&v, "shared_secret")).unwrap();
    assert_eq!(v["max_skip"].as_u64().unwrap(), u64::from(ancord_node::ratchet::MAX_SKIP));
    for d in v["dirs"].as_array().unwrap() {
        let dir = match s(d, "dir") {
            "i2r" => Direction::InitiatorToResponder,
            _ => Direction::ResponderToInitiator,
        };
        assert_eq!(b64u_encode(&mailbox_id(&ss, dir)), s(d, "mailbox_id"));
        let mut tx = Sender::new(&ss, dir);
        let mut rx = Receiver::new(&ss, dir);
        for m in d["messages"].as_array().unwrap() {
            let id: [u8; 16] = b64u_decode_array(s(m, "id")).unwrap();
            let plaintext = b64u_decode(s(m, "plaintext_b64")).unwrap();
            // odesílatel musí vyrobit přesně stejné bajty jako referenční implementace
            assert_eq!(tx.seal(&id, &plaintext).unwrap(), s(m, "ct"));
            // příjemce referenční ciphertext otevře
            assert_eq!(rx.open(&id, s(m, "ct")).unwrap(), plaintext);
        }
    }
}
