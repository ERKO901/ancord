# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 188 files · ~149,861 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 47 file(s) not represented in the graph (top: .toml 18, .xml 11, (none) 9)

## Summary
- 2905 nodes · 7607 edges · 127 communities (100 shown, 27 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 416 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f835a933`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- StoreError
- scheduler.ts
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
- .save
- DeviceKey
- backend.ts
- CliError
- tests/e2e.rs
- core.rs
- push/store.ts
- user.rs
- client.rs
- WorkerClient
- deploy.ts
- dataplane.rs
- Cmd
- Table
- ancord-node/src/store.rs
- b64u_encode
- b64uEncode
- UiEvent
- ancord-core/src/net.rs
- WorkerError
- contract.rs
- sign.rs
- VirtualList.svelte
- ancord-core/src/guard.rs
- b64u_decode_array
- NodeState
- peer.rs
- n2n.rs
- push_id
- app.svelte.ts
- handler.rs
- android_keystore.rs
- space.rs
- android_sync.rs
- RealPushSender
- local-forward.py
- 13. Klient (Android a desktop)
- src/e2e.rs
- ProofError
- Šest nodů v Dockeru (C3)
- unix_now
- .with
- server.rs
- SyncReceiver.kt
- services
- Chat.svelte
- tauri.conf.json
- options
- NODE_NAME
- .lock
- vectors.test.ts
- node1
- node2
- pairwise.py
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
- Session
- ancord klient (Android a desktop)
- default.json
- build.rs
- identity.rs
- ancord-core/src/lib.rs
- ancord-client
- Melo v2 – architektura
- ancord-node/src/main.rs
- MainActivity
- ConnectionState
- routing.ts
- Settings.svelte
- sha256
- ScanActivity
- LocalNet
- Sink
- ancord Worker
- ConnectionIssue
- Egress
- update_message
- super
- Step
- Outcome

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 92 edges
3. `Session` - 64 edges
4. `DataError` - 59 edges
5. `Core` - 57 edges
6. `b64uEncode()` - 54 edges
7. `unix_now()` - 53 edges
8. `NodeSocket` - 49 edges
9. `UserKeys` - 49 edges
10. `b64uDecodeLen()` - 45 edges

## Surprising Connections (you probably didn't know these)
- `5.4 Push notifikace` --references--> `push_id()`  [INFERRED]
  ARCHITECTURE.md → spec/reference/pairwise.py
- `12.4 Pořadí prací a milníky` --references--> `CloudflareClient`  [INFERRED]
  ARCHITECTURE.md → worker/src/cloudflare.ts
- `1.3 Podpisy a doménová separace` --references--> `Domain`  [INFERRED]
  spec/PROTOCOL.md → worker/src/crypto.ts
- `graphify` --references--> `json()`  [INFERRED]
  CLAUDE.md → worker/src/http.ts
- `Nody` --references--> `node()`  [INFERRED]
  ARCHITECTURE.md → worker/test/challenge.test.ts

## Import Cycles
- 1-file cycle: `node/crates/ancord-node/src/challenge/space.rs -> node/crates/ancord-node/src/challenge/space.rs`
- 2-file cycle: `node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs`
- 2-file cycle: `client/src-tauri/src/core.rs -> client/src-tauri/src/net.rs -> client/src-tauri/src/core.rs`

## Communities (127 total, 27 thin omitted)

### Community 1 - "StoreError"
Cohesion: 0.15
Nodes (12): io(), KeychainDeviceKey, read_wrap(), Store, StoreError, Corrupt, Io, Passphrase (+4 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.05
Nodes (116): ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth(), Dim, DIM_COL (+108 more)

### Community 3 - "push.test.ts"
Cohesion: 0.07
Nodes (63): vitest, ApiEnvelope, createCloudflareClient(), MockCall, sharedMockClient(), TUNNEL_NAME_PREFIX, base, cache (+55 more)

### Community 4 - "index.ts"
Cohesion: 0.11
Nodes (53): NodeAuth, verifyNodeRequest(), clock, workerSigner(), b64uDecodeLen(), ErrorCode, errorResponse(), HttpError (+45 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "keepalive_if_due"
Cohesion: 0.24
Nodes (6): deliver(), keepalive_if_due(), Prepared, transmit(), transmit_with_push(), push_id_of()

### Community 8 - "BuildTask.kt"
Cohesion: 0.13
Nodes (3): BuildTask, Config, RustPlugin

### Community 9 - "data.rs"
Cohesion: 0.07
Nodes (50): ack(), ack_local(), anchors_in(), blocking(), check_ts(), DataError, BadRequest, InsufficientNodes (+42 more)

### Community 10 - "ratchet.py"
Cohesion: 0.17
Nodes (18): 13.2 Ratchet (klient ↔ klient), b64u(), bootstrap_seed(), dh(), header_aad(), header_key0(), header_nonce(), hkdf() (+10 more)

### Community 11 - "channel.rs"
Cohesion: 0.06
Nodes (37): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, channel_roundtrip_detects_reorder_replay_and_tamper(), ChannelError, BadPublicKey, Decrypt, Exhausted (+29 more)

### Community 12 - "package.json"
Cohesion: 0.05
Nodes (34): dependencies, @tauri-apps/api, devDependencies, jsdom, svelte, svelte-check, @sveltejs/vite-plugin-svelte, @tauri-apps/cli (+26 more)

### Community 13 - "CloudflareClient"
Cohesion: 0.09
Nodes (9): apply, cf(), Envelope, main(), CloudflareClient, isManagedHostname(), MANAGED_COMMENT, MockCloudflareClient (+1 more)

### Community 14 - "Registry"
Cohesion: 0.12
Nodes (8): Result, toResult(), AuthCtx, BeginResult, ConfirmInput, ConfirmResult, Registry, migrate()

### Community 15 - "channel.py"
Cohesion: 0.15
Nodes (15): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+7 more)

### Community 16 - "vectors.rs"
Cohesion: 0.06
Nodes (46): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value(), b64u_decode() (+38 more)

### Community 17 - "path"
Cohesion: 0.16
Nodes (8): Settings, ThemeChoice, Dark, Light, System, LoadMeter, measure(), Measured

### Community 18 - "App.svelte"
Cohesion: 0.09
Nodes (21): hasMain, app, errText(), address, busy, name, scan(), busy (+13 more)

### Community 19 - ".save"
Cohesion: 0.27
Nodes (3): session_conv(), StorePersist, Task

### Community 20 - "DeviceKey"
Cohesion: 0.22
Nodes (9): auto(), AutoDeviceKey, DeviceKey, Env, Fake, FileDeviceKey, foreign_device_key_does_not_open(), keychain_is_preferred_and_file_is_not_created() (+1 more)

### Community 21 - "backend.ts"
Cohesion: 0.13
Nodes (19): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+11 more)

### Community 22 - "CliError"
Cohesion: 0.10
Nodes (33): b64u_encode(), ack_inbox(), CliError, Admission, AllFailed, Connection, Node, PowRequired (+25 more)

### Community 23 - "tests/e2e.rs"
Cohesion: 0.04
Nodes (19): classify(), MAX_BACKOFF_S, PING_EVERY, STATUS_EVERY, SYNC_LIMIT, main(), PUSH_FILE, MAX_SUBSCRIPTIONS (+11 more)

### Community 24 - "core.rs"
Cohesion: 0.08
Nodes (11): ContactRec, identity_is_created_once_and_address_has_the_scheme(), IDENTITY_KEY, MAX_TEXT, msg_key(), now(), now_ms(), OutItem (+3 more)

### Community 25 - "push/store.ts"
Cohesion: 0.10
Nodes (27): aesGcmDecrypt(), aesGcmEncrypt(), Env, PushJob, queue(), AdmitOptions, AdmitResult, RateLimiter (+19 more)

### Community 26 - "user.rs"
Cohesion: 0.07
Nodes (43): safety(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), both_sides_agree_and_the_one_time_prekey_is_consumed(), dh(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION (+35 more)

### Community 27 - "client.rs"
Cohesion: 0.11
Nodes (32): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+24 more)

### Community 28 - "WorkerClient"
Cohesion: 0.17
Nodes (7): hostname_matches(), activate(), register(), Registration, run(), backoff(), WorkerClient

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.09
Nodes (49): anchors_of(), anchors_send_signed_wake_only_for_new_messages_with_push_id(), call_peer(), caller_rejects_responder_that_cannot_prove_identity(), cluster(), cluster_with_worker(), delivery_status_follows_the_message_lifecycle(), directory() (+41 more)

### Community 31 - "Cmd"
Cohesion: 0.09
Nodes (22): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Online (+14 more)

### Community 32 - "Table"
Cohesion: 0.22
Nodes (7): prefix_end(), Table, Contacts, Conversations, Identity, Messages, Settings

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.06
Nodes (38): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+30 more)

### Community 34 - "b64u_encode"
Cohesion: 0.12
Nodes (4): KEEPALIVE, on_directory(), admit(), spawn()

### Community 35 - "b64uEncode"
Cohesion: 0.11
Nodes (21): canonicalBytes(), blobKey(), BwDir, ensureBlob(), signBwToken(), verifyBwToken(), hmacSha256(), b64uDecode() (+13 more)

### Community 36 - "UiEvent"
Cohesion: 0.18
Nodes (8): UiEvent, ConnectionIssue, ConnectionState, ContactRequest, DeliveryState, MessageReceived, Typing, ChanSink

### Community 37 - "ancord-core/src/net.rs"
Cohesion: 0.06
Nodes (35): sync_once(), open(), Channel, Accepted, address(), admission(), admit(), admit_unchecked() (+27 more)

### Community 38 - "WorkerError"
Cohesion: 0.10
Nodes (8): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse, nonce_b64(), signed_header()

### Community 39 - "contract.rs"
Cohesion: 0.16
Nodes (14): Contact, ConversationPage, ConversationSummary, DeliveryState, Delivered, Pending, Queued, Unknown (+6 more)

### Community 40 - "sign.rs"
Cohesion: 0.10
Nodes (21): auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_DIRECTORY, DOMAIN_HEARTBEAT, DOMAIN_INBOX_GUARD, DOMAIN_N2N_RESP, DOMAIN_PROFILE (+13 more)

### Community 41 - "VirtualList.svelte"
Cohesion: 0.09
Nodes (15): HeightIndex, bottom, host, index, {
    items,
    key,
    row,
    estimate = 64,
    overscan = 600,
    onTop,
    label,
  }, keys, measure(), ro (+7 more)

### Community 43 - "ancord-core/src/guard.rs"
Cohesion: 0.19
Nodes (7): MAX_BITS, payload(), stamp(), STAMP_LEN, stamp_nonce(), stamp_valid(), TS_SKEW_S

### Community 44 - "b64u_decode_array"
Cohesion: 0.22
Nodes (4): MAX_OTKS, pow_nonce(), signed_payload(), TS_SKEW_S

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (29): ConnGuard, SubscriptionGuard, handoff_if_needed(), purge_expired(), run(), add_subscriber(), after_store(), announce() (+21 more)

### Community 46 - "peer.rs"
Cohesion: 0.12
Nodes (28): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+20 more)

### Community 47 - "n2n.rs"
Cohesion: 0.25
Nodes (15): SessionKeys, call(), call_once(), Established, Refused, Session, Exchange, Done (+7 more)

### Community 48 - "push_id"
Cohesion: 0.06
Nodes (55): 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), ancord node, Bezpečnostní poznámky, CLI klient: běžné použití (+47 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.24
Nodes (20): addContact(), boot(), createIdentity(), deleteConversation(), enterReady(), go(), loadOlder(), lockNow() (+12 more)

### Community 50 - "handler.rs"
Cohesion: 0.25
Nodes (13): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, space_commit() (+5 more)

### Community 51 - "android_keystore.rs"
Cohesion: 0.18
Nodes (5): AndroidKeystoreDeviceKey, Bridge, call(), io(), JNI_OnLoad()

### Community 52 - "space.rs"
Cohesion: 0.07
Nodes (31): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+23 more)

### Community 53 - "android_sync.rs"
Cohesion: 0.27
Nodes (3): Java_cz_melonet_ancord_SyncReceiver_nativeSync(), LogSink, run()

### Community 54 - "RealPushSender"
Cohesion: 0.40
Nodes (3): b64urJson(), pemToDer(), RealPushSender

### Community 56 - "13. Klient (Android a desktop)"
Cohesion: 0.12
Nodes (17): 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.15 Mimo rozsah v1 klienta, 13.1 Cíle a pořadí priorit, 13.2 Volba technologie, 13.3 Vrstvy a hranice důvěry (+9 more)

### Community 57 - "src/e2e.rs"
Cohesion: 0.22
Nodes (11): client(), free_port(), is_loopback_port(), NODES, online(), received(), repo(), state_of() (+3 more)

### Community 58 - "ProofError"
Cohesion: 0.19
Nodes (8): ProofError, BadRequest, InvalidProof, is_for(), parse_pk(), optional_push_id(), parse_push_id(), PUSH_ID_LEN

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 60 - "unix_now"
Cohesion: 0.09
Nodes (22): local.sh script, Guard, Profile, profile_id(), unix_now(), heartbeat(), from_record(), handoff() (+14 more)

### Community 61 - ".with"
Cohesion: 0.18
Nodes (11): contacts_reject_garbage_self_and_duplicates(), identity_contacts_and_settings_survive_a_restart(), memory_core_cannot_send(), offline_messages_are_queued_paged_and_survive_a_restart(), qr_is_an_svg(), ready(), ready_keys(), safety_number_is_the_same_on_both_sides() (+3 more)

### Community 62 - "server.rs"
Cohesion: 0.15
Nodes (7): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S

### Community 64 - "services"
Cohesion: 0.14
Nodes (13): name, services, volumes, node1-data, node2-data, node3-data, node4-data, node5-data (+5 more)

### Community 65 - "Chat.svelte"
Cohesion: 0.13
Nodes (18): contactName(), connLabel, initials(), issueLabel, shortAddress(), stateLabel, timeFull(), timeShort() (+10 more)

### Community 66 - "tauri.conf.json"
Cohesion: 0.10
Nodes (20): minSdkVersion, app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand (+12 more)

### Community 67 - "options"
Cohesion: 0.29
Nodes (11): driver, options, logging, logging, logging, logging, logging, logging (+3 more)

### Community 68 - "NODE_NAME"
Cohesion: 0.53
Nodes (10): NODE_NAME, RUST_LOG, WORKER_PUBKEY, WORKER_URL, environment, environment, environment, environment (+2 more)

### Community 69 - ".lock"
Cohesion: 0.16
Nodes (5): conv_id(), lock_err(), locked_core_exposes_nothing(), passphrase_profile_stays_locked_until_the_right_passphrase(), hex_encode()

### Community 70 - "vectors.test.ts"
Cohesion: 0.06
Nodes (68): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+60 more)

### Community 71 - "node1"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 72 - "node2"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 73 - "pairwise.py"
Cohesion: 0.18
Nodes (9): 13.4 Jádro: crate `ancord-core`, hkdf(), mailbox(), profile(), raw(), sha(), solve(), x3dh() (+1 more)

### Community 74 - "node3"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 75 - "reach.rs"
Cohesion: 0.18
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 76 - "src-tauri/src/store.rs"
Cohesion: 0.10
Nodes (13): FAST, FAST, CONTEXT, FAST, KdfParams, META, MODE_DEVICE, MODE_PASSPHRASE (+5 more)

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
Cohesion: 0.27
Nodes (9): 13.14 Milníky fáze D, connText(), ConnectionIssue, area, onInput(), onKey(), setTyping(), submit() (+1 more)

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

### Community 86 - "compilerOptions"
Cohesion: 0.15
Nodes (12): compilerOptions, module, moduleResolution, noUncheckedIndexedAccess, noUnusedLocals, skipLibCheck, strict, target (+4 more)

### Community 87 - "Core"
Cohesion: 0.12
Nodes (25): Core, address_get(), address_qr(), apply_protection(), AppSink, connection_state(), contact_add(), contact_list() (+17 more)

### Community 89 - ".create"
Cohesion: 0.39
Nodes (6): device_mode_reports_no_passphrase(), passphrase_is_required_and_checked(), plaintext_is_not_in_the_file(), records_survive_a_restart(), scan_and_delete_prefix_are_scoped(), tampered_or_moved_record_is_rejected()

### Community 91 - "ancord"
Cohesion: 0.25
Nodes (7): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura

### Community 92 - "screens.test.ts"
Cohesion: 0.36
Nodes (7): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start(), @tauri-apps/api

### Community 95 - "Session"
Cohesion: 0.10
Nodes (24): create_user(), load_session(), load_user(), lock_file(), save_session(), save_user(), write_private(), Cli (+16 more)

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.29
Nodes (6): ancord klient (Android a desktop), Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14), Síť

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 100 - "identity.rs"
Cohesion: 0.12
Nodes (7): node_id_of(), HOSTNAME_ID_CHARS, hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected()

### Community 106 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (26): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+18 more)

### Community 109 - "ancord-node/src/main.rs"
Cohesion: 0.27
Nodes (5): NodeError, main(), resolve_worker_key(), run(), shutdown_signal()

### Community 112 - "ConnectionState"
Cohesion: 0.50
Nodes (4): ConnectionState, Connecting, Offline, Online

### Community 113 - "routing.ts"
Cohesion: 0.33
Nodes (8): ClientGeo, distanceKm(), haversineKm(), pickP2C(), selectNodes(), Snapshot, SnapshotNode, weightedIndex()

### Community 114 - "Settings.svelte"
Cohesion: 0.50
Nodes (3): ThemeChoice, locks, themes

### Community 115 - "sha256"
Cohesion: 0.23
Nodes (8): leading_zero_bits(), solve(), valid(), compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start()

### Community 118 - "Sink"
Cohesion: 0.15
Nodes (9): Cmd, Flush, Typing, Link, NetConfig, run(), set_state(), Sink (+1 more)

### Community 120 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 121 - "ConnectionIssue"
Cohesion: 0.29
Nodes (7): ConnectionIssue, NodeRejected, NodeUnreachable, ProtocolMismatch, RouteUntrusted, WorkerError, WorkerUnreachable

### Community 124 - "super"
Cohesion: 0.47
Nodes (3): anchors(), deterministic_and_bounded(), weight()

### Community 125 - "Step"
Cohesion: 0.40
Nodes (4): Step, Frame, Line, Quit

### Community 126 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Banned, Reconnect, Reregister, run()

## Knowledge Gaps
- **133 isolated node(s):** `@tauri-apps/cli`, `@tsconfig/svelte`, `@types/node`, `jsdom`, `svelte-check` (+128 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 920 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **27 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `13. Klient (Android a desktop)`, `pairwise.py`, `push.test.ts`, `b64uEncode`?**
  _High betweenness centrality (0.214) - this node is a cross-community bridge._
- **What connects `@tauri-apps/cli`, `@tsconfig/svelte`, `@types/node` to the rest of the system?**
  _133 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `ScanActivity.kt` be split into smaller, more focused modules?**
  _Cohesion score 0.13333333333333333 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `pairwise.py`, `Melo v2 – architektura`, `ratchet.py`, `channel.py`, `13. Klient (Android a desktop)`?**
  _High betweenness centrality (0.170) - this node is a cross-community bridge._
- **Should `StoreError` be split into smaller, more focused modules?**
  _Cohesion score 0.14532019704433496 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `scheduler.ts`, `push.test.ts`, `index.ts`, `vectors.test.ts`, `push_id`, `RealPushSender`, `push/store.ts`?**
  _High betweenness centrality (0.148) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.053620784964068545 - nodes in this community are weakly interconnected._