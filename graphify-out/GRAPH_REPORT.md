# Graph Report - ancord  (2026-10-29)

## Corpus Check
- 119 files · ~93,920 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 4 file(s) not represented in the graph (top: (none) 3, .jsonc 1)

## Summary
- 1886 nodes · 5460 edges · 69 communities (61 shown, 8 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 340 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `911ac357`
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
- ratchet.rs
- compilerOptions
- registry.ts
- utf8
- b64u_decode_array
- registration.ts
- b64u_encode
- space.rs
- WorkerError
- Ephemeral
- NodeSocket
- e2e.rs
- ancord Worker
- senders.ts
- user.rs
- client.rs
- register.rs
- NodeState
- dataplane.rs
- Cmd
- ratchet.rs
- store.rs
- control.rs
- b64u_encode
- online.rs
- ancord-node/src/main.rs
- handler.rs
- pairwise.py
- ancord node
- ancord-cli
- ws.ts
- n2n.rs
- peer.rs
- three_nodes_register_challenge_route_and_admit
- push_id
- ticket.rs
- config.ts
- server.rs
- wake.rs
- snapshot.ts
- CliError
- local-forward.py
- encoding.ts
- context.ts
- UserKeys
- dev
- local.sh
- canonical_bytes
- SeedRng
- super
- Direction
- cpu.ts
- vectors.test.ts
- ancord-cli/src/lib.rs

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
- `12.4 Pořadí prací a milníky` --references--> `CloudflareClient`  [INFERRED]
  ARCHITECTURE.md → worker/src/cloudflare.ts
- `1.3 Podpisy a doménová separace` --references--> `Domain`  [INFERRED]
  spec/PROTOCOL.md → worker/src/crypto.ts

## Import Cycles
- 1-file cycle: `node/crates/ancord-node/src/challenge/space.rs -> node/crates/ancord-node/src/challenge/space.rs`
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

## Communities (69 total, 8 thin omitted)

### Community 0 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (26): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code), 1. Cíle a hranice (+18 more)

### Community 1 - "gen-vectors.ts"
Cohesion: 0.14
Nodes (24): buildTable(), buildTree(), D0, decodeBytes(), decodePath(), depthOf(), DI, LEAF_BYTES (+16 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.12
Nodes (44): ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth(), Dim, DIM_COL (+36 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (44): vitest, sharedMockClient(), withOverrides(), Domain, resetRouteCache(), allMeasured(), node(), open (+36 more)

### Community 4 - "index.ts"
Cohesion: 0.05
Nodes (69): 12.2 Pravidla práce, ancord, Cloudflare (firemní zóna melonet.cz), Konvence, Postup, Příkazy, Struktura, NodeAuth (+61 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.08
Nodes (25): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+17 more)

### Community 6 - "profile.rs"
Cohesion: 0.13
Nodes (20): blocking(), fan_out(), local(), OpError, Failed, Full, peer_post(), get() (+12 more)

### Community 7 - "run"
Cohesion: 0.11
Nodes (22): create_user(), load_user(), lock_file(), parse_address(), save_user(), write_private(), Cli, conversation_files() (+14 more)

### Community 8 - "package.json"
Cohesion: 0.09
Nodes (26): @cloudflare/vitest-plugin, @cloudflare/workers-types, @noble/hashes, tsx, typescript, wrangler, dependencies, @noble/hashes@^2.4.0 (+18 more)

### Community 9 - "data.rs"
Cohesion: 0.06
Nodes (50): ack(), ack_local(), anchors_in(), check_ts(), DataError, BadRequest, InsufficientNodes, InvalidProof (+42 more)

### Community 10 - "ratchet.py"
Cohesion: 0.16
Nodes (15): 9. Otevřené body, 13.2 Ratchet (klient ↔ klient), b64u(), bootstrap_seed(), dh(), hkdf(), kdf_rk(), msg_key() (+7 more)

### Community 11 - "channel.rs"
Cohesion: 0.08
Nodes (32): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, ChannelError, BadPublicKey, Decrypt, Exhausted, Padding (+24 more)

### Community 12 - "ratchet.rs"
Cohesion: 0.12
Nodes (19): aad(), bootstrap_secret(), dh(), expand(), kdf_rk(), message_key(), MessageKey, next_chain_key() (+11 more)

### Community 13 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, exactOptionalPropertyTypes, isolatedModules, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+10 more)

### Community 14 - "registry.ts"
Cohesion: 0.11
Nodes (10): ErrorCode, Result, STATUS, toResult(), SignedDirectory, AuthCtx, BeginResult, ConfirmInput (+2 more)

### Community 15 - "utf8"
Cohesion: 0.18
Nodes (8): canonicalBytes(), Signer, b64uEncode(), issueTicket(), deletePush(), registerPush(), nowS(), TestNode

### Community 16 - "b64u_decode_array"
Cohesion: 0.24
Nodes (24): b64u_decode(), b64u_decode_array(), verify_ticket(), arr(), bob_profile_push_id(), canonical(), channel_client_node(), channel_node_node() (+16 more)

### Community 17 - "registration.ts"
Cohesion: 0.21
Nodes (21): genesisSet(), requiredBits(), timeoutSignal(), log(), activate(), assertAsnNotPenalized(), assertLimits(), banNode() (+13 more)

### Community 18 - "b64u_encode"
Cohesion: 0.17
Nodes (14): Event, Message, Typing, message_status(), next_event(), next_pushed(), receive(), Role (+6 more)

### Community 19 - "space.rs"
Cohesion: 0.06
Nodes (23): Backing, Mem, CHUNK, D0, DI, LEAF_BYTES, leaf_hash(), node_hash() (+15 more)

### Community 20 - "WorkerError"
Cohesion: 0.11
Nodes (9): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse, nonce_b64(), signed_header() (+1 more)

### Community 21 - "Ephemeral"
Cohesion: 0.23
Nodes (5): channel_roundtrip_detects_reorder_replay_and_tamper(), derive(), different_transcript_yields_different_keys(), Ephemeral, frame_counter_exhaustion_is_fatal()

### Community 22 - "NodeSocket"
Cohesion: 0.22
Nodes (11): Connection, lookup_profile(), next_text(), NodeSocket, RawSocket, read_frame(), rpc(), send_typing() (+3 more)

### Community 23 - "e2e.rs"
Cohesion: 0.06
Nodes (6): main(), PUSH_FILE, ADMIN, admin_nodes(), forward(), ZONE

### Community 24 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Co je potřeba před nasazením (fáze C), Konfigurace, Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 25 - "senders.ts"
Cohesion: 0.10
Nodes (26): aesGcmDecrypt(), aesGcmEncrypt(), PushJob, queue(), processPushBatch(), b64urJson(), CachedToken, createPushSender() (+18 more)

### Community 26 - "user.rs"
Cohesion: 0.07
Nodes (40): address_uri(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), both_sides_agree_and_the_one_time_prekey_is_consumed(), dh(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION (+32 more)

### Community 27 - "client.rs"
Cohesion: 0.11
Nodes (31): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+23 more)

### Community 28 - "register.rs"
Cohesion: 0.16
Nodes (9): hostname_matches(), activate(), register(), Registration, DOMAIN_REGISTER, run(), start(), Tunnel (+1 more)

### Community 29 - "NodeState"
Cohesion: 0.06
Nodes (35): ConnGuard, SubscriptionGuard, leading_zero_bits(), solve(), valid(), handoff_if_needed(), purge_expired(), run() (+27 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.06
Nodes (62): auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_PROFILE, DOMAIN_PROOF, DOMAIN_PUSH_REG, DOMAIN_REPORT, DOMAIN_REQ (+54 more)

### Community 31 - "Cmd"
Cohesion: 0.09
Nodes (22): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Online (+14 more)

### Community 32 - "ratchet.rs"
Cohesion: 0.13
Nodes (16): conversation_heals_after_a_state_leak(), forged_message_does_not_advance_state(), gap_limit(), garbage_is_rejected(), Header, HEADER_LEN, MAX_SKIP, message_of_the_previous_chain_arrives_after_a_dh_step() (+8 more)

### Community 33 - "store.rs"
Cohesion: 0.08
Nodes (30): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_profile(), HEADER_LEN (+22 more)

### Community 34 - "control.rs"
Cohesion: 0.12
Nodes (9): heartbeat(), KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run() (+1 more)

### Community 35 - "b64u_encode"
Cohesion: 0.13
Nodes (8): base32_encode(), HOSTNAME_ID_CHARS, hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected(), node_id_of()

### Community 36 - "online.rs"
Cohesion: 0.17
Nodes (11): ack_inbox(), now_s(), publish_profile(), MAX_SUBSCRIPTIONS, OnlineEvent, Contact, Message, Postponed (+3 more)

### Community 37 - "ancord-node/src/main.rs"
Cohesion: 0.23
Nodes (4): NodeError, main(), resolve_worker_key(), run()

### Community 38 - "handler.rs"
Cohesion: 0.22
Nodes (19): compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start(), answer(), bandwidth(), cpu() (+11 more)

### Community 39 - "pairwise.py"
Cohesion: 0.11
Nodes (22): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+14 more)

### Community 40 - "ancord node"
Cohesion: 0.20
Nodes (9): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, E2E test (B1 až B4), Konfigurace nodu, Příkazy (+1 more)

### Community 44 - "ws.ts"
Cohesion: 0.31
Nodes (9): broadcastDirectory(), publishDirectory(), CfInfo, handleHeartbeat(), handleMessage(), isInt(), num(), onConnect() (+1 more)

### Community 45 - "n2n.rs"
Cohesion: 0.17
Nodes (20): SessionKeys, call(), Established, Refused, Session, Exchange, exchange(), Done (+12 more)

### Community 46 - "peer.rs"
Cohesion: 0.15
Nodes (24): Json, ack(), anchored_mailbox(), ApiError, deliver(), dispatch(), exchange(), get() (+16 more)

### Community 47 - "three_nodes_register_challenge_route_and_admit"
Cohesion: 0.14
Nodes (12): address(), Prepared, transmit(), transmit_with_push(), eventually(), free_port(), NodeProc, Proc (+4 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (45): 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake`, 10. Push (+37 more)

### Community 49 - "ticket.rs"
Cohesion: 0.16
Nodes (14): DOMAIN_DIRECTORY, DOMAIN_TICKET, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody, TicketError, BadSignature (+6 more)

### Community 50 - "config.ts"
Cohesion: 0.21
Nodes (11): base, cache, ChallengeKind, DeepPartial, isObject(), loadConfig(), merge(), ProfileName (+3 more)

### Community 51 - "server.rs"
Cohesion: 0.19
Nodes (6): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, SKEW_S

### Community 52 - "wake.rs"
Cohesion: 0.33
Nodes (4): admit(), optional_push_id(), PUSH_ID_LEN, spawn()

### Community 53 - "snapshot.ts"
Cohesion: 0.11
Nodes (31): createCloudflareClient(), Config, Env, RegCtx, DirectoryBody, DirectoryNode, eligibleForDirectory(), ensureDirectory() (+23 more)

### Community 54 - "CliError"
Cohesion: 0.13
Nodes (11): CliError, Admission, AllFailed, Node, Route, Session, Step, Frame (+3 more)

### Community 56 - "encoding.ts"
Cohesion: 0.14
Nodes (24): blobKey(), BwDir, ensureBlob(), signBwToken(), verifyBwToken(), hmacSha256(), B64U_LOOKUP, b64uDecode() (+16 more)

### Community 57 - "context.ts"
Cohesion: 0.17
Nodes (5): ApiEnvelope, CloudflareClient, MANAGED_COMMENT, MockCall, TUNNEL_NAME_PREFIX

### Community 58 - "UserKeys"
Cohesion: 0.19
Nodes (6): Accepted, InboxBatch, process_inbox(), safety(), Pairwise, UserKeys

### Community 61 - "canonical_bytes"
Cohesion: 0.30
Nodes (7): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value()

### Community 64 - "super"
Cohesion: 0.47
Nodes (3): anchors(), deterministic_and_bounded(), weight()

### Community 65 - "Direction"
Cohesion: 0.40
Nodes (4): Direction, InitiatorToResponder, ResponderToInitiator, mailbox_id()

### Community 69 - "cpu.ts"
Cohesion: 0.36
Nodes (8): advance(), advanceNative(), computeCheckpoints(), CpuParams, CpuVerdict, laneStart(), verifyCpu(), u32le()

### Community 70 - "vectors.test.ts"
Cohesion: 0.07
Nodes (37): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+29 more)

### Community 71 - "ancord-cli/src/lib.rs"
Cohesion: 0.14
Nodes (14): admission(), admit(), admit_unchecked(), connect_any(), default_url(), fetch_route(), HistoryEntry, Received (+6 more)

## Knowledge Gaps
- **87 isolated node(s):** `ancord-cli`, `Route`, `Admission`, `Node`, `Session` (+82 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 502 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **8 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `push.test.ts`, `utf8`?**
  _High betweenness centrality (0.414) - this node is a cross-community bridge._
- **Are the 44 inferred relationships involving `unix_now()` (e.g. with `authenticate()` and `do_send()`) actually correct?**
  _`unix_now()` has 44 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ancord-cli`, `Route`, `Admission` to the rest of the system?**
  _87 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Melo v2 – architektura` be split into smaller, more focused modules?**
  _Cohesion score 0.07407407407407407 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `utf8` to `gen-vectors.ts`, `scheduler.ts`, `push.test.ts`, `index.ts`, `cpu.ts`, `vectors.test.ts`, `push_id`, `registration.ts`, `snapshot.ts`, `encoding.ts`, `senders.ts`?**
  _High betweenness centrality (0.273) - this node is a cross-community bridge._
- **Should `gen-vectors.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.13675213675213677 - nodes in this community are weakly interconnected._
- **Why does `mailbox_id()` connect `push_id` to `ancord node`, `ratchet.py`?**
  _High betweenness centrality (0.267) - this node is a cross-community bridge._