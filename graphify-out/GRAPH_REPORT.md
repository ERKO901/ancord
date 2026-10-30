# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 176 files · ~130,702 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 30 file(s) not represented in the graph (top: .toml 18, (none) 6, .xml 2)

## Summary
- 2760 nodes · 7429 edges · 124 communities (105 shown, 19 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 412 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `70aefa79`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- 13. Klient (Android a desktop)
- src-tauri/src/store.rs
- scheduler.ts
- push.test.ts
- index.ts
- ancord – protokol (verze 1)
- ratchet.rs
- Session
- ancord-cli/src/main.rs
- data.rs
- ratchet.py
- channel.rs
- package.json
- RealCloudflareClient
- Registry
- b64uDecodeLen
- vectors.rs
- registration.ts
- App.svelte
- space_bench.rs
- WorkerClient
- backend.ts
- ancord-core/src/net.rs
- tests/e2e.rs
- n2n.rs
- b64uEncode
- user.rs
- client.rs
- duration
- deploy.ts
- dataplane.rs
- Cmd
- src-tauri/src/net.rs
- ancord-node/src/store.rs
- control.rs
- TestNode
- pairwise.py
- NodeSocket
- handler.rs
- json
- ticket.rs
- VirtualList.svelte
- core.rs
- snapshot.ts
- NodeState
- peer.rs
- identity.rs
- push_id
- app.svelte.ts
- ws.ts
- DirStore
- contract.rs
- UiEvent
- .contact_add
- local-forward.py
- ws.ts
- src/e2e.rs
- Profile
- Šest nodů v Dockeru (C3)
- b64u_encode
- ancord-node/src/lib.rs
- server.rs
- two_clients_chat_live_offline_and_without_duplicates
- services
- Chat.svelte
- tauri.conf.json
- options
- NODE_NAME
- .with
- b64uEncode
- node1
- node2
- Onboarding.svelte
- node3
- Shared
- node4
- node5
- node6
- config.ts
- x-node
- sha256
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh
- compilerOptions
- Core
- canonical.rs
- super
- path
- ancord
- screens.test.ts
- RO
- space.ts
- register_all
- NoDisk
- ancord klient (Android a desktop)
- default.json
- build.rs
- Egress
- SeedRng
- ancord-client
- 12. Pokyny pro implementaci (pro Claude Code)
- ThemeChoice
- 5. Control plane (Cloudflare Worker, TypeScript)
- space.rs
- eventually
- WorkerClient
- ancord-node/src/main.rs
- reach.ts
- SpaceTable
- cloudflare.ts
- config.ts
- files.rs
- ancord node
- routing.ts
- ancord Worker
- CloudflareClient
- 4. Skrytí IP
- Backing

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 79 edges
3. `DataError` - 59 edges
4. `Core` - 57 edges
5. `Session` - 54 edges
6. `b64uEncode()` - 54 edges
7. `unix_now()` - 53 edges
8. `b64uDecodeLen()` - 45 edges
9. `HttpError` - 45 edges
10. `NodeSocket` - 44 edges

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
- 1-file cycle: `node/crates/ancord-cli/src/main.rs -> node/crates/ancord-cli/src/main.rs`
- 2-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 2-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 2-file cycle: `node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs`
- 2-file cycle: `client/src-tauri/src/core.rs -> client/src-tauri/src/net.rs -> client/src-tauri/src/core.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 3-file cycle: `node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 5-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 5-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`

## Communities (124 total, 19 thin omitted)

### Community 0 - "13. Klient (Android a desktop)"
Cohesion: 0.08
Nodes (27): 10. Profily dev/prod a bootstrap, 11. Fáze, 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.14 Milníky fáze D, 13.15 Mimo rozsah v1 klienta (+19 more)

### Community 1 - "src-tauri/src/store.rs"
Cohesion: 0.06
Nodes (50): FAST, FAST, auto(), AutoDeviceKey, CONTEXT, device_mode_reports_no_passphrase(), DeviceKey, Env (+42 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.13
Nodes (36): ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, combineBandwidth(), Dim, DIM_COL, dispatch() (+28 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (43): vitest, clock, randomBytes(), resetRouteCache(), pushDeps, allMeasured(), node(), open (+35 more)

### Community 4 - "index.ts"
Cohesion: 0.09
Nodes (61): verifyNodeRequest(), aesGcmDecrypt(), workerSigner(), PushJob, ErrorCode, errorResponse(), HttpError, STATUS (+53 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "Session"
Cohesion: 0.13
Nodes (15): StorePersist, deliver(), HistoryEntry, keepalive_after_real_message_only_and_not_shown(), keepalive_if_due(), now_s(), pair(), parse_address() (+7 more)

### Community 8 - "ancord-cli/src/main.rs"
Cohesion: 0.10
Nodes (4): Route, main(), open(), PUSH_FILE

### Community 9 - "data.rs"
Cohesion: 0.06
Nodes (71): profile_id(), unix_now(), ack(), ack_local(), anchors_in(), blocking(), check_ts(), DataError (+63 more)

### Community 10 - "ratchet.py"
Cohesion: 0.19
Nodes (16): b64u(), bootstrap_seed(), header_aad(), header_key0(), header_nonce(), hkdf(), kdf_rk(), msg_key() (+8 more)

### Community 11 - "channel.rs"
Cohesion: 0.06
Nodes (39): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, Channel, channel_roundtrip_detects_reorder_replay_and_tamper(), ChannelError, BadPublicKey, Decrypt (+31 more)

### Community 12 - "package.json"
Cohesion: 0.07
Nodes (34): dependencies, @tauri-apps/api@^2.12.4, devDependencies, jsdom@^30.1.3, svelte@^5.57.3, svelte-check@^4.7.7, @sveltejs/vite-plugin-svelte@^7.3.1, @tauri-apps/cli@^2.12.4 (+26 more)

### Community 13 - "RealCloudflareClient"
Cohesion: 0.17
Nodes (8): apply, cf(), Envelope, main(), isManagedHostname(), MANAGED_COMMENT, RealCloudflareClient, TUNNEL_NAME_PREFIX

### Community 14 - "Registry"
Cohesion: 0.13
Nodes (12): Result, toResult(), SignedDirectory, AuthCtx, BeginInput, BeginResult, ConfirmInput, ConfirmResult (+4 more)

### Community 15 - "b64uDecodeLen"
Cohesion: 0.22
Nodes (13): verifyKey(), verifySig(), b64uDecode(), b64uDecodeLen(), fromUtf8(), sortKeys(), TicketBody, TicketCheck (+5 more)

### Community 16 - "vectors.rs"
Cohesion: 0.28
Nodes (23): b64u_decode(), b64u_decode_array(), arr(), bob_profile_push_id(), canonical(), channel_client_node(), channel_node_node(), channel_padding() (+15 more)

### Community 17 - "registration.ts"
Cohesion: 0.27
Nodes (17): genesisSet(), requiredBits(), log(), activate(), banNode(), closeSockets(), consumeNonce(), decodePubkey() (+9 more)

### Community 18 - "App.svelte"
Cohesion: 0.10
Nodes (16): hasMain, ThemeChoice, { title, onback, hideBackOnWide = false, right }, Name, { name, size = 22 }, paths, confirmDelete, contact (+8 more)

### Community 19 - "space_bench.rs"
Cohesion: 0.19
Nodes (12): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+4 more)

### Community 20 - "WorkerClient"
Cohesion: 0.11
Nodes (8): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse, nonce_b64(), signed_header()

### Community 21 - "backend.ts"
Cohesion: 0.13
Nodes (21): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+13 more)

### Community 22 - "ancord-core/src/net.rs"
Cohesion: 0.14
Nodes (18): Accepted, admission(), admit(), admit_unchecked(), AllFailed, connect_any(), fetch_route(), INBOX_POW_BITS (+10 more)

### Community 23 - "tests/e2e.rs"
Cohesion: 0.07
Nodes (10): ADMIN, admin_nodes(), forward(), free_port(), NodeProc, Proc, spawn_node(), worker_dir() (+2 more)

### Community 24 - "n2n.rs"
Cohesion: 0.21
Nodes (17): call(), call_once(), Established, Refused, Session, Exchange, exchange(), Done (+9 more)

### Community 25 - "b64uEncode"
Cohesion: 0.11
Nodes (16): Env, AdmitOptions, AdmitResult, RateLimiter, b64urJson(), CachedToken, createPushSender(), LogPushSender (+8 more)

### Community 26 - "user.rs"
Cohesion: 0.07
Nodes (44): safety(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), both_sides_agree_and_the_one_time_prekey_is_consumed(), dh(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION (+36 more)

### Community 27 - "client.rs"
Cohesion: 0.09
Nodes (32): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+24 more)

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.05
Nodes (68): auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_HEARTBEAT, DOMAIN_INBOX_GUARD, DOMAIN_N2N_RESP, DOMAIN_PROOF, DOMAIN_PUSH_REG (+60 more)

### Community 31 - "Cmd"
Cohesion: 0.10
Nodes (20): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Publish (+12 more)

### Community 32 - "src-tauri/src/net.rs"
Cohesion: 0.09
Nodes (15): Cmd, Flush, Typing, Link, MAX_BACKOFF_S, NetConfig, PING_EVERY, run() (+7 more)

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.07
Nodes (39): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+31 more)

### Community 34 - "control.rs"
Cohesion: 0.09
Nodes (10): heartbeat(), KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run() (+2 more)

### Community 35 - "TestNode"
Cohesion: 0.14
Nodes (13): RFC-8785, canonicalBytes(), blobKey(), BwDir, ensureBlob(), signBwToken(), verifyBwToken(), hmacSha256() (+5 more)

### Community 36 - "pairwise.py"
Cohesion: 0.09
Nodes (12): ContactRec, conv_id(), IDENTITY_KEY, MAX_TEXT, msg_key(), Network, now_ms(), OutItem (+4 more)

### Community 37 - "NodeSocket"
Cohesion: 0.14
Nodes (29): run(), b64u_encode(), ack_inbox(), address(), Connection, PowRequired, Event, Message (+21 more)

### Community 38 - "handler.rs"
Cohesion: 0.30
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 39 - "json"
Cohesion: 0.10
Nodes (22): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+14 more)

### Community 40 - "ticket.rs"
Cohesion: 0.14
Nodes (15): DOMAIN_DIRECTORY, DOMAIN_TICKET, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody, TicketError, BadSignature (+7 more)

### Community 41 - "VirtualList.svelte"
Cohesion: 0.09
Nodes (14): HeightIndex, bottom, host, index, {
    items,
    key,
    row,
    estimate = 64,
    overscan = 600,
    onTop,
    label,
  }, keys, ro, schedule() (+6 more)

### Community 43 - "core.rs"
Cohesion: 0.11
Nodes (8): Guard, MAX_BITS, payload(), stamp(), STAMP_LEN, stamp_nonce(), stamp_valid(), TS_SKEW_S

### Community 44 - "snapshot.ts"
Cohesion: 0.12
Nodes (29): Json, DirectoryBody, DirectoryNode, eligibleForDirectory(), ensureDirectory(), epochOf(), readDirectory(), excludedIds() (+21 more)

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (29): ConnGuard, SubscriptionGuard, handoff_if_needed(), purge_expired(), run(), InSessions, add_subscriber(), after_store() (+21 more)

### Community 46 - "peer.rs"
Cohesion: 0.14
Nodes (27): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+19 more)

### Community 47 - "identity.rs"
Cohesion: 0.12
Nodes (6): HOSTNAME_ID_CHARS, hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected()

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (49): 13.4 Jádro: crate `ancord-core`, 13.8 Push (Android), 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu (+41 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.20
Nodes (22): addContact(), boot(), createIdentity(), deleteConversation(), enterReady(), go(), loadOlder(), lockNow() (+14 more)

### Community 50 - "ws.ts"
Cohesion: 0.17
Nodes (15): session_conv(), soft(), Task, CliError, Admission, Node, Route, Session (+7 more)

### Community 51 - "DirStore"
Cohesion: 0.15
Nodes (6): DirStore, label_of(), Step, Frame, Line, Quit

### Community 52 - "contract.rs"
Cohesion: 0.15
Nodes (14): Contact, ConversationPage, ConversationSummary, DeliveryState, Delivered, Pending, Queued, Unknown (+6 more)

### Community 53 - "UiEvent"
Cohesion: 0.14
Nodes (12): ConnectionState, Connecting, Offline, Online, UiEvent, ConnectionState, ContactRequest, DeliveryState (+4 more)

### Community 54 - ".contact_add"
Cohesion: 0.24
Nodes (10): contacts_reject_garbage_self_and_duplicates(), identity_contacts_and_settings_survive_a_restart(), memory_core_cannot_send(), offline_messages_are_queued_paged_and_survive_a_restart(), qr_is_an_svg(), ready(), ready_keys(), safety_number_is_the_same_on_both_sides() (+2 more)

### Community 56 - "ws.ts"
Cohesion: 0.17
Nodes (18): challengeTick(), ensureSchedule(), liveConnectedNodes(), needsCommit(), onPong(), slotsOf(), startNext(), startSlot() (+10 more)

### Community 57 - "src/e2e.rs"
Cohesion: 0.29
Nodes (6): NODES, online(), received(), state_of(), texts(), wait()

### Community 58 - "Profile"
Cohesion: 0.14
Nodes (13): ProofError, BadRequest, InvalidProof, is_for(), MAX_OTKS, parse_pk(), pow_nonce(), Profile (+5 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 61 - "ancord-node/src/lib.rs"
Cohesion: 0.20
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 62 - "server.rs"
Cohesion: 0.14
Nodes (7): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S

### Community 63 - "two_clients_chat_live_offline_and_without_duplicates"
Cohesion: 0.25
Nodes (5): free_port(), is_loopback_port(), LocalNet, repo(), two_clients_chat_live_offline_and_without_duplicates()

### Community 64 - "services"
Cohesion: 0.14
Nodes (13): name, services, volumes, node1-data, node2-data, node3-data, node4-data, node5-data (+5 more)

### Community 65 - "Chat.svelte"
Cohesion: 0.18
Nodes (16): app, contactName(), connLabel, initials(), shortAddress(), stateLabel, timeFull(), timeShort() (+8 more)

### Community 66 - "tauri.conf.json"
Cohesion: 0.11
Nodes (18): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, devUrl (+10 more)

### Community 67 - "options"
Cohesion: 0.29
Nodes (11): driver, options, logging, logging, logging, logging, logging, logging (+3 more)

### Community 68 - "NODE_NAME"
Cohesion: 0.53
Nodes (10): NODE_NAME, RUST_LOG, WORKER_PUBKEY, WORKER_URL, environment, environment, environment, environment (+2 more)

### Community 69 - ".with"
Cohesion: 0.16
Nodes (5): identity_is_created_once_and_address_has_the_scheme(), lock_err(), locked_core_exposes_nothing(), now(), passphrase_profile_stays_locked_until_the_right_passphrase()

### Community 70 - "b64uEncode"
Cohesion: 0.06
Nodes (58): keyOf(), main(), out, PKCS8, seedOf(), write(), NodeAuth, requestPayload() (+50 more)

### Community 71 - "node1"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 72 - "node2"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 73 - "Onboarding.svelte"
Cohesion: 0.19
Nodes (11): errText(), unlock(), busy, go(), pass, busy, create(), pass (+3 more)

### Community 74 - "node3"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 76 - "Shared"
Cohesion: 0.17
Nodes (4): Awaiting, Inner, Shared, update_message()

### Community 77 - "node4"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 78 - "node5"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 79 - "node6"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 81 - "x-node"
Cohesion: 0.22
Nodes (9): x-node, build, cap_drop, image, read_only, restart, security_opt, stop_grace_period (+1 more)

### Community 82 - "sha256"
Cohesion: 0.21
Nodes (8): leading_zero_bits(), solve(), valid(), compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start()

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

### Community 86 - "compilerOptions"
Cohesion: 0.15
Nodes (12): compilerOptions, module, moduleResolution, noUncheckedIndexedAccess, noUnusedLocals, skipLibCheck, strict, target (+4 more)

### Community 87 - "Core"
Cohesion: 0.12
Nodes (25): Core, address_get(), address_qr(), apply_protection(), AppSink, connection_state(), contact_add(), contact_list() (+17 more)

### Community 88 - "canonical.rs"
Cohesion: 0.30
Nodes (7): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value()

### Community 89 - "super"
Cohesion: 0.14
Nodes (5): base32_encode(), anchors(), deterministic_and_bounded(), weight(), PUSH_ID_LEN

### Community 90 - "path"
Cohesion: 0.27
Nodes (3): LoadMeter, measure(), Measured

### Community 91 - "ancord"
Cohesion: 0.25
Nodes (7): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura

### Community 92 - "screens.test.ts"
Cohesion: 0.43
Nodes (6): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start()

### Community 94 - "space.ts"
Cohesion: 0.18
Nodes (17): buildTree(), D0, decodeBytes(), decodePath(), depthOf(), DI, LEAF_BYTES, leafHash() (+9 more)

### Community 95 - "register_all"
Cohesion: 0.14
Nodes (13): Cli, Online, conversation_files(), load_push(), load_session(), now(), online_session(), OnlineCtx (+5 more)

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.29
Nodes (6): ancord klient (Android a desktop), Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14), Síť

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 106 - "12. Pokyny pro implementaci (pro Claude Code)"
Cohesion: 0.33
Nodes (6): 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code)

### Community 107 - "ThemeChoice"
Cohesion: 0.33
Nodes (5): Settings, ThemeChoice, Dark, Light, System

### Community 108 - "5. Control plane (Cloudflare Worker, TypeScript)"
Cohesion: 0.40
Nodes (5): 5.1 Identita a registrace nodu, 5.2 Důkazy zdrojů, 5.3 Load balancing a routing, 5.4 Push notifikace, 5. Control plane (Cloudflare Worker, TypeScript)

### Community 109 - "space.rs"
Cohesion: 0.15
Nodes (12): CHUNK, D0, DI, LEAF_BYTES, leaf_hash(), node_hash(), PAIR, ref_indices() (+4 more)

### Community 111 - "WorkerClient"
Cohesion: 0.22
Nodes (7): hostname_matches(), activate(), register(), Registration, run(), backoff(), WorkerClient

### Community 112 - "ancord-node/src/main.rs"
Cohesion: 0.22
Nodes (5): NodeError, main(), resolve_worker_key(), run(), shutdown_signal()

### Community 113 - "reach.ts"
Cohesion: 0.22
Nodes (9): Config, Signer, timeoutSignal(), RegCtx, Candidate, MAX_REPORTED, probe(), reachTick() (+1 more)

### Community 114 - "SpaceTable"
Cohesion: 0.24
Nodes (5): SpaceError, BadIndex, BadLeaves, Io, SpaceTable

### Community 115 - "cloudflare.ts"
Cohesion: 0.26
Nodes (5): ApiEnvelope, createCloudflareClient(), MockCall, MockCloudflareClient, sharedMockClient()

### Community 116 - "config.ts"
Cohesion: 0.22
Nodes (12): base, cache, ChallengeKind, DeepPartial, isObject(), loadConfig(), merge(), ProfileName (+4 more)

### Community 117 - "files.rs"
Cohesion: 0.24
Nodes (7): create_user(), load_session(), load_user(), lock_file(), save_session(), save_user(), write_private()

### Community 118 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

### Community 119 - "routing.ts"
Cohesion: 0.33
Nodes (8): ClientGeo, distanceKm(), haversineKm(), pickP2C(), selectNodes(), Snapshot, SnapshotNode, weightedIndex()

### Community 120 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 122 - "4. Skrytí IP"
Cohesion: 0.33
Nodes (6): 4. Skrytí IP, Důsledky, Node bez veřejné IP, Nody, Uživatelé, Závislost na Cloudflare a cena

### Community 123 - "Backing"
Cohesion: 0.40
Nodes (3): Backing, File, Mem

## Knowledge Gaps
- **118 isolated node(s):** `ancord-client`, `None`, `Locked`, `Ready`, `Queued` (+113 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 823 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **19 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `TestNode`, `push.test.ts`, `b64uEncode`?**
  _High betweenness centrality (0.259) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `NodeState` (e.g. with `OutSessions` and `Reach`) actually correct?**
  _`NodeState` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ancord-client`, `None`, `Locked` to the rest of the system?**
  _118 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `13. Klient (Android a desktop)` be split into smaller, more focused modules?**
  _Cohesion score 0.07936507936507936 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `5. Control plane (Cloudflare Worker, TypeScript)`, `ancord node`, `json`?**
  _High betweenness centrality (0.189) - this node is a cross-community bridge._
- **Should `src-tauri/src/store.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05745658835546476 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `scheduler.ts`, `TestNode`, `index.ts`, `push.test.ts`, `snapshot.ts`, `b64uDecodeLen`, `push_id`, `registration.ts`, `ws.ts`, `b64uEncode`, `space.ts`?**
  _High betweenness centrality (0.151) - this node is a cross-community bridge._