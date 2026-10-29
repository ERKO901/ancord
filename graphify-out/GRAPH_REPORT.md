# Graph Report - ancord  (2026-10-29)

## Corpus Check
- 126 files · ~98,275 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 7 file(s) not represented in the graph (top: (none) 5, .example 1, .jsonc 1)

## Summary
- 2050 nodes · 5669 edges · 86 communities (75 shown, 11 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 345 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `fb3fdfa1`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Melo v2 – architektura
- gen-vectors.ts
- scheduler.ts
- push.test.ts
- index.ts
- ancord – protokol (verze 1)
- profile.rs
- run
- package.json
- data.rs
- ratchet.py
- channel.rs
- Ratchet
- compilerOptions
- registry.ts
- utf8
- b64u_decode_array
- registration.ts
- b64u_encode
- space.rs
- WorkerError
- Ephemeral
- ancord-cli/src/lib.rs
- e2e.rs
- ratchet.rs
- SendResult
- user.rs
- client.rs
- duration
- deploy.ts
- dataplane.rs
- Cmd
- digest
- store.rs
- control.rs
- b64u_encode
- online.rs
- WorkerError
- handler.rs
- pairwise.py
- ancord node
- ancord-cli
- ws.ts
- NodeState
- peer.rs
- handler.ts
- push_id
- server.rs
- config.ts
- cloudflare.ts
- wake.rs
- snapshot.ts
- Step
- local-forward.py
- b64uEncode
- CloudflareClient
- ticket.rs
- Šest nodů v Dockeru (C3)
- local.sh
- canonical_bytes
- encoding.rs
- pairwise.py
- services
- establish
- errors.ts
- options
- NODE_NAME
- cpu.ts
- vectors.test.ts
- node1
- node2
- http.ts
- node3
- routing.ts
- register.rs
- node4
- node5
- node6
- .new
- x-node
- SeedRng
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 135 edges
2. `CliError` - 68 edges
3. `unix_now()` - 60 edges
4. `DataError` - 57 edges
5. `b64uEncode()` - 54 edges
6. `b64u_encode()` - 53 edges
7. `Session` - 47 edges
8. `b64u_decode_array()` - 46 edges
9. `b64uDecodeLen()` - 45 edges
10. `HttpError` - 45 edges

## Surprising Connections (you probably didn't know these)
- `5.4 Push notifikace` --references--> `push_id()`  [INFERRED]
  ARCHITECTURE.md → spec/reference/pairwise.py
- `E2E test (B1 až B4)` --references--> `push_id()`  [INFERRED]
  node/README.md → spec/reference/pairwise.py
- `Konfigurace nodu` --references--> `push_id()`  [INFERRED]
  node/README.md → spec/reference/pairwise.py
- `9. Otevřené body` --references--> `dh()`  [INFERRED]
  ARCHITECTURE.md → spec/reference/ratchet.py
- `12.4 Pořadí prací a milníky` --references--> `CloudflareClient`  [INFERRED]
  ARCHITECTURE.md → worker/src/cloudflare.ts

## Import Cycles
- 2-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 2-file cycle: `node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs`
- 2-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/profile.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 3-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 3-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs`
- 3-file cycle: `node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/store.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/state.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/store.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/profile.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/profile.rs -> node/crates/ancord-node/src/data.rs`

## Communities (86 total, 11 thin omitted)

### Community 0 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (28): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+20 more)

### Community 1 - "gen-vectors.ts"
Cohesion: 0.15
Nodes (25): buildTable(), buildTree(), D0, decodeBytes(), decodePath(), depthOf(), DI, LEAF_BYTES (+17 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.12
Nodes (46): blobKey(), ensureBlob(), ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth() (+38 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (44): vitest, sharedMockClient(), Domain, randomBytes(), resetRouteCache(), allMeasured(), node(), open (+36 more)

### Community 4 - "index.ts"
Cohesion: 0.18
Nodes (32): verifyNodeRequest(), clock, workerSigner(), clientInfo(), json(), parseJsonObject(), readBody(), reqString() (+24 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.08
Nodes (25): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+17 more)

### Community 6 - "profile.rs"
Cohesion: 0.11
Nodes (18): blocking(), put_local(), get_local(), handoff(), is_for(), MAX_OTKS, parse_pk(), pow_nonce() (+10 more)

### Community 7 - "run"
Cohesion: 0.11
Nodes (20): create_user(), fetch_route(), load_user(), lock_file(), parse_address(), save_user(), write_private(), Cli (+12 more)

### Community 8 - "package.json"
Cohesion: 0.08
Nodes (29): @cloudflare/vitest-plugin, @cloudflare/workers-types, @noble/hashes, tsx, @types/node, typescript, wrangler, dependencies (+21 more)

### Community 9 - "data.rs"
Cohesion: 0.07
Nodes (32): anchors_in(), check_ts(), BadRequest, DEFAULT_FETCH, fetch(), find_node(), FORBIDDEN_DELAY, FORBIDDEN_RETRIES (+24 more)

### Community 10 - "ratchet.py"
Cohesion: 0.21
Nodes (13): b64u(), bootstrap_seed(), dh(), hkdf(), kdf_rk(), msg_key(), next_ck(), Party (+5 more)

### Community 11 - "channel.rs"
Cohesion: 0.09
Nodes (32): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, ChannelError, BadPublicKey, Decrypt, Exhausted, Padding (+24 more)

### Community 12 - "Ratchet"
Cohesion: 0.13
Nodes (18): aad(), bootstrap_secret(), dh(), expand(), kdf_rk(), message_key(), MessageKey, next_chain_key() (+10 more)

### Community 13 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, exactOptionalPropertyTypes, isolatedModules, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+10 more)

### Community 14 - "registry.ts"
Cohesion: 0.13
Nodes (8): Result, toResult(), AuthCtx, BeginResult, ConfirmInput, ConfirmResult, unbanNode(), Registry

### Community 15 - "utf8"
Cohesion: 0.17
Nodes (7): RFC-8785, canonicalBytes(), canonicalJson(), deletePush(), registerPush(), nowS(), TestNode

### Community 16 - "b64u_decode_array"
Cohesion: 0.24
Nodes (24): b64u_decode(), b64u_decode_array(), verify_ticket(), arr(), bob_profile_push_id(), canonical(), channel_client_node(), channel_node_node() (+16 more)

### Community 17 - "registration.ts"
Cohesion: 0.12
Nodes (25): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu), genesisSet() (+17 more)

### Community 18 - "b64u_encode"
Cohesion: 0.14
Nodes (8): HistoryEntry, Role, Initiator, Responder, Session, load_session(), mailbox_push_key(), NoDisk

### Community 19 - "space.rs"
Cohesion: 0.06
Nodes (24): Backing, File, Mem, CHUNK, D0, DI, LEAF_BYTES, leaf_hash() (+16 more)

### Community 20 - "WorkerError"
Cohesion: 0.15
Nodes (5): BeginResponse, ConfirmResponse, nonce_b64(), signed_header(), WorkerClient

### Community 21 - "Ephemeral"
Cohesion: 0.20
Nodes (8): Channel, channel_roundtrip_detects_reorder_replay_and_tamper(), derive(), different_transcript_yields_different_keys(), Ephemeral, frame_counter_exhaustion_is_fatal(), SessionKeys, OutSession

### Community 22 - "ancord-cli/src/lib.rs"
Cohesion: 0.08
Nodes (47): Accepted, address(), admission(), admit(), admit_unchecked(), CliError, Admission, AllFailed (+39 more)

### Community 23 - "e2e.rs"
Cohesion: 0.05
Nodes (11): main(), PUSH_FILE, ADMIN, admin_nodes(), eventually(), forward(), NodeProc, Proc (+3 more)

### Community 24 - "ratchet.rs"
Cohesion: 0.13
Nodes (16): conversation_heals_after_a_state_leak(), forged_message_does_not_advance_state(), gap_limit(), garbage_is_rejected(), Header, HEADER_LEN, MAX_SKIP, message_of_the_previous_chain_arrives_after_a_dh_step() (+8 more)

### Community 26 - "user.rs"
Cohesion: 0.07
Nodes (43): address_uri(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), both_sides_agree_and_the_one_time_prekey_is_consumed(), dh(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION (+35 more)

### Community 27 - "client.rs"
Cohesion: 0.10
Nodes (31): attach_rid(), authenticate(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED, do_ack(), do_fetch() (+23 more)

### Community 29 - "deploy.ts"
Cohesion: 0.10
Nodes (19): apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS, IP_HEADERS, KV (+11 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.05
Nodes (77): call(), Established, Refused, Session, Exchange, exchange(), Done, UnknownSession (+69 more)

### Community 31 - "Cmd"
Cohesion: 0.09
Nodes (22): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Online (+14 more)

### Community 32 - "digest"
Cohesion: 0.23
Nodes (8): compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start(), leading_zero_bits(), solve(), valid()

### Community 33 - "store.rs"
Cohesion: 0.09
Nodes (30): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_profile(), HEADER_LEN (+22 more)

### Community 34 - "control.rs"
Cohesion: 0.12
Nodes (9): heartbeat(), KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run() (+1 more)

### Community 35 - "b64u_encode"
Cohesion: 0.12
Nodes (8): base32_encode(), HOSTNAME_ID_CHARS, hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected(), node_id_of()

### Community 36 - "online.rs"
Cohesion: 0.18
Nodes (12): ack_inbox(), now_s(), publish_profile(), MAX_SUBSCRIPTIONS, Online, OnlineEvent, Contact, Message (+4 more)

### Community 37 - "WorkerError"
Cohesion: 0.13
Nodes (9): NodeError, WorkerError, Api, Protocol, Transport, main(), resolve_worker_key(), run() (+1 more)

### Community 38 - "handler.rs"
Cohesion: 0.33
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 39 - "pairwise.py"
Cohesion: 0.20
Nodes (13): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+5 more)

### Community 40 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

### Community 44 - "ws.ts"
Cohesion: 0.17
Nodes (21): Json, Config, Signer, RegCtx, broadcastDirectory(), DirectoryBody, DirectoryNode, ensureDirectory() (+13 more)

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (41): ack(), ack_local(), fan_out(), handoff(), peer_post(), plan(), push_record(), send() (+33 more)

### Community 46 - "peer.rs"
Cohesion: 0.15
Nodes (24): ack(), anchored_mailbox(), ApiError, deliver(), dispatch(), exchange(), get(), handshake() (+16 more)

### Community 47 - "handler.ts"
Cohesion: 0.12
Nodes (26): NodeAuth, aesGcmDecrypt(), aesGcmEncrypt(), Env, PushJob, AdmitOptions, AdmitResult, RateLimiter (+18 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (46): 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake`, 10. Push (+38 more)

### Community 49 - "server.rs"
Cohesion: 0.13
Nodes (12): Incoming, Closed, Invalid, Json, bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof() (+4 more)

### Community 50 - "config.ts"
Cohesion: 0.17
Nodes (14): createCloudflareClient(), base, cache, ChallengeKind, DeepPartial, isObject(), loadConfig(), merge() (+6 more)

### Community 51 - "cloudflare.ts"
Cohesion: 0.18
Nodes (10): apply, cf(), Envelope, main(), ApiEnvelope, isManagedHostname(), MANAGED_COMMENT, MockCall (+2 more)

### Community 52 - "wake.rs"
Cohesion: 0.33
Nodes (4): admit(), optional_push_id(), PUSH_ID_LEN, spawn()

### Community 53 - "snapshot.ts"
Cohesion: 0.15
Nodes (21): eligibleForDirectory(), BeginInput, capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf() (+13 more)

### Community 54 - "Step"
Cohesion: 0.40
Nodes (4): Step, Frame, Line, Quit

### Community 56 - "b64uEncode"
Cohesion: 0.11
Nodes (25): BwDir, signBwToken(), verifyBwToken(), hmacSha256(), verifySig(), B64U_LOOKUP, b64uDecode(), b64uEncode() (+17 more)

### Community 58 - "ticket.rs"
Cohesion: 0.16
Nodes (14): DOMAIN_DIRECTORY, DOMAIN_TICKET, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody, TicketError, BadSignature (+6 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 61 - "canonical_bytes"
Cohesion: 0.30
Nodes (7): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value()

### Community 62 - "encoding.rs"
Cohesion: 0.18
Nodes (3): anchors(), deterministic_and_bounded(), weight()

### Community 63 - "pairwise.py"
Cohesion: 0.15
Nodes (9): b64u(), hkdf(), mailbox(), profile(), raw(), sha(), solve(), x3dh() (+1 more)

### Community 64 - "services"
Cohesion: 0.14
Nodes (13): name, services, volumes, node1-data, node2-data, node3-data, node4-data, node5-data (+5 more)

### Community 65 - "establish"
Cohesion: 0.27
Nodes (8): Bucket, close(), ConnGuard, establish(), recv_plain(), reject(), Secure, session()

### Community 66 - "errors.ts"
Cohesion: 0.14
Nodes (12): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura, ErrorCode (+4 more)

### Community 67 - "options"
Cohesion: 0.29
Nodes (11): driver, options, logging, logging, logging, logging, logging, logging (+3 more)

### Community 68 - "NODE_NAME"
Cohesion: 0.53
Nodes (10): NODE_NAME, RUST_LOG, WORKER_PUBKEY, WORKER_URL, environment, environment, environment, environment (+2 more)

### Community 69 - "cpu.ts"
Cohesion: 0.21
Nodes (14): advance(), advanceNative(), computeCheckpoints(), CpuParams, CpuVerdict, laneStart(), verifyCpu(), b64uDecodeLen() (+6 more)

### Community 70 - "vectors.test.ts"
Cohesion: 0.08
Nodes (31): keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload(), ED25519 (+23 more)

### Community 71 - "node1"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 72 - "node2"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 73 - "http.ts"
Cohesion: 0.23
Nodes (10): HttpError, expandIpv6(), ipPrefix(), MAX_BODY_BYTES, optInt(), reqB64(), reqInt(), reqObject() (+2 more)

### Community 74 - "node3"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 75 - "routing.ts"
Cohesion: 0.32
Nodes (7): ClientGeo, distanceKm(), haversineKm(), pickP2C(), Snapshot, SnapshotNode, weightedIndex()

### Community 76 - "register.rs"
Cohesion: 0.21
Nodes (7): hostname_matches(), activate(), register(), Registration, DOMAIN_REGISTER, run(), backoff()

### Community 77 - "node4"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 78 - "node5"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 79 - "node6"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 80 - ".new"
Cohesion: 0.29
Nodes (5): Direction, InitiatorToResponder, ResponderToInitiator, mailbox_id(), root_key()

### Community 81 - "x-node"
Cohesion: 0.22
Nodes (9): x-node, build, cap_drop, image, read_only, restart, security_opt, stop_grace_period (+1 more)

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

## Knowledge Gaps
- **89 isolated node(s):** `ancord-cli`, `Route`, `Admission`, `Node`, `Session` (+84 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 611 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **11 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `b64uEncode`, `push.test.ts`, `utf8`?**
  _High betweenness centrality (0.373) - this node is a cross-community bridge._
- **Are the 44 inferred relationships involving `unix_now()` (e.g. with `authenticate()` and `do_send()`) actually correct?**
  _`unix_now()` has 44 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ancord-cli`, `Route`, `Admission` to the rest of the system?**
  _89 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Melo v2 – architektura` be split into smaller, more focused modules?**
  _Cohesion score 0.06896551724137931 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `gen-vectors.ts`, `scheduler.ts`, `push.test.ts`, `index.ts`, `cpu.ts`, `vectors.test.ts`, `ws.ts`, `handler.ts`, `utf8`, `registration.ts`, `push_id`, `snapshot.ts`?**
  _High betweenness centrality (0.251) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.12074829931972789 - nodes in this community are weakly interconnected._
- **Why does `mailbox_id()` connect `push_id` to `ancord node`, `ratchet.py`?**
  _High betweenness centrality (0.242) - this node is a cross-community bridge._