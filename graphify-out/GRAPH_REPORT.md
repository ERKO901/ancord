# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 196 files · ~143,781 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 58 file(s) not represented in the graph (top: .toml 27, .xml 13, (none) 9)

## Summary
- 3058 nodes · 8031 edges · 133 communities (107 shown, 26 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 418 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f24f1d78`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- StoreError
- registration.ts
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
- CloudflareClient
- Registry
- channel.py
- vectors.rs
- online.rs
- App.svelte
- ContactDetail.svelte
- AutoDeviceKey
- backend.ts
- ancord-core/src/net.rs
- tests/e2e.rs
- core.rs
- push/store.ts
- .initiate_with
- client.rs
- identity.rs
- deploy.ts
- dataplane.rs
- Cmd
- Table
- ancord-node/src/store.rs
- b64u_encode
- b64uEncode
- .with
- Session
- WorkerError
- contract.rs
- space_bench.rs
- VirtualList.svelte
- b64u_decode_array
- UserKeys
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
- 13. Klient (Android a desktop)
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
- channel_roundtrip_detects_reorder_replay_and_tamper
- biometric_unlocks_only_a_passphrase_profile_and_password_still_works
- BiometricKey.kt
- ancord
- screens.test.ts
- RO
- UserKeys
- ancord klient (Android a desktop)
- default.json
- build.rs
- UiEvent
- ancord-core/src/lib.rs
- ancord-client
- Melo v2 – architektura
- ancord-node/src/main.rs
- MainActivity
- pairwise.py
- Settings.svelte
- ancord-cli/src/main.rs
- sha256
- ScanActivity
- SpaceTable
- message.rs
- registry.ts
- ancord Worker
- duration
- Backing
- scoring.ts
- KdfParams
- path
- ticket.ts
- ConnectionIssue
- src-tauri/src/net.rs
- routing.ts
- ancord node
- scripts
- LocalNet

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 80 edges
3. `Core` - 75 edges
4. `DataError` - 59 edges
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

## Communities (133 total, 26 thin omitted)

### Community 1 - "StoreError"
Cohesion: 0.18
Nodes (11): io(), read_wrap(), StoreError, Corrupt, Io, Passphrase, Unlock, unwrap_key() (+3 more)

### Community 2 - "registration.ts"
Cohesion: 0.17
Nodes (21): genesisSet(), requiredBits(), timeoutSignal(), log(), probe(), activate(), assertAsnNotPenalized(), assertLimits() (+13 more)

### Community 3 - "push.test.ts"
Cohesion: 0.07
Nodes (61): vitest, createCloudflareClient(), sharedMockClient(), base, cache, ChallengeKind, DeepPartial, isObject() (+53 more)

### Community 4 - "index.ts"
Cohesion: 0.12
Nodes (50): verifyNodeRequest(), clock, workerSigner(), b64uDecodeLen(), ErrorCode, errorResponse(), HttpError, STATUS (+42 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "sign.rs"
Cohesion: 0.10
Nodes (20): MAX_OTKS, TS_SKEW_S, auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_HEARTBEAT, DOMAIN_N2N_RESP, DOMAIN_PROFILE (+12 more)

### Community 8 - "BuildTask.kt"
Cohesion: 0.13
Nodes (3): BuildTask, Config, RustPlugin

### Community 9 - "data.rs"
Cohesion: 0.06
Nodes (57): profile_id(), ack(), anchors_in(), blocking(), DEFAULT_FETCH, fan_out(), fetch(), find_node() (+49 more)

### Community 10 - "ratchet.py"
Cohesion: 0.18
Nodes (16): solve(), b64u(), bootstrap_seed(), header_aad(), header_key0(), header_nonce(), hkdf(), kdf_rk() (+8 more)

### Community 11 - "channel.rs"
Cohesion: 0.09
Nodes (32): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, ChannelError, BadPublicKey, Decrypt, Exhausted, Padding (+24 more)

### Community 12 - "package.json"
Cohesion: 0.09
Nodes (26): dependencies, @tauri-apps/api@^2.12.4, devDependencies, jsdom@^30.1.3, svelte@^5.57.3, svelte-check@^4.7.7, @sveltejs/vite-plugin-svelte@^7.3.1, @tauri-apps/cli@^2.12.4 (+18 more)

### Community 13 - "CloudflareClient"
Cohesion: 0.09
Nodes (12): apply, cf(), Envelope, main(), ApiEnvelope, CloudflareClient, isManagedHostname(), MANAGED_COMMENT (+4 more)

### Community 14 - "Registry"
Cohesion: 0.12
Nodes (9): Result, toResult(), SignedDirectory, AuthCtx, BeginResult, ConfirmInput, ConfirmResult, unbanNode() (+1 more)

### Community 15 - "channel.py"
Cohesion: 0.15
Nodes (14): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+6 more)

### Community 16 - "vectors.rs"
Cohesion: 0.05
Nodes (49): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value(), b64u_decode() (+41 more)

### Community 17 - "online.rs"
Cohesion: 0.21
Nodes (11): now_s(), publish_profile(), MAX_SUBSCRIPTIONS, Online, OnlineEvent, Contact, Message, Postponed (+3 more)

### Community 18 - "App.svelte"
Cohesion: 0.07
Nodes (26): version, hasMain, top, isTopLevel(), { title, subtitle, onback, hideBackOnWide = false, large = false, lead, right }, Name, { name, size = 22 }, paths (+18 more)

### Community 19 - "ContactDetail.svelte"
Cohesion: 0.11
Nodes (19): 13.14 Milníky fáze D, errText(), unlock(), ConnectionIssue, confirmDelete, contact, copied, editing (+11 more)

### Community 21 - "backend.ts"
Cohesion: 0.10
Nodes (22): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+14 more)

### Community 22 - "ancord-core/src/net.rs"
Cohesion: 0.09
Nodes (51): run(), Channel, b64u_encode(), Accepted, ack_inbox(), address(), admission(), admit() (+43 more)

### Community 23 - "tests/e2e.rs"
Cohesion: 0.05
Nodes (13): ADMIN, admin_nodes(), eventually(), forward(), free_port(), next_online_message(), NodeProc, NoDisk (+5 more)

### Community 24 - "core.rs"
Cohesion: 0.08
Nodes (17): BIO_FILE, ContactRec, conv_id(), IDENTITY_KEY, MAX_TEXT, msg_key(), NAME_KEY, Network (+9 more)

### Community 25 - "push/store.ts"
Cohesion: 0.08
Nodes (35): NodeAuth, aesGcmDecrypt(), aesGcmEncrypt(), Env, PushJob, queue(), AdmitOptions, AdmitResult (+27 more)

### Community 26 - ".initiate_with"
Cohesion: 0.09
Nodes (29): addresses_parse_with_or_without_the_scheme_and_reject_garbage(), dh(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION, expand(), FirstRatchetKey, Own (+21 more)

### Community 27 - "client.rs"
Cohesion: 0.06
Nodes (48): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+40 more)

### Community 28 - "identity.rs"
Cohesion: 0.10
Nodes (10): HOSTNAME_ID_CHARS, hostname_matches(), hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected(), activate() (+2 more)

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.09
Nodes (50): unix_now(), anchors_of(), anchors_send_signed_wake_only_for_new_messages_with_push_id(), call_peer(), caller_rejects_responder_that_cannot_prove_identity(), cluster(), cluster_with_worker(), delivery_status_follows_the_message_lifecycle() (+42 more)

### Community 31 - "Cmd"
Cohesion: 0.10
Nodes (20): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Publish (+12 more)

### Community 32 - "Table"
Cohesion: 0.16
Nodes (9): prefix_end(), Record, Store, Table, Contacts, Conversations, Identity, Messages (+1 more)

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.08
Nodes (35): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+27 more)

### Community 34 - "b64u_encode"
Cohesion: 0.12
Nodes (7): KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run()

### Community 35 - "b64uEncode"
Cohesion: 0.19
Nodes (6): RFC-8785, canonicalBytes(), canonicalJson(), utf8(), deletePush(), TestNode

### Community 36 - ".with"
Cohesion: 0.13
Nodes (3): ready_keys(), short_key(), clean_name()

### Community 37 - "Session"
Cohesion: 0.12
Nodes (14): StorePersist, deliver(), HistoryEntry, keepalive_after_real_message_only_and_not_shown(), pair(), parse_address(), Prepared, register_push() (+6 more)

### Community 38 - "WorkerError"
Cohesion: 0.10
Nodes (9): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse, nonce_b64(), signed_header() (+1 more)

### Community 39 - "contract.rs"
Cohesion: 0.11
Nodes (20): BiometricStatus, Contact, ConversationPage, ConversationSummary, DeliveryState, Delivered, Pending, Queued (+12 more)

### Community 40 - "space_bench.rs"
Cohesion: 0.19
Nodes (12): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+4 more)

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
Cohesion: 0.15
Nodes (9): Guard, MAX_BITS, payload(), stamp(), STAMP_LEN, stamp_nonce(), stamp_valid(), TS_SKEW_S (+1 more)

### Community 44 - "UserKeys"
Cohesion: 0.18
Nodes (12): both_sides_agree_and_the_one_time_prekey_is_consumed(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), Initiation, keys_survive_a_roundtrip_through_the_file(), pair(), profile_checks_time_and_proof_of_work(), push_ids_are_stable_and_bound_to_the_mailbox(), rotation_deletes_old_prekeys_so_old_handshakes_cannot_be_opened_later() (+4 more)

### Community 45 - "NodeState"
Cohesion: 0.06
Nodes (33): ConnGuard, SubscriptionGuard, handoff_if_needed(), purge_expired(), run(), InSessions, add_subscriber(), after_store() (+25 more)

### Community 46 - "peer.rs"
Cohesion: 0.14
Nodes (27): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+19 more)

### Community 47 - "n2n.rs"
Cohesion: 0.20
Nodes (17): call(), call_once(), Established, Refused, Session, Exchange, exchange(), Done (+9 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (46): 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake`, 10. Push (+38 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.26
Nodes (21): addContact(), boot(), changePassphrase(), createIdentity(), deleteConversation(), enterReady(), loadOlder(), lockNow() (+13 more)

### Community 50 - "handler.rs"
Cohesion: 0.30
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 51 - "android_keystore.rs"
Cohesion: 0.14
Nodes (8): AndroidBiometric, AndroidKeystoreDeviceKey, Bridge, BRIDGE, call(), call_on(), io(), JNI_OnLoad()

### Community 52 - "space.rs"
Cohesion: 0.15
Nodes (12): CHUNK, D0, DI, LEAF_BYTES, leaf_hash(), node_hash(), PAIR, ref_indices() (+4 more)

### Community 53 - "android_sync.rs"
Cohesion: 0.36
Nodes (3): Java_cz_melonet_ancord_SyncReceiver_nativeSync(), LogSink, run()

### Community 54 - "app"
Cohesion: 0.14
Nodes (12): app, go(), tabOf(), totalUnread(), View, current, tabs, unread (+4 more)

### Community 56 - "13. Klient (Android a desktop)"
Cohesion: 0.11
Nodes (20): 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.15 Mimo rozsah v1 klienta, 13.1 Cíle a pořadí priorit, 13.2 Volba technologie, 13.3 Vrstvy a hranice důvěry (+12 more)

### Community 57 - "src/e2e.rs"
Cohesion: 0.15
Nodes (14): ChanSink, client(), Egress, free_port(), is_loopback_port(), NODES, online(), received() (+6 more)

### Community 58 - "ProofError"
Cohesion: 0.10
Nodes (14): ProofError, BadRequest, InvalidProof, anchors(), deterministic_and_bounded(), weight(), is_for(), parse_pk() (+6 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 61 - ".unwrap"
Cohesion: 0.29
Nodes (15): biometric_is_refused_for_a_profile_without_passphrase(), contact_shows_the_key_until_the_peer_name_arrives_and_a_label_wins(), contacts_reject_garbage_self_and_duplicates(), display_name_is_cleaned_persisted_and_sent_with_queued_messages(), identity_contacts_and_settings_survive_a_restart(), identity_is_created_once_and_address_has_the_scheme(), legacy_placeholder_names_are_not_kept_as_labels(), memory_core_cannot_send() (+7 more)

### Community 62 - "server.rs"
Cohesion: 0.16
Nodes (7): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S

### Community 64 - "services"
Cohesion: 0.14
Nodes (13): name, services, volumes, node1-data, node2-data, node3-data, node4-data, node5-data (+5 more)

### Community 65 - "Chat.svelte"
Cohesion: 0.11
Nodes (27): contactName(), byName(), collator, connLabel, connText(), dayLabel(), fold(), hue() (+19 more)

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
Cohesion: 0.14
Nodes (6): Awaiting, biometric_unlocks_only_a_passphrase_profile_and_password_still_works(), Inner, lock_err(), locked_core_exposes_nothing(), Shared

### Community 70 - "vectors.test.ts"
Cohesion: 0.06
Nodes (64): keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload(), advance() (+56 more)

### Community 71 - "node1"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 72 - "node2"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 73 - "scheduler.ts"
Cohesion: 0.12
Nodes (47): blobKey(), ensureBlob(), ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth() (+39 more)

### Community 74 - "node3"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 75 - "reach.rs"
Cohesion: 0.20
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 76 - "src-tauri/src/store.rs"
Cohesion: 0.11
Nodes (25): auto(), CONTEXT, device_mode_reports_no_passphrase(), DeviceKey, Env, env(), Fake, FileDeviceKey (+17 more)

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
Cohesion: 0.11
Nodes (33): Core, address_get(), address_qr(), apply_protection(), AppSink, biometric_disable(), biometric_enable(), biometric_status() (+25 more)

### Community 88 - "channel_roundtrip_detects_reorder_replay_and_tamper"
Cohesion: 0.23
Nodes (6): channel_roundtrip_detects_reorder_replay_and_tamper(), derive(), different_transcript_yields_different_keys(), Ephemeral, frame_counter_exhaustion_is_fatal(), SessionKeys

### Community 90 - "BiometricKey.kt"
Cohesion: 0.10
Nodes (3): BiometricKey, AuthenticationCallback, KeystoreKey

### Community 91 - "ancord"
Cohesion: 0.25
Nodes (7): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura

### Community 92 - "screens.test.ts"
Cohesion: 0.43
Nodes (6): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start()

### Community 95 - "UserKeys"
Cohesion: 0.17
Nodes (7): create_user(), load_session(), load_user(), save_user(), write_private(), safety(), UserKeys

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.22
Nodes (8): ancord klient (Android a desktop), Android release APK, Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14), Síť, Vzhled (0.2.0)

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 100 - "UiEvent"
Cohesion: 0.12
Nodes (12): ConnectionState, Connecting, Offline, Online, UiEvent, ConnectionIssue, ConnectionState, ContactRequest (+4 more)

### Community 106 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (26): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+18 more)

### Community 109 - "ancord-node/src/main.rs"
Cohesion: 0.18
Nodes (7): NodeError, main(), resolve_worker_key(), run(), shutdown_signal(), run(), backoff()

### Community 112 - "pairwise.py"
Cohesion: 0.18
Nodes (9): 13.4 Jádro: crate `ancord-core`, b64u(), hkdf(), mailbox(), profile(), raw(), sha(), x3dh() (+1 more)

### Community 113 - "Settings.svelte"
Cohesion: 0.17
Nodes (10): { checked, onchange, label }, locks, pwBad, pwBusy, pwDone, pwNew, pwNew2, pwOld (+2 more)

### Community 114 - "ancord-cli/src/main.rs"
Cohesion: 0.06
Nodes (25): lock_file(), save_session(), Cli, Online, Route, conversation_files(), DirStore, label_of() (+17 more)

### Community 115 - "sha256"
Cohesion: 0.21
Nodes (8): leading_zero_bits(), solve(), valid(), compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start()

### Community 117 - "SpaceTable"
Cohesion: 0.24
Nodes (5): SpaceError, BadIndex, BadLeaves, Io, SpaceTable

### Community 118 - "message.rs"
Cohesion: 0.27
Nodes (9): Content, FRAME_V1, MAX_NAME_BYTES, MAX_NAME_CHARS, name_and_text_round_trip(), names_are_cleaned_and_limited(), pack(), text_starting_with_the_marker_is_escaped_by_a_frame() (+1 more)

### Community 119 - "registry.ts"
Cohesion: 0.12
Nodes (38): Json, Config, Signer, RegCtx, broadcastDirectory(), DirectoryBody, DirectoryNode, eligibleForDirectory() (+30 more)

### Community 120 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 122 - "Backing"
Cohesion: 0.40
Nodes (3): Backing, File, Mem

### Community 123 - "scoring.ts"
Cohesion: 0.31
Nodes (9): capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf(), trustOf(), utilOf() (+1 more)

### Community 124 - "KdfParams"
Cohesion: 0.40
Nodes (4): FAST, FAST, FAST, KdfParams

### Community 126 - "path"
Cohesion: 0.27
Nodes (3): LoadMeter, measure(), Measured

### Community 127 - "ticket.ts"
Cohesion: 0.19
Nodes (16): BwDir, signBwToken(), verifyBwToken(), hmacSha256(), verifySig(), B64U_LOOKUP, b64uDecode(), decoder (+8 more)

### Community 128 - "ConnectionIssue"
Cohesion: 0.29
Nodes (7): ConnectionIssue, NodeRejected, NodeUnreachable, ProtocolMismatch, RouteUntrusted, WorkerError, WorkerUnreachable

### Community 130 - "src-tauri/src/net.rs"
Cohesion: 0.10
Nodes (16): classify(), Cmd, Flush, Typing, Link, MAX_BACKOFF_S, NetConfig, PING_EVERY (+8 more)

### Community 132 - "routing.ts"
Cohesion: 0.33
Nodes (8): ClientGeo, distanceKm(), haversineKm(), pickP2C(), selectNodes(), Snapshot, SnapshotNode, weightedIndex()

### Community 133 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

### Community 134 - "scripts"
Cohesion: 0.22
Nodes (9): scripts, budget, build, check, dev, preview, tauri, test (+1 more)

## Knowledge Gaps
- **126 isolated node(s):** `ancord-client`, `None`, `Locked`, `Ready`, `Queued` (+121 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 932 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **26 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `b64uEncode`, `push.test.ts`, `vectors.test.ts`, `pairwise.py`, `13. Klient (Android a desktop)`?**
  _High betweenness centrality (0.239) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `NodeState` (e.g. with `OutSessions` and `Reach`) actually correct?**
  _`NodeState` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ancord-client`, `None`, `Locked` to the rest of the system?**
  _126 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `ScanActivity.kt` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `ancord node`, `Melo v2 – architektura`, `channel.py`, `pairwise.py`, `13. Klient (Android a desktop)`?**
  _High betweenness centrality (0.194) - this node is a cross-community bridge._
- **Should `push.test.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.07183544303797468 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `vectors.test.ts` to `registration.ts`, `push.test.ts`, `index.ts`, `b64uEncode`, `scheduler.ts`, `push_id`, `registry.ts`, `push/store.ts`, `ticket.ts`?**
  _High betweenness centrality (0.145) - this node is a cross-community bridge._