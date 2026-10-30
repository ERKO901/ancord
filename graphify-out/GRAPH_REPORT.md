# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 189 files · ~135,631 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 48 file(s) not represented in the graph (top: .toml 19, .xml 11, (none) 9)

## Summary
- 2901 nodes · 7643 edges · 132 communities (105 shown, 27 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 415 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d675df3f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- ScanActivity.kt
- StoreError
- registration.ts
- push.test.ts
- index.ts
- ancord – protokol (verze 1)
- ratchet.rs
- keepalive_if_due
- BuildTask.kt
- data.rs
- ratchet.py
- channel.rs
- package.json
- CloudflareClient
- Registry
- channel.py
- vectors.rs
- path
- App.svelte
- CliError
- AutoDeviceKey
- backend.ts
- CliError
- tests/e2e.rs
- core.rs
- push/store.ts
- .initiate_with
- client.rs
- WorkerClient
- deploy.ts
- dataplane.rs
- Cmd
- Table
- ancord-node/src/store.rs
- b64u_encode
- b64uEncode
- ancord-core/src/net.rs
- WorkerError
- UiEvent
- sign.rs
- VirtualList.svelte
- b64u_decode_array
- config.ts
- NodeState
- peer.rs
- n2n.rs
- push_id
- app.svelte.ts
- handler.rs
- android_keystore.rs
- space.rs
- android_sync.rs
- NodeSocket
- local-forward.py
- 13. Klient (Android a desktop)
- src/e2e.rs
- ProofError
- Šest nodů v Dockeru (C3)
- local.sh
- .with
- server.rs
- SyncReceiver.kt
- services
- Chat.svelte
- tauri.conf.json
- options
- NODE_NAME
- vectors.test.ts
- node1
- node2
- scheduler.ts
- node3
- reach.rs
- src-tauri/src/store.rs
- node4
- node5
- node6
- pathbuf
- x-node
- Composer.svelte
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh
- compilerOptions
- Core
- Shared
- .create
- KeystoreKey.kt
- ancord
- screens.test.ts
- RO
- register_all
- ancord klient (Android a desktop)
- default.json
- build.rs
- identity.rs
- ancord-core/src/lib.rs
- ancord-client
- Melo v2 – architektura
- ancord-node/src/main.rs
- MainActivity
- UserKeys
- ancord-cli/src/main.rs
- sha256
- ws.ts
- ancord Worker
- snapshot.ts
- super
- Step
- ticket.rs
- ticket.ts
- pair
- space_bench.rs
- src-tauri/src/net.rs
- user.rs
- canonical.rs
- ancord node
- scripts
- NoDisk
- MockCloudflareClient
- SeedRng

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 80 edges
3. `DataError` - 59 edges
4. `Core` - 57 edges
5. `b64uEncode()` - 54 edges
6. `Session` - 54 edges
7. `unix_now()` - 53 edges
8. `HttpError` - 45 edges
9. `b64uDecodeLen()` - 45 edges
10. `NodeSocket` - 44 edges

## Surprising Connections (you probably didn't know these)
- `Nody` --references--> `node()`  [INFERRED]
  ARCHITECTURE.md → worker/test/challenge.test.ts
- `5.4 Push notifikace` --references--> `push_id()`  [INFERRED]
  ARCHITECTURE.md → spec/reference/pairwise.py
- `12.4 Pořadí prací a milníky` --references--> `CloudflareClient`  [INFERRED]
  ARCHITECTURE.md → worker/src/cloudflare.ts
- `1.3 Podpisy a doménová separace` --references--> `Domain`  [INFERRED]
  spec/PROTOCOL.md → worker/src/crypto.ts
- `Cloudflare (firemní zóna melonet.cz)` --references--> `node()`  [INFERRED]
  CLAUDE.md → worker/test/challenge.test.ts

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
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 4-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/data.rs`
- 5-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`
- 5-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/n2n.rs -> node/crates/ancord-node/src/peer.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/data.rs`

## Communities (132 total, 27 thin omitted)

### Community 1 - "StoreError"
Cohesion: 0.15
Nodes (14): DeviceKey, Fake, io(), KeychainDeviceKey, read_wrap(), StoreError, Corrupt, Io (+6 more)

### Community 2 - "registration.ts"
Cohesion: 0.16
Nodes (32): bwAuthorize(), bwRecord, challengeTick(), combineBandwidth(), finish(), finishWitness(), liveConnectedNodes(), onResponse() (+24 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (48): vitest, sharedMockClient(), loadConfig(), validate(), withOverrides(), Domain, fromHex(), resetRouteCache() (+40 more)

### Community 4 - "index.ts"
Cohesion: 0.10
Nodes (54): verifyNodeRequest(), clock, workerSigner(), ErrorCode, errorResponse(), HttpError, STATUS, unwrap() (+46 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "keepalive_if_due"
Cohesion: 0.12
Nodes (19): load_session(), deliver(), HistoryEntry, keepalive_after_real_message_only_and_not_shown(), keepalive_if_due(), now_s(), pair(), parse_address() (+11 more)

### Community 8 - "BuildTask.kt"
Cohesion: 0.13
Nodes (3): BuildTask, Config, RustPlugin

### Community 9 - "data.rs"
Cohesion: 0.07
Nodes (51): profile_id(), ack(), ack_local(), anchors_in(), blocking(), DEFAULT_FETCH, fan_out(), fetch() (+43 more)

### Community 10 - "ratchet.py"
Cohesion: 0.19
Nodes (16): b64u(), bootstrap_seed(), header_aad(), header_key0(), header_nonce(), hkdf(), kdf_rk(), msg_key() (+8 more)

### Community 11 - "channel.rs"
Cohesion: 0.07
Nodes (39): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, Channel, channel_roundtrip_detects_reorder_replay_and_tamper(), ChannelError, BadPublicKey, Decrypt (+31 more)

### Community 12 - "package.json"
Cohesion: 0.09
Nodes (27): dependencies, @tauri-apps/api@^2.12.4, devDependencies, jsdom@^30.1.3, svelte@^5.57.3, svelte-check@^4.7.7, @sveltejs/vite-plugin-svelte@^7.3.1, @tauri-apps/cli@^2.12.4 (+19 more)

### Community 13 - "CloudflareClient"
Cohesion: 0.11
Nodes (11): apply, cf(), Envelope, main(), ApiEnvelope, CloudflareClient, isManagedHostname(), MANAGED_COMMENT (+3 more)

### Community 14 - "Registry"
Cohesion: 0.12
Nodes (12): createCloudflareClient(), Result, toResult(), SignedDirectory, AuthCtx, BeginInput, BeginResult, ConfirmInput (+4 more)

### Community 15 - "channel.py"
Cohesion: 0.10
Nodes (22): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+14 more)

### Community 16 - "vectors.rs"
Cohesion: 0.28
Nodes (23): b64u_decode(), b64u_decode_array(), arr(), bob_profile_push_id(), canonical(), channel_client_node(), channel_node_node(), channel_padding() (+15 more)

### Community 17 - "path"
Cohesion: 0.27
Nodes (3): LoadMeter, measure(), Measured

### Community 18 - "App.svelte"
Cohesion: 0.10
Nodes (19): hasMain, errText(), add(), address, busy, name, busy, go() (+11 more)

### Community 19 - "CliError"
Cohesion: 0.19
Nodes (10): save_contact_rec(), session_conv(), StorePersist, update_message(), Task, CliError, Admission, Node (+2 more)

### Community 21 - "backend.ts"
Cohesion: 0.11
Nodes (20): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+12 more)

### Community 22 - "CliError"
Cohesion: 0.24
Nodes (8): b64u_encode(), Online, OnlineEvent, Contact, Message, Postponed, Typing, Persist

### Community 23 - "tests/e2e.rs"
Cohesion: 0.06
Nodes (11): ADMIN, admin_nodes(), eventually(), forward(), free_port(), NodeProc, Proc, spawn_node() (+3 more)

### Community 24 - "core.rs"
Cohesion: 0.06
Nodes (20): ContactRec, FAST, identity_is_created_once_and_address_has_the_scheme(), IDENTITY_KEY, MAX_TEXT, msg_key(), Network, now() (+12 more)

### Community 25 - "push/store.ts"
Cohesion: 0.08
Nodes (33): NodeAuth, aesGcmDecrypt(), aesGcmEncrypt(), base32Encode(), PushJob, queue(), AdmitOptions, AdmitResult (+25 more)

### Community 26 - ".initiate_with"
Cohesion: 0.13
Nodes (18): addresses_parse_with_or_without_the_scheme_and_reject_garbage(), dh(), envelope_key(), FirstRatchetKey, Own, Peer, Initiation, Opened (+10 more)

### Community 27 - "client.rs"
Cohesion: 0.06
Nodes (49): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+41 more)

### Community 28 - "WorkerClient"
Cohesion: 0.18
Nodes (7): hostname_matches(), register(), Registration, run(), start(), Tunnel, backoff()

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.07
Nodes (55): unix_now(), Identity, key_persists_with_private_mode(), loose_permissions_are_rejected(), activate(), anchors_of(), anchors_send_signed_wake_only_for_new_messages_with_push_id(), call_peer() (+47 more)

### Community 31 - "Cmd"
Cohesion: 0.10
Nodes (20): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Publish (+12 more)

### Community 32 - "Table"
Cohesion: 0.22
Nodes (8): Record, Store, Table, Contacts, Conversations, Identity, Messages, Settings

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.07
Nodes (39): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+31 more)

### Community 34 - "b64u_encode"
Cohesion: 0.12
Nodes (8): heartbeat(), KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run()

### Community 35 - "b64uEncode"
Cohesion: 0.13
Nodes (17): canonicalBytes(), b64uDecodeLen(), b64uEncode(), randomBytes(), requiredBits(), activate(), decodePubkey(), registerBegin() (+9 more)

### Community 37 - "ancord-core/src/net.rs"
Cohesion: 0.13
Nodes (20): sync_once(), Accepted, admission(), admit(), admit_unchecked(), AllFailed, connect_any(), default_url() (+12 more)

### Community 38 - "WorkerError"
Cohesion: 0.12
Nodes (9): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse, nonce_b64(), signed_header() (+1 more)

### Community 39 - "UiEvent"
Cohesion: 0.06
Nodes (37): ConnectionIssue, NodeRejected, NodeUnreachable, ProtocolMismatch, RouteUntrusted, WorkerError, WorkerUnreachable, ConnectionState (+29 more)

### Community 40 - "sign.rs"
Cohesion: 0.12
Nodes (17): auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_HEARTBEAT, DOMAIN_N2N_RESP, DOMAIN_PROOF, DOMAIN_PUSH_REG, DOMAIN_REGISTER (+9 more)

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

### Community 43 - "b64u_decode_array"
Cohesion: 0.11
Nodes (9): Guard, MAX_BITS, payload(), stamp(), STAMP_LEN, stamp_nonce(), stamp_valid(), TS_SKEW_S (+1 more)

### Community 44 - "config.ts"
Cohesion: 0.09
Nodes (22): base, cache, ChallengeKind, Config, DeepPartial, isObject(), merge(), ProfileName (+14 more)

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (30): is_member(), handoff_if_needed(), purge_expired(), run(), InSessions, add_subscriber(), after_store(), announce() (+22 more)

### Community 46 - "peer.rs"
Cohesion: 0.13
Nodes (29): Json, MAX_ACK_IDS, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch() (+21 more)

### Community 47 - "n2n.rs"
Cohesion: 0.21
Nodes (17): call(), call_once(), Established, Refused, Session, Exchange, exchange(), Done (+9 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (46): 13.4 Jádro: crate `ancord-core`, 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake` (+38 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.20
Nodes (22): addContact(), boot(), createIdentity(), deleteConversation(), enterReady(), go(), loadOlder(), lockNow() (+14 more)

### Community 50 - "handler.rs"
Cohesion: 0.30
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 51 - "android_keystore.rs"
Cohesion: 0.15
Nodes (6): AndroidKeystoreDeviceKey, Bridge, BRIDGE, call(), io(), JNI_OnLoad()

### Community 52 - "space.rs"
Cohesion: 0.10
Nodes (20): Backing, File, Mem, CHUNK, D0, DI, LEAF_BYTES, leaf_hash() (+12 more)

### Community 53 - "android_sync.rs"
Cohesion: 0.36
Nodes (3): Java_cz_melonet_ancord_SyncReceiver_nativeSync(), LogSink, run()

### Community 54 - "NodeSocket"
Cohesion: 0.18
Nodes (17): Connection, PowRequired, Event, Message, Typing, message_status(), next_event(), next_pushed() (+9 more)

### Community 56 - "13. Klient (Android a desktop)"
Cohesion: 0.11
Nodes (20): 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.15 Mimo rozsah v1 klienta, 13.1 Cíle a pořadí priorit, 13.2 Volba technologie, 13.3 Vrstvy a hranice důvěry (+12 more)

### Community 57 - "src/e2e.rs"
Cohesion: 0.12
Nodes (15): ChanSink, client(), Egress, free_port(), is_loopback_port(), LocalNet, NODES, online() (+7 more)

### Community 58 - "ProofError"
Cohesion: 0.12
Nodes (14): ProofError, BadRequest, InvalidProof, is_for(), MAX_OTKS, parse_pk(), pow_nonce(), Profile (+6 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 61 - ".with"
Cohesion: 0.10
Nodes (15): contacts_reject_garbage_self_and_duplicates(), conv_id(), identity_contacts_and_settings_survive_a_restart(), Inner, locked_core_exposes_nothing(), memory_core_cannot_send(), offline_messages_are_queued_paged_and_survive_a_restart(), passphrase_profile_stays_locked_until_the_right_passphrase() (+7 more)

### Community 62 - "server.rs"
Cohesion: 0.13
Nodes (8): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, SKEW_S, admit(), spawn()

### Community 64 - "services"
Cohesion: 0.14
Nodes (13): name, services, volumes, node1-data, node2-data, node3-data, node4-data, node5-data (+5 more)

### Community 65 - "Chat.svelte"
Cohesion: 0.12
Nodes (20): app, contactName(), connLabel, connText(), initials(), issueLabel, shortAddress(), stateLabel (+12 more)

### Community 66 - "tauri.conf.json"
Cohesion: 0.10
Nodes (20): minSdkVersion, app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand (+12 more)

### Community 67 - "options"
Cohesion: 0.29
Nodes (11): driver, options, logging, logging, logging, logging, logging, logging (+3 more)

### Community 68 - "NODE_NAME"
Cohesion: 0.53
Nodes (10): NODE_NAME, RUST_LOG, WORKER_PUBKEY, WORKER_URL, environment, environment, environment, environment (+2 more)

### Community 70 - "vectors.test.ts"
Cohesion: 0.06
Nodes (60): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+52 more)

### Community 71 - "node1"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 72 - "node2"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 73 - "scheduler.ts"
Cohesion: 0.13
Nodes (27): blobKey(), ensureBlob(), ALLOWED_COLUMNS, ChallengeRow, Dim, DIM_COL, dispatch(), dispatchWitness() (+19 more)

### Community 74 - "node3"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 75 - "reach.rs"
Cohesion: 0.17
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 76 - "src-tauri/src/store.rs"
Cohesion: 0.12
Nodes (10): CONTEXT, META, MODE_DEVICE, MODE_PASSPHRASE, NONCE_LEN, prefix_end(), SALT_LEN, WRAP_HEADER (+2 more)

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

### Community 82 - "Composer.svelte"
Cohesion: 0.43
Nodes (6): area, onInput(), onKey(), setTyping(), submit(), text

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

### Community 86 - "compilerOptions"
Cohesion: 0.15
Nodes (12): compilerOptions, module, moduleResolution, noUncheckedIndexedAccess, noUnusedLocals, skipLibCheck, strict, target (+4 more)

### Community 87 - "Core"
Cohesion: 0.12
Nodes (25): Core, address_get(), address_qr(), apply_protection(), AppSink, connection_state(), contact_add(), contact_list() (+17 more)

### Community 88 - "Shared"
Cohesion: 0.13
Nodes (8): Awaiting, lock_err(), Shared, NetConfig, run(), set_state(), Sink, start()

### Community 89 - ".create"
Cohesion: 0.27
Nodes (13): auto(), device_mode_reports_no_passphrase(), Env, env(), FileDeviceKey, foreign_device_key_does_not_open(), keychain_is_preferred_and_file_is_not_created(), passphrase_is_required_and_checked() (+5 more)

### Community 91 - "ancord"
Cohesion: 0.25
Nodes (7): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura

### Community 92 - "screens.test.ts"
Cohesion: 0.43
Nodes (6): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start()

### Community 95 - "register_all"
Cohesion: 0.11
Nodes (11): Online, DirStore, label_of(), load_push(), now(), online_session(), OnlineCtx, PushConfig (+3 more)

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.29
Nodes (6): ancord klient (Android a desktop), Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14), Síť

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 100 - "identity.rs"
Cohesion: 0.25
Nodes (3): HOSTNAME_ID_CHARS, hostname_of(), KEY_FILE

### Community 106 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (26): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+18 more)

### Community 109 - "ancord-node/src/main.rs"
Cohesion: 0.20
Nodes (5): NodeError, main(), resolve_worker_key(), run(), shutdown_signal()

### Community 110 - "MainActivity"
Cohesion: 0.27
Nodes (3): 13.14 Milníky fáze D, MainActivity, ScanBridge

### Community 113 - "UserKeys"
Cohesion: 0.14
Nodes (20): create_user(), load_user(), lock_file(), save_session(), save_user(), write_private(), Cli, conversation_files() (+12 more)

### Community 114 - "ancord-cli/src/main.rs"
Cohesion: 0.09
Nodes (4): Route, main(), open(), PUSH_FILE

### Community 115 - "sha256"
Cohesion: 0.23
Nodes (8): leading_zero_bits(), solve(), valid(), compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start()

### Community 119 - "ws.ts"
Cohesion: 0.16
Nodes (21): Json, ensureSchedule(), onPong(), slotsOf(), broadcastDirectory(), DirectoryBody, DirectoryNode, eligibleForDirectory() (+13 more)

### Community 120 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 123 - "snapshot.ts"
Cohesion: 0.17
Nodes (19): excludedIds(), capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf(), trustOf() (+11 more)

### Community 124 - "super"
Cohesion: 0.18
Nodes (4): base32_encode(), anchors(), deterministic_and_bounded(), weight()

### Community 125 - "Step"
Cohesion: 0.40
Nodes (4): Step, Frame, Line, Quit

### Community 126 - "ticket.rs"
Cohesion: 0.14
Nodes (15): DOMAIN_DIRECTORY, DOMAIN_TICKET, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody, TicketError, BadSignature (+7 more)

### Community 127 - "ticket.ts"
Cohesion: 0.16
Nodes (19): BwDir, signBwToken(), verifyBwToken(), decodeBytes(), decodePath(), depthOf(), leafHash(), verifyOpenings() (+11 more)

### Community 128 - "pair"
Cohesion: 0.19
Nodes (11): both_sides_agree_and_the_one_time_prekey_is_consumed(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), keys_survive_a_roundtrip_through_the_file(), pair(), profile_checks_time_and_proof_of_work(), push_ids_are_stable_and_bound_to_the_mailbox(), rotation_deletes_old_prekeys_so_old_handshakes_cannot_be_opened_later(), safety_number_is_symmetric_and_pair_specific() (+3 more)

### Community 129 - "space_bench.rs"
Cohesion: 0.21
Nodes (12): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+4 more)

### Community 130 - "src-tauri/src/net.rs"
Cohesion: 0.15
Nodes (5): classify(), MAX_BACKOFF_S, PING_EVERY, STATUS_EVERY, SYNC_LIMIT

### Community 131 - "user.rs"
Cohesion: 0.16
Nodes (11): ENVELOPE_HEAD, ENVELOPE_VERSION, expand(), NOW, parse_address(), PROFILE_MAX_AGE_S, SPK_KEEP_S, SPK_ROTATE_S (+3 more)

### Community 132 - "canonical.rs"
Cohesion: 0.30
Nodes (7): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value()

### Community 133 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

### Community 134 - "scripts"
Cohesion: 0.22
Nodes (9): scripts, budget, build, check, dev, preview, tauri, test (+1 more)

## Knowledge Gaps
- **125 isolated node(s):** `Corrupt`, `Passphrase`, `Unlock`, `ancord-client`, `BadPublicKey` (+120 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 880 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **27 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `13. Klient (Android a desktop)`, `b64uEncode`, `push.test.ts`?**
  _High betweenness centrality (0.254) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `NodeState` (e.g. with `OutSessions` and `Reach`) actually correct?**
  _`NodeState` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Corrupt`, `Passphrase`, `Unlock` to the rest of the system?**
  _125 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `ScanActivity.kt` be split into smaller, more focused modules?**
  _Cohesion score 0.11052631578947368 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `ancord node`, `Melo v2 – architektura`, `ratchet.py`, `channel.py`, `13. Klient (Android a desktop)`?**
  _High betweenness centrality (0.196) - this node is a cross-community bridge._
- **Should `StoreError` be split into smaller, more focused modules?**
  _Cohesion score 0.1476923076923077 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `registration.ts`, `push.test.ts`, `index.ts`, `vectors.test.ts`, `scheduler.ts`, `push_id`, `ws.ts`, `push/store.ts`, `snapshot.ts`, `ticket.ts`?**
  _High betweenness centrality (0.171) - this node is a cross-community bridge._