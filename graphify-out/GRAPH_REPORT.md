# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 196 files · ~144,873 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 60 file(s) not represented in the graph (top: .toml 29, .xml 13, (none) 9)

## Summary
- 3078 nodes · 8100 edges · 144 communities (107 shown, 37 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 432 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c83b42b6`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- DeviceKey
- registry.ts
- push.test.ts
- index.ts
- ancord – protokol (verze 1)
- ratchet.rs
- sign.rs
- BuildTask.kt
- data.rs
- ratchet.py
- channel.rs
- package.json
- cloudflare.ts
- Registry
- run
- vectors.rs
- CliError
- App.svelte
- ContactDetail.svelte
- 13. Klient (Android a desktop)
- backend.ts
- ancord-core/src/net.rs
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
- .with
- ws.ts
- WorkerError
- contract.rs
- Table
- VirtualList.svelte
- b64u_decode_array
- .create
- NodeState
- peer.rs
- n2n.rs
- push_id
- app.svelte.ts
- handler.rs
- android_keystore.rs
- space.rs
- android_sync.rs
- app
- local-forward.py
- Melo v2 – architektura
- src/e2e.rs
- ProofError
- Šest nodů v Dockeru (C3)
- local.sh
- .unwrap
- server.rs
- SyncReceiver.kt
- services
- Chat.svelte
- tauri.conf.json
- options
- NODE_NAME
- Shared
- vectors.test.ts
- node1
- node2
- scheduler.ts
- node3
- pairwise.py
- src-tauri/src/store.rs
- node4
- node5
- node6
- Link
- x-node
- ancord-cli/src/main.rs
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh
- compilerOptions
- Core
- ticket.rs
- biometric_unlocks_only_a_passphrase_profile_and_password_still_works
- BiometricKey.kt
- ancord
- screens.test.ts
- RO
- MockCloudflareClient
- ancord klient (Android a desktop)
- default.json
- build.rs
- UiEvent
- ancord-core/src/lib.rs
- ancord-client
- Outcome
- ancord-node/src/main.rs
- MainActivity
- reach.rs
- Settings.svelte
- sha256
- ScanActivity
- message.rs
- registry.ts
- reach.ts
- duration
- scoring.ts
- canonical.rs
- identity.rs
- registration.ts
- ConnectionIssue
- super
- src-tauri/src/net.rs
- routing.ts
- ancord node
- scripts
- Egress
- SeedRng
- ThemeChoice

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 81 edges
3. `Core` - 79 edges
4. `DataError` - 59 edges
5. `b64uEncode()` - 54 edges
6. `Session` - 54 edges
7. `unix_now()` - 53 edges
8. `HttpError` - 45 edges
9. `b64uDecodeLen()` - 45 edges
10. `NodeSocket` - 44 edges

## Surprising Connections (you probably didn't know these)
- `12.4 Pořadí prací a milníky` --references--> `CloudflareClient`  [INFERRED]
  ARCHITECTURE.md → worker/src/cloudflare.ts
- `Nody` --references--> `node()`  [INFERRED]
  ARCHITECTURE.md → worker/test/challenge.test.ts
- `Cloudflare (firemní zóna melonet.cz)` --references--> `node()`  [INFERRED]
  CLAUDE.md → worker/test/challenge.test.ts
- `graphify` --references--> `json()`  [INFERRED]
  CLAUDE.md → worker/src/http.ts
- `5.4 Push notifikace` --references--> `push_id()`  [INFERRED]
  ARCHITECTURE.md → spec/reference/pairwise.py

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

## Communities (144 total, 37 thin omitted)

### Community 1 - "DeviceKey"
Cohesion: 0.33
Nodes (6): auto(), AutoDeviceKey, DeviceKey, Env, Fake, FileDeviceKey

### Community 2 - "registry.ts"
Cohesion: 0.14
Nodes (8): CloudflareClient, Config, Signer, RegCtx, unbanNode(), NodeSummary, migrate(), CfInfo

### Community 3 - "push.test.ts"
Cohesion: 0.08
Nodes (54): vitest, sharedMockClient(), base, cache, DeepPartial, isObject(), loadConfig(), merge() (+46 more)

### Community 4 - "index.ts"
Cohesion: 0.11
Nodes (53): NodeAuth, verifyNodeRequest(), clock, workerSigner(), b64uDecodeLen(), ErrorCode, errorResponse(), HttpError (+45 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "sign.rs"
Cohesion: 0.12
Nodes (17): auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_HEARTBEAT, DOMAIN_N2N_RESP, DOMAIN_PROOF, DOMAIN_PUSH_REG, DOMAIN_REGISTER (+9 more)

### Community 8 - "BuildTask.kt"
Cohesion: 0.13
Nodes (3): BuildTask, Config, RustPlugin

### Community 9 - "data.rs"
Cohesion: 0.07
Nodes (53): profile_id(), ack(), ack_local(), anchors_in(), blocking(), DEFAULT_FETCH, fan_out(), fetch() (+45 more)

### Community 10 - "ratchet.py"
Cohesion: 0.17
Nodes (18): 9. Otevřené body, b64u(), bootstrap_seed(), dh(), header_aad(), header_key0(), header_nonce(), hkdf() (+10 more)

### Community 11 - "channel.rs"
Cohesion: 0.06
Nodes (38): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, channel_roundtrip_detects_reorder_replay_and_tamper(), ChannelError, BadPublicKey, Decrypt, Exhausted (+30 more)

### Community 12 - "package.json"
Cohesion: 0.09
Nodes (26): dependencies, @tauri-apps/api@^2.12.4, devDependencies, jsdom@^30.1.3, svelte@^5.57.3, svelte-check@^4.7.7, @sveltejs/vite-plugin-svelte@^7.3.1, @tauri-apps/cli@^2.12.4 (+18 more)

### Community 13 - "cloudflare.ts"
Cohesion: 0.14
Nodes (11): apply, cf(), Envelope, main(), ApiEnvelope, createCloudflareClient(), isManagedHostname(), MANAGED_COMMENT (+3 more)

### Community 14 - "Registry"
Cohesion: 0.14
Nodes (8): Result, toResult(), AuthCtx, BeginResult, closeSockets(), ConfirmInput, ConfirmResult, Registry

### Community 15 - "run"
Cohesion: 0.25
Nodes (19): bwAuthorize(), bwRecord, challengeTick(), combineBandwidth(), dropChallengesOf(), finish(), finishWitness(), onPong() (+11 more)

### Community 16 - "vectors.rs"
Cohesion: 0.23
Nodes (24): b64u_decode(), b64u_decode_array(), arr(), bob_profile_push_id(), canonical(), channel_client_node(), channel_node_node(), channel_padding() (+16 more)

### Community 17 - "CliError"
Cohesion: 0.09
Nodes (47): StorePersist, update_message(), b64u_encode(), ack_inbox(), CliError, Admission, Connection, Node (+39 more)

### Community 18 - "App.svelte"
Cohesion: 0.09
Nodes (16): version, { title, subtitle, onback, hideBackOnWide = false, large = false, lead, right }, Name, { name, size = 22 }, paths, points, address, busy (+8 more)

### Community 19 - "ContactDetail.svelte"
Cohesion: 0.08
Nodes (28): hasMain, top, errText(), isTopLevel(), unlock(), { size = 40, tile = true }, confirmDelete, contact (+20 more)

### Community 20 - "13. Klient (Android a desktop)"
Cohesion: 0.13
Nodes (16): 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.15 Mimo rozsah v1 klienta, 13.1 Cíle a pořadí priorit, 13.2 Volba technologie, 13.3 Vrstvy a hranice důvěry (+8 more)

### Community 21 - "backend.ts"
Cohesion: 0.11
Nodes (21): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+13 more)

### Community 22 - "ancord-core/src/net.rs"
Cohesion: 0.09
Nodes (26): sync_once(), Channel, Accepted, admission(), admit(), admit_unchecked(), AllFailed, connect_any() (+18 more)

### Community 23 - "tests/e2e.rs"
Cohesion: 0.05
Nodes (12): ADMIN, admin_nodes(), eventually(), forward(), free_port(), next_online_message(), NodeProc, NoDisk (+4 more)

### Community 24 - "core.rs"
Cohesion: 0.09
Nodes (15): BIO_FILE, ContactRec, conv_id(), IDENTITY_KEY, MAX_TEXT, msg_key(), NAME_KEY, Network (+7 more)

### Community 25 - "push/store.ts"
Cohesion: 0.07
Nodes (34): aesGcmDecrypt(), aesGcmEncrypt(), Env, PushJob, queue(), AdmitOptions, AdmitResult, RateLimiter (+26 more)

### Community 26 - ".initiate_with"
Cohesion: 0.06
Nodes (46): address(), safety(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), both_sides_agree_and_the_one_time_prekey_is_consumed(), dh(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), ENVELOPE_HEAD, envelope_key() (+38 more)

### Community 27 - "client.rs"
Cohesion: 0.06
Nodes (48): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+40 more)

### Community 28 - "WorkerClient"
Cohesion: 0.22
Nodes (7): hostname_matches(), activate(), register(), Registration, run(), backoff(), WorkerClient

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.07
Nodes (52): Identity, key_persists_with_private_mode(), loose_permissions_are_rejected(), anchors_of(), anchors_send_signed_wake_only_for_new_messages_with_push_id(), call_peer(), caller_rejects_responder_that_cannot_prove_identity(), cluster() (+44 more)

### Community 31 - "Cmd"
Cohesion: 0.10
Nodes (20): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Publish (+12 more)

### Community 32 - "Table"
Cohesion: 0.14
Nodes (11): io(), KeychainDeviceKey, read_wrap(), Store, StoreError, Corrupt, Io, Passphrase (+3 more)

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.07
Nodes (39): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+31 more)

### Community 34 - "b64u_encode"
Cohesion: 0.12
Nodes (5): heartbeat(), KEEPALIVE, on_directory(), admit(), spawn()

### Community 35 - "b64uEncode"
Cohesion: 0.18
Nodes (8): canonicalBytes(), b64uEncode(), issueTicket(), deletePush(), pushKey(), registerPush(), nowS(), TestNode

### Community 36 - ".with"
Cohesion: 0.12
Nodes (4): ready_keys(), save_contact_rec(), short_key(), clean_name()

### Community 37 - "ws.ts"
Cohesion: 0.25
Nodes (13): ensureSchedule(), slotsOf(), startNext(), broadcastDirectory(), parseUnreachable(), recordReports(), publishDirectory(), handleHeartbeat() (+5 more)

### Community 38 - "WorkerError"
Cohesion: 0.12
Nodes (8): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse, nonce_b64(), signed_header()

### Community 39 - "contract.rs"
Cohesion: 0.11
Nodes (19): BiometricStatus, ConnectionState, Connecting, Offline, Online, Contact, ConversationPage, ConversationSummary (+11 more)

### Community 40 - "Table"
Cohesion: 0.18
Nodes (8): prefix_end(), Record, Table, Contacts, Conversations, Identity, Messages, Settings

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

### Community 44 - ".create"
Cohesion: 0.33
Nodes (10): device_mode_reports_no_passphrase(), env(), foreign_device_key_does_not_open(), keychain_is_preferred_and_file_is_not_created(), passphrase_is_required_and_checked(), plaintext_is_not_in_the_file(), records_survive_a_restart(), scan_and_delete_prefix_are_scoped() (+2 more)

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (32): unix_now(), SubscriptionGuard, handoff_if_needed(), purge_expired(), run(), InSessions, add_subscriber(), after_store() (+24 more)

### Community 46 - "peer.rs"
Cohesion: 0.14
Nodes (27): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+19 more)

### Community 47 - "n2n.rs"
Cohesion: 0.20
Nodes (17): call(), call_once(), Established, Refused, Session, Exchange, exchange(), Done (+9 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (49): 13.4 Jádro: crate `ancord-core`, 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake` (+41 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.24
Nodes (25): acceptContact(), addContact(), boot(), changePassphrase(), createIdentity(), declineContact(), deleteConversation(), enterReady() (+17 more)

### Community 50 - "handler.rs"
Cohesion: 0.30
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 51 - "android_keystore.rs"
Cohesion: 0.12
Nodes (8): AndroidBiometric, AndroidKeystoreDeviceKey, Bridge, BRIDGE, call(), call_on(), io(), JNI_OnLoad()

### Community 52 - "space.rs"
Cohesion: 0.07
Nodes (32): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+24 more)

### Community 53 - "android_sync.rs"
Cohesion: 0.36
Nodes (3): Java_cz_melonet_ancord_SyncReceiver_nativeSync(), LogSink, run()

### Community 54 - "app"
Cohesion: 0.16
Nodes (13): app, tabOf(), totalUnread(), View, current, tabs, unread, area (+5 more)

### Community 56 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (26): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+18 more)

### Community 57 - "src/e2e.rs"
Cohesion: 0.15
Nodes (12): FAST, free_port(), is_loopback_port(), LocalNet, NODES, online(), received(), repo() (+4 more)

### Community 58 - "ProofError"
Cohesion: 0.13
Nodes (13): ProofError, BadRequest, InvalidProof, is_for(), MAX_OTKS, parse_pk(), pow_nonce(), Profile (+5 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 61 - ".unwrap"
Cohesion: 0.23
Nodes (18): add_sends_a_name_only_intro_and_an_incoming_request_is_accepted_or_declined(), biometric_is_refused_for_a_profile_without_passphrase(), biometric_unlocks_only_a_passphrase_profile_and_password_still_works(), contact_shows_the_key_until_the_peer_name_arrives_and_a_label_wins(), contacts_reject_garbage_self_and_duplicates(), display_name_is_cleaned_persisted_and_sent_with_queued_messages(), identity_contacts_and_settings_survive_a_restart(), identity_is_created_once_and_address_has_the_scheme() (+10 more)

### Community 62 - "server.rs"
Cohesion: 0.15
Nodes (7): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S

### Community 64 - "services"
Cohesion: 0.14
Nodes (13): name, services, volumes, node1-data, node2-data, node3-data, node4-data, node5-data (+5 more)

### Community 65 - "Chat.svelte"
Cohesion: 0.09
Nodes (31): 13.14 Milníky fáze D, contactName(), byName(), collator, connLabel, connText(), dayLabel(), fold() (+23 more)

### Community 66 - "tauri.conf.json"
Cohesion: 0.10
Nodes (20): minSdkVersion, app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand (+12 more)

### Community 67 - "options"
Cohesion: 0.29
Nodes (11): driver, options, logging, logging, logging, logging, logging, logging (+3 more)

### Community 68 - "NODE_NAME"
Cohesion: 0.53
Nodes (10): NODE_NAME, RUST_LOG, WORKER_PUBKEY, WORKER_URL, environment, environment, environment, environment (+2 more)

### Community 69 - "Shared"
Cohesion: 0.12
Nodes (5): Awaiting, Inner, lock_err(), Shared, NetConfig

### Community 70 - "vectors.test.ts"
Cohesion: 0.06
Nodes (69): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+61 more)

### Community 71 - "node1"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 72 - "node2"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 73 - "scheduler.ts"
Cohesion: 0.12
Nodes (29): blobKey(), ensureBlob(), ALLOWED_COLUMNS, ChallengeRow, Dim, DIM_COL, dispatch(), dispatchWitness() (+21 more)

### Community 74 - "node3"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 75 - "pairwise.py"
Cohesion: 0.10
Nodes (22): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+14 more)

### Community 76 - "src-tauri/src/store.rs"
Cohesion: 0.11
Nodes (14): FAST, CONTEXT, FAST, KdfParams, META, MODE_DEVICE, MODE_PASSPHRASE, NONCE_LEN (+6 more)

### Community 77 - "node4"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 78 - "node5"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 79 - "node6"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 80 - "Link"
Cohesion: 0.29
Nodes (4): Cmd, Flush, Typing, Link

### Community 81 - "x-node"
Cohesion: 0.22
Nodes (9): x-node, build, cap_drop, image, read_only, restart, security_opt, stop_grace_period (+1 more)

### Community 82 - "ancord-cli/src/main.rs"
Cohesion: 0.06
Nodes (31): create_user(), load_session(), load_user(), lock_file(), save_session(), save_user(), write_private(), Cli (+23 more)

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

### Community 86 - "compilerOptions"
Cohesion: 0.15
Nodes (12): compilerOptions, module, moduleResolution, noUncheckedIndexedAccess, noUnusedLocals, skipLibCheck, strict, target (+4 more)

### Community 87 - "Core"
Cohesion: 0.10
Nodes (35): Core, address_get(), address_qr(), apply_protection(), AppSink, biometric_disable(), biometric_enable(), biometric_status() (+27 more)

### Community 88 - "ticket.rs"
Cohesion: 0.14
Nodes (15): DOMAIN_DIRECTORY, DOMAIN_TICKET, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody, TicketError, BadSignature (+7 more)

### Community 90 - "BiometricKey.kt"
Cohesion: 0.10
Nodes (3): BiometricKey, AuthenticationCallback, KeystoreKey

### Community 91 - "ancord"
Cohesion: 0.25
Nodes (7): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura

### Community 92 - "screens.test.ts"
Cohesion: 0.43
Nodes (6): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start()

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.20
Nodes (9): ancord klient (Android a desktop), Android release APK, Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14), Síť, Vzhled (0.2.0) (+1 more)

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 100 - "UiEvent"
Cohesion: 0.15
Nodes (10): UiEvent, ConnectionIssue, ConnectionState, ContactRequest, ContactUpdate, DeliveryState, MessageReceived, Typing (+2 more)

### Community 106 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Banned, Reconnect, Reregister, run()

### Community 109 - "ancord-node/src/main.rs"
Cohesion: 0.20
Nodes (5): NodeError, main(), resolve_worker_key(), run(), shutdown_signal()

### Community 112 - "reach.rs"
Cohesion: 0.17
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 113 - "Settings.svelte"
Cohesion: 0.14
Nodes (12): ThemeChoice, { checked, onchange, label }, locks, pwBad, pwBusy, pwDone, pwNew, pwNew2 (+4 more)

### Community 115 - "sha256"
Cohesion: 0.21
Nodes (8): leading_zero_bits(), solve(), valid(), compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start()

### Community 118 - "message.rs"
Cohesion: 0.27
Nodes (9): Content, FRAME_V1, MAX_NAME_BYTES, MAX_NAME_CHARS, name_and_text_round_trip(), names_are_cleaned_and_limited(), pack(), text_starting_with_the_marker_is_escaped_by_a_frame() (+1 more)

### Community 119 - "registry.ts"
Cohesion: 0.20
Nodes (18): DirectoryBody, DirectoryNode, eligibleForDirectory(), ensureDirectory(), epochOf(), readDirectory(), SignedDirectory, excludedIds() (+10 more)

### Community 120 - "reach.ts"
Cohesion: 0.15
Nodes (13): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu), log() (+5 more)

### Community 123 - "scoring.ts"
Cohesion: 0.24
Nodes (11): BeginInput, capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf(), trustOf() (+3 more)

### Community 125 - "canonical.rs"
Cohesion: 0.30
Nodes (7): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value()

### Community 126 - "identity.rs"
Cohesion: 0.09
Nodes (9): node_id_of(), Claimed, Config, HOSTNAME_ID_CHARS, hostname_of(), KEY_FILE, LoadMeter, measure() (+1 more)

### Community 127 - "registration.ts"
Cohesion: 0.14
Nodes (26): BwDir, signBwToken(), verifyBwToken(), genesisSet(), hmacSha256(), verifySig(), b64uDecode(), fromUtf8() (+18 more)

### Community 128 - "ConnectionIssue"
Cohesion: 0.29
Nodes (7): ConnectionIssue, NodeRejected, NodeUnreachable, ProtocolMismatch, RouteUntrusted, WorkerError, WorkerUnreachable

### Community 129 - "super"
Cohesion: 0.13
Nodes (6): base32_encode(), hex_encode(), anchors(), deterministic_and_bounded(), weight(), PUSH_ID_LEN

### Community 130 - "src-tauri/src/net.rs"
Cohesion: 0.15
Nodes (10): classify(), MAX_BACKOFF_S, PING_EVERY, run(), set_state(), Sink, start(), STATUS_EVERY (+2 more)

### Community 132 - "routing.ts"
Cohesion: 0.33
Nodes (8): ClientGeo, distanceKm(), haversineKm(), pickP2C(), selectNodes(), Snapshot, SnapshotNode, weightedIndex()

### Community 133 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

### Community 134 - "scripts"
Cohesion: 0.22
Nodes (9): scripts, budget, build, check, dev, preview, tauri, test (+1 more)

### Community 138 - "ThemeChoice"
Cohesion: 0.33
Nodes (5): Settings, ThemeChoice, Dark, Light, System

## Knowledge Gaps
- **126 isolated node(s):** `ancord-client`, `None`, `Locked`, `Ready`, `Queued` (+121 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 943 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **37 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `push/store.ts`, `b64uEncode`, `13. Klient (Android a desktop)`?**
  _High betweenness centrality (0.222) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `NodeState` (e.g. with `OutSessions` and `Reach`) actually correct?**
  _`NodeState` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ancord-client`, `None`, `Locked` to the rest of the system?**
  _126 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `ScanActivity.kt` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `ancord node`, `ratchet.py`, `pairwise.py`, `13. Klient (Android a desktop)`, `Melo v2 – architektura`?**
  _High betweenness centrality (0.179) - this node is a cross-community bridge._
- **Should `registry.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.1368421052631579 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `push.test.ts`, `index.ts`, `vectors.test.ts`, `scheduler.ts`, `push_id`, `registry.ts`, `push/store.ts`, `registration.ts`?**
  _High betweenness centrality (0.158) - this node is a cross-community bridge._