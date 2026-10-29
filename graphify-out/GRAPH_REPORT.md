# Graph Report - ancord  (2026-10-29)

## Corpus Check
- 126 files · ~98,275 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 7 file(s) not represented in the graph (top: (none) 5, .example 1, .jsonc 1)

## Summary
- 1887 nodes · 5461 edges · 79 communities (70 shown, 9 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 340 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `1ee7e5d3`
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
- SpaceTable
- WorkerError
- Ephemeral
- CliError
- e2e.rs
- ancord Worker
- senders.ts
- user.rs
- client.rs
- duration
- NodeState
- dataplane.rs
- Cmd
- digest
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
- presence.rs
- peer.rs
- handler.ts
- push_id
- pair
- config.ts
- HttpError
- wake.rs
- snapshot.ts
- Step
- local-forward.py
- encoding.ts
- registry.ts
- UserKeys
- dev
- local.sh
- canonical_bytes
- pairwise.py
- super
- establish
- errors.ts
- WorkerError
- sysinfo.rs
- cpu.ts
- vectors.test.ts
- ancord-cli/src/lib.rs
- config.rs
- http.ts
- space.rs
- routing.ts
- register.rs
- SpaceError
- UserError

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 135 edges
2. `CliError` - 68 edges
3. `unix_now()` - 60 edges
4. `DataError` - 57 edges
5. `b64uEncode()` - 54 edges
6. `b64u_encode()` - 53 edges
7. `Session` - 47 edges
8. `b64u_decode_array()` - 46 edges
9. `HttpError` - 45 edges
10. `b64uDecodeLen()` - 45 edges

## Surprising Connections (you probably didn't know these)
- `Cloudflare (firemní zóna melonet.cz)` --references--> `node()`  [INFERRED]
  CLAUDE.md → worker/test/challenge.test.ts
- `1.3 Podpisy a doménová separace` --references--> `Domain`  [INFERRED]
  spec/PROTOCOL.md → worker/src/crypto.ts
- `12.4 Pořadí prací a milníky` --references--> `CloudflareClient`  [INFERRED]
  ARCHITECTURE.md → worker/src/cloudflare.ts
- `12.3 Co musí dodat uživatel` --references--> `node()`  [INFERRED]
  ARCHITECTURE.md → worker/test/challenge.test.ts
- `Nody` --references--> `node()`  [INFERRED]
  ARCHITECTURE.md → worker/test/challenge.test.ts

## Import Cycles
- 1-file cycle: `node/crates/ancord-node/src/challenge/space.rs -> node/crates/ancord-node/src/challenge/space.rs`
- 2-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 2-file cycle: `node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs`
- 2-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/profile.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 3-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs`
- 3-file cycle: `node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/store.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/state.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/store.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/profile.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/profile.rs -> node/crates/ancord-node/src/data.rs`

## Communities (79 total, 9 thin omitted)

### Community 0 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (27): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+19 more)

### Community 1 - "gen-vectors.ts"
Cohesion: 0.15
Nodes (25): buildTable(), buildTree(), D0, decodeBytes(), decodePath(), depthOf(), DI, LEAF_BYTES (+17 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.12
Nodes (45): ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth(), Dim, DIM_COL (+37 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (44): vitest, ApiEnvelope, createCloudflareClient(), MANAGED_COMMENT, MockCall, sharedMockClient(), TUNNEL_NAME_PREFIX, Domain (+36 more)

### Community 4 - "index.ts"
Cohesion: 0.18
Nodes (31): clock, workerSigner(), clientInfo(), json(), parseJsonObject(), readBody(), reqString(), adminAllowed() (+23 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.08
Nodes (25): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+17 more)

### Community 6 - "profile.rs"
Cohesion: 0.18
Nodes (13): get(), get_local(), handoff(), is_for(), parse_pk(), pow_nonce(), Profile, profile_id() (+5 more)

### Community 7 - "run"
Cohesion: 0.09
Nodes (21): fetch_route(), load_user(), lock_file(), parse_address(), Cli, conversation_files(), DirStore, label_of() (+13 more)

### Community 8 - "package.json"
Cohesion: 0.09
Nodes (26): @cloudflare/vitest-plugin, @cloudflare/workers-types, @noble/hashes, tsx, typescript, wrangler, dependencies, @noble/hashes@^2.4.0 (+18 more)

### Community 9 - "data.rs"
Cohesion: 0.07
Nodes (41): ack(), ack_local(), anchors_in(), blocking(), DEFAULT_FETCH, fan_out(), fetch(), find_node() (+33 more)

### Community 10 - "ratchet.py"
Cohesion: 0.16
Nodes (15): 9. Otevřené body, 13.2 Ratchet (klient ↔ klient), b64u(), bootstrap_seed(), dh(), hkdf(), kdf_rk(), msg_key() (+7 more)

### Community 11 - "channel.rs"
Cohesion: 0.08
Nodes (32): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, ChannelError, BadPublicKey, Decrypt, Exhausted, Padding (+24 more)

### Community 12 - "ratchet.rs"
Cohesion: 0.08
Nodes (39): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, expand() (+31 more)

### Community 13 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, exactOptionalPropertyTypes, isolatedModules, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+10 more)

### Community 14 - "registry.ts"
Cohesion: 0.13
Nodes (7): Result, toResult(), AuthCtx, BeginResult, ConfirmInput, ConfirmResult, Registry

### Community 15 - "utf8"
Cohesion: 0.21
Nodes (7): canonicalBytes(), b64uEncode(), issueTicket(), deletePush(), registerPush(), nowS(), TestNode

### Community 16 - "b64u_decode_array"
Cohesion: 0.09
Nodes (39): b64u_decode(), b64u_decode_array(), DOMAIN_DIRECTORY, DOMAIN_TICKET, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody (+31 more)

### Community 17 - "registration.ts"
Cohesion: 0.20
Nodes (18): genesisSet(), randomBytes(), requiredBits(), timeoutSignal(), activate(), assertAsnNotPenalized(), assertLimits(), consumeNonce() (+10 more)

### Community 18 - "b64u_encode"
Cohesion: 0.14
Nodes (14): Event, Message, Typing, next_event(), next_pushed(), receive(), Role, Initiator (+6 more)

### Community 19 - "SpaceTable"
Cohesion: 0.18
Nodes (8): Backing, Mem, leaf_hash(), node_hash(), SpaceTable, t0(), ti(), tree_offsets()

### Community 20 - "WorkerError"
Cohesion: 0.15
Nodes (5): BeginResponse, ConfirmResponse, nonce_b64(), signed_header(), WorkerClient

### Community 21 - "Ephemeral"
Cohesion: 0.21
Nodes (7): channel_roundtrip_detects_reorder_replay_and_tamper(), derive(), different_transcript_yields_different_keys(), Ephemeral, frame_counter_exhaustion_is_fatal(), SessionKeys, OutSession

### Community 22 - "CliError"
Cohesion: 0.16
Nodes (21): ack_inbox(), CliError, Admission, AllFailed, Connection, Node, Route, Session (+13 more)

### Community 23 - "e2e.rs"
Cohesion: 0.05
Nodes (16): connect_any(), Route, RouteNode, open(), ADMIN, admin_nodes(), eventually(), forward() (+8 more)

### Community 24 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Co je potřeba před nasazením (fáze C), Konfigurace, Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 25 - "senders.ts"
Cohesion: 0.10
Nodes (26): aesGcmDecrypt(), aesGcmEncrypt(), pushIdOf(), processPushBatch(), b64urJson(), CachedToken, createPushSender(), LogPushSender (+18 more)

### Community 26 - "user.rs"
Cohesion: 0.09
Nodes (25): MAX_OTKS, address_uri(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), dh(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION, expand() (+17 more)

### Community 27 - "client.rs"
Cohesion: 0.10
Nodes (35): attach_rid(), authenticate(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED, do_ack(), do_fetch() (+27 more)

### Community 29 - "NodeState"
Cohesion: 0.09
Nodes (10): ConnGuard, SubscriptionGuard, Identity, handoff_if_needed(), purge_expired(), run(), InSessions, Subscribers (+2 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.05
Nodes (77): call(), Established, Refused, Session, Exchange, exchange(), Done, UnknownSession (+69 more)

### Community 31 - "Cmd"
Cohesion: 0.09
Nodes (22): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Online (+14 more)

### Community 32 - "digest"
Cohesion: 0.26
Nodes (8): compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start(), leading_zero_bits(), solve(), valid()

### Community 33 - "store.rs"
Cohesion: 0.08
Nodes (30): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_profile(), HEADER_LEN (+22 more)

### Community 34 - "control.rs"
Cohesion: 0.13
Nodes (8): KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run(), DOMAIN_HEARTBEAT

### Community 35 - "b64u_encode"
Cohesion: 0.15
Nodes (7): base32_encode(), HOSTNAME_ID_CHARS, hostname_of(), KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected(), node_id_of()

### Community 36 - "online.rs"
Cohesion: 0.22
Nodes (10): now_s(), publish_profile(), MAX_SUBSCRIPTIONS, Online, OnlineEvent, Contact, Message, Postponed (+2 more)

### Community 37 - "ancord-node/src/main.rs"
Cohesion: 0.18
Nodes (7): NodeError, main(), resolve_worker_key(), run(), activate(), run(), backoff()

### Community 38 - "handler.rs"
Cohesion: 0.33
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 39 - "pairwise.py"
Cohesion: 0.19
Nodes (14): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+6 more)

### Community 40 - "ancord node"
Cohesion: 0.20
Nodes (9): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, E2E test (B1 až B4), Konfigurace nodu, Příkazy (+1 more)

### Community 44 - "ws.ts"
Cohesion: 0.22
Nodes (17): Json, log(), broadcastDirectory(), DirectoryBody, DirectoryNode, eligibleForDirectory(), ensureDirectory(), epochOf() (+9 more)

### Community 45 - "presence.rs"
Cohesion: 0.12
Nodes (24): peer_post(), add_subscriber(), after_store(), announce(), coalesce(), current_presence(), deliver_local(), deliver_online() (+16 more)

### Community 46 - "peer.rs"
Cohesion: 0.09
Nodes (33): Json, ack(), anchored_mailbox(), ApiError, deliver(), dispatch(), exchange(), get() (+25 more)

### Community 47 - "handler.ts"
Cohesion: 0.15
Nodes (14): NodeAuth, Env, PushJob, AdmitOptions, AdmitResult, RateLimiter, pushDeps, readDirectory() (+6 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (45): 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake`, 10. Push (+37 more)

### Community 49 - "pair"
Cohesion: 0.19
Nodes (11): both_sides_agree_and_the_one_time_prekey_is_consumed(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), keys_survive_a_roundtrip_through_the_file(), pair(), profile_checks_time_and_proof_of_work(), push_ids_are_stable_and_bound_to_the_mailbox(), rotation_deletes_old_prekeys_so_old_handshakes_cannot_be_opened_later(), safety_number_is_symmetric_and_pair_specific() (+3 more)

### Community 50 - "config.ts"
Cohesion: 0.22
Nodes (12): base, cache, ChallengeKind, DeepPartial, isObject(), loadConfig(), merge(), ProfileName (+4 more)

### Community 51 - "HttpError"
Cohesion: 0.19
Nodes (4): isManagedHostname(), MockCloudflareClient, RealCloudflareClient, HttpError

### Community 52 - "wake.rs"
Cohesion: 0.33
Nodes (4): admit(), optional_push_id(), PUSH_ID_LEN, spawn()

### Community 53 - "snapshot.ts"
Cohesion: 0.15
Nodes (21): SignedDirectory, BeginInput, capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf() (+13 more)

### Community 54 - "Step"
Cohesion: 0.40
Nodes (4): Step, Frame, Line, Quit

### Community 56 - "encoding.ts"
Cohesion: 0.17
Nodes (18): blobKey(), BwDir, ensureBlob(), signBwToken(), verifyBwToken(), hmacSha256(), B64U_LOOKUP, b64uDecode() (+10 more)

### Community 57 - "registry.ts"
Cohesion: 0.15
Nodes (7): CloudflareClient, Config, Signer, RegCtx, unbanNode(), NodeSummary, CfInfo

### Community 58 - "UserKeys"
Cohesion: 0.21
Nodes (7): address(), create_user(), safety(), save_user(), write_private(), mailbox_push_key(), UserKeys

### Community 61 - "canonical_bytes"
Cohesion: 0.24
Nodes (8): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value(), heartbeat()

### Community 63 - "pairwise.py"
Cohesion: 0.18
Nodes (8): b64u(), hkdf(), mailbox(), profile(), raw(), sha(), x3dh(), xpub()

### Community 64 - "super"
Cohesion: 0.47
Nodes (3): anchors(), deterministic_and_bounded(), weight()

### Community 65 - "establish"
Cohesion: 0.27
Nodes (10): Bucket, close(), establish(), Incoming, Closed, Invalid, recv_plain(), reject() (+2 more)

### Community 66 - "errors.ts"
Cohesion: 0.14
Nodes (12): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura, ErrorCode (+4 more)

### Community 67 - "WorkerError"
Cohesion: 0.18
Nodes (4): WorkerError, Api, Protocol, Transport

### Community 68 - "sysinfo.rs"
Cohesion: 0.27
Nodes (3): LoadMeter, measure(), Measured

### Community 69 - "cpu.ts"
Cohesion: 0.33
Nodes (9): advance(), advanceNative(), computeCheckpoints(), CpuParams, CpuVerdict, laneStart(), verifyCpu(), randomInt() (+1 more)

### Community 70 - "vectors.test.ts"
Cohesion: 0.07
Nodes (38): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+30 more)

### Community 71 - "ancord-cli/src/lib.rs"
Cohesion: 0.13
Nodes (12): Accepted, admission(), admit(), admit_unchecked(), default_url(), HistoryEntry, InboxBatch, next_text() (+4 more)

### Community 73 - "http.ts"
Cohesion: 0.27
Nodes (9): expandIpv6(), ipPrefix(), MAX_BODY_BYTES, optInt(), reqB64(), reqInt(), reqObject(), requireMethod() (+1 more)

### Community 74 - "space.rs"
Cohesion: 0.22
Nodes (5): CHUNK, D0, DI, LEAF_BYTES, PAIR

### Community 75 - "routing.ts"
Cohesion: 0.32
Nodes (7): ClientGeo, distanceKm(), haversineKm(), pickP2C(), Snapshot, SnapshotNode, weightedIndex()

### Community 76 - "register.rs"
Cohesion: 0.43
Nodes (4): hostname_matches(), register(), Registration, DOMAIN_REGISTER

### Community 77 - "SpaceError"
Cohesion: 0.33
Nodes (5): ref_index(), SpaceError, BadIndex, BadLeaves, Io

### Community 78 - "UserError"
Cohesion: 0.29
Nodes (7): UserError, BadEnvelope, BadProfile, PrekeyGone, SelfConversation, StaleProfile, WeakKey

## Knowledge Gaps
- **87 isolated node(s):** `BadPublicKey`, `Decrypt`, `Exhausted`, `Padding`, `TooLarge` (+82 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 503 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **9 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `senders.ts`, `utf8`?**
  _High betweenness centrality (0.417) - this node is a cross-community bridge._
- **Are the 44 inferred relationships involving `unix_now()` (e.g. with `authenticate()` and `do_send()`) actually correct?**
  _`unix_now()` has 44 INFERRED edges - model-reasoned connections that need verification._
- **What connects `BadPublicKey`, `Decrypt`, `Exhausted` to the rest of the system?**
  _87 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Melo v2 – architektura` be split into smaller, more focused modules?**
  _Cohesion score 0.07142857142857142 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `utf8` to `gen-vectors.ts`, `scheduler.ts`, `push.test.ts`, `index.ts`, `cpu.ts`, `vectors.test.ts`, `ws.ts`, `push_id`, `registration.ts`, `snapshot.ts`, `encoding.ts`, `senders.ts`?**
  _High betweenness centrality (0.290) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.12244897959183673 - nodes in this community are weakly interconnected._
- **Why does `mailbox_id()` connect `push_id` to `ancord node`, `ratchet.py`?**
  _High betweenness centrality (0.283) - this node is a cross-community bridge._