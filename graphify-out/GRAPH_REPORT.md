# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 174 files · ~136,243 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 30 file(s) not represented in the graph (top: .toml 18, (none) 6, .xml 2)

## Summary
- 2748 nodes · 7348 edges · 110 communities (96 shown, 14 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 410 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `de9d4f8a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Melo v2 – architektura
- src-tauri/src/store.rs
- scheduler.ts
- push.test.ts
- index.ts
- ancord – protokol (verze 1)
- ratchet.rs
- CliError
- NodeSocket
- data.rs
- ratchet.py
- channel.rs
- package.json
- RealCloudflareClient
- Registry
- TestNode
- vectors.rs
- registration.ts
- App.svelte
- space.rs
- WorkerClient
- backend.ts
- ancord-core/src/net.rs
- tests/e2e.rs
- n2n.rs
- sign.rs
- .initiate_with
- client.rs
- duration
- deploy.ts
- dataplane.rs
- Cmd
- src-tauri/src/net.rs
- ancord-node/src/store.rs
- control.rs
- Identity
- pairwise.py
- ancord-node/src/main.rs
- handler.rs
- json
- ancord node
- VirtualList.svelte
- core.rs
- snapshot.ts
- NodeState
- peer.rs
- push/store.ts
- push_id
- app.svelte.ts
- ws.ts
- WorkerError
- src/e2e.rs
- registry.ts
- Step
- local-forward.py
- routing.ts
- 13. Klient (Android a desktop)
- Profile
- Šest nodů v Dockeru (C3)
- b64u_encode
- mailbox_push_key
- .nowS
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
- identity.rs
- Shared
- node4
- node5
- node6
- config.ts
- x-node
- ancord Worker
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh
- compilerOptions
- Core
- run
- pair
- admit_unchecked
- ancord
- screens.test.ts
- RO
- MockCloudflareClient
- user.rs
- server.rs
- ancord klient (Android a desktop)
- default.json
- build.rs
- .lock
- channel_roundtrip_detects_reorder_replay_and_tamper
- ancord-client
- reach.rs
- cluster_with_worker
- TestNode
- peer_channel_rejects_plaintext_tamper_replay_and_unknown_session

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 90 edges
3. `Session` - 64 edges
4. `DataError` - 59 edges
5. `Core` - 56 edges
6. `b64uEncode()` - 54 edges
7. `unix_now()` - 53 edges
8. `NodeSocket` - 49 edges
9. `UserKeys` - 49 edges
10. `b64uDecodeLen()` - 45 edges

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
- 2-file cycle: `node/crates/ancord-node/src/presence.rs -> node/crates/ancord-node/src/state.rs -> node/crates/ancord-node/src/presence.rs`
- 2-file cycle: `client/src-tauri/src/core.rs -> client/src-tauri/src/net.rs -> client/src-tauri/src/core.rs`

## Communities (110 total, 14 thin omitted)

### Community 0 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (26): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+18 more)

### Community 1 - "src-tauri/src/store.rs"
Cohesion: 0.06
Nodes (47): FAST, FAST, auto(), AutoDeviceKey, CONTEXT, device_mode_reports_no_passphrase(), DeviceKey, Env (+39 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.12
Nodes (43): blobKey(), ensureBlob(), ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth() (+35 more)

### Community 3 - "push.test.ts"
Cohesion: 0.09
Nodes (49): vitest, ApiEnvelope, createCloudflareClient(), MockCall, sharedMockClient(), loadConfig(), validate(), withOverrides() (+41 more)

### Community 4 - "index.ts"
Cohesion: 0.11
Nodes (54): NodeAuth, verifyNodeRequest(), clock, workerSigner(), b64uDecodeLen(), PushJob, ErrorCode, errorResponse() (+46 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "CliError"
Cohesion: 0.10
Nodes (20): StorePersist, create_user(), load_session(), lock_file(), save_session(), save_user(), write_private(), DirStore (+12 more)

### Community 8 - "NodeSocket"
Cohesion: 0.18
Nodes (12): NodeSocket, now_s(), publish_guard(), publish_profile(), Online, OnlineEvent, Contact, Message (+4 more)

### Community 9 - "data.rs"
Cohesion: 0.06
Nodes (54): ack(), ack_local(), anchors_in(), check_ts(), DataError, BadRequest, InsufficientNodes, InvalidProof (+46 more)

### Community 10 - "ratchet.py"
Cohesion: 0.17
Nodes (18): 9. Otevřené body, b64u(), bootstrap_seed(), dh(), header_aad(), header_key0(), header_nonce(), hkdf() (+10 more)

### Community 11 - "channel.rs"
Cohesion: 0.08
Nodes (32): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, ChannelError, BadPublicKey, Decrypt, Exhausted, Padding (+24 more)

### Community 12 - "package.json"
Cohesion: 0.05
Nodes (34): dependencies, @tauri-apps/api, devDependencies, jsdom, svelte, svelte-check, @sveltejs/vite-plugin-svelte, @tauri-apps/cli (+26 more)

### Community 13 - "RealCloudflareClient"
Cohesion: 0.17
Nodes (8): apply, cf(), Envelope, main(), isManagedHostname(), MANAGED_COMMENT, RealCloudflareClient, TUNNEL_NAME_PREFIX

### Community 14 - "Registry"
Cohesion: 0.12
Nodes (8): Result, toResult(), AuthCtx, BeginResult, closeSockets(), ConfirmInput, ConfirmResult, Registry

### Community 15 - "TestNode"
Cohesion: 0.23
Nodes (3): registerPush(), nowS(), TestNode

### Community 16 - "vectors.rs"
Cohesion: 0.05
Nodes (49): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value(), b64u_decode() (+41 more)

### Community 17 - "registration.ts"
Cohesion: 0.10
Nodes (35): BwDir, signBwToken(), verifyBwToken(), genesisSet(), ED25519, hmacSha256(), HOSTNAME_ID_CHARS, hostnameOf() (+27 more)

### Community 18 - "App.svelte"
Cohesion: 0.13
Nodes (10): hasMain, { title, onback, hideBackOnWide = false, right }, address, busy, name, address, copied, qr (+2 more)

### Community 19 - "space.rs"
Cohesion: 0.05
Nodes (39): leading_zero_bits(), solve(), valid(), attack(), Attacker, build(), CHAIN, log2_arg() (+31 more)

### Community 20 - "WorkerClient"
Cohesion: 0.14
Nodes (5): BeginResponse, ConfirmResponse, nonce_b64(), signed_header(), WorkerClient

### Community 21 - "backend.ts"
Cohesion: 0.13
Nodes (21): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+13 more)

### Community 22 - "ancord-core/src/net.rs"
Cohesion: 0.09
Nodes (38): Channel, b64u_encode(), Accepted, ack_inbox(), deliver(), Event, Message, Typing (+30 more)

### Community 23 - "tests/e2e.rs"
Cohesion: 0.04
Nodes (14): main(), PUSH_FILE, MAX_SUBSCRIPTIONS, ADMIN, admin_nodes(), eventually(), forward(), free_port() (+6 more)

### Community 24 - "n2n.rs"
Cohesion: 0.21
Nodes (15): SessionKeys, call(), call_once(), Established, Refused, Session, Exchange, Done (+7 more)

### Community 25 - "sign.rs"
Cohesion: 0.10
Nodes (21): auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_DIRECTORY, DOMAIN_HEARTBEAT, DOMAIN_INBOX_GUARD, DOMAIN_N2N_RESP, DOMAIN_PROFILE (+13 more)

### Community 26 - ".initiate_with"
Cohesion: 0.14
Nodes (17): addresses_parse_with_or_without_the_scheme_and_reject_garbage(), dh(), envelope_key(), FirstRatchetKey, Own, Peer, Opened, Pairwise (+9 more)

### Community 27 - "client.rs"
Cohesion: 0.09
Nodes (32): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+24 more)

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.17
Nodes (25): unix_now(), anchors_of(), call_peer(), caller_rejects_responder_that_cannot_prove_identity(), cluster(), delivery_status_follows_the_message_lifecycle(), dnode(), forged_or_unproven_profiles_are_refused() (+17 more)

### Community 31 - "Cmd"
Cohesion: 0.09
Nodes (22): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Online (+14 more)

### Community 32 - "src-tauri/src/net.rs"
Cohesion: 0.10
Nodes (14): Cmd, Flush, Typing, Link, MAX_BACKOFF_S, NetConfig, PING_EVERY, run() (+6 more)

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.07
Nodes (37): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+29 more)

### Community 34 - "control.rs"
Cohesion: 0.11
Nodes (8): heartbeat(), KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run()

### Community 35 - "Identity"
Cohesion: 0.12
Nodes (10): node_id_of(), hostname_matches(), Identity, key_persists_with_private_mode(), loose_permissions_are_rejected(), activate(), register(), Registration (+2 more)

### Community 36 - "pairwise.py"
Cohesion: 0.14
Nodes (9): b64u(), hkdf(), mailbox(), profile(), raw(), sha(), solve(), x3dh() (+1 more)

### Community 37 - "ancord-node/src/main.rs"
Cohesion: 0.27
Nodes (5): NodeError, main(), resolve_worker_key(), run(), shutdown_signal()

### Community 38 - "handler.rs"
Cohesion: 0.25
Nodes (13): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, space_commit() (+5 more)

### Community 39 - "json"
Cohesion: 0.19
Nodes (13): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+5 more)

### Community 40 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

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

### Community 43 - "core.rs"
Cohesion: 0.11
Nodes (13): ContactRec, conv_id(), identity_is_created_once_and_address_has_the_scheme(), IDENTITY_KEY, MAX_TEXT, msg_key(), now(), now_ms() (+5 more)

### Community 44 - "snapshot.ts"
Cohesion: 0.21
Nodes (17): DirectoryBody, DirectoryNode, eligibleForDirectory(), ensureDirectory(), epochOf(), readDirectory(), SignedDirectory, excludedIds() (+9 more)

### Community 45 - "NodeState"
Cohesion: 0.06
Nodes (30): ConnGuard, SubscriptionGuard, handoff_if_needed(), purge_expired(), run(), add_subscriber(), after_store(), announce() (+22 more)

### Community 46 - "peer.rs"
Cohesion: 0.12
Nodes (29): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+21 more)

### Community 47 - "push/store.ts"
Cohesion: 0.11
Nodes (23): aesGcmDecrypt(), aesGcmEncrypt(), fromUtf8(), queue(), processPushBatch(), b64urJson(), CachedToken, createPushSender() (+15 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (48): 13.4 Jádro: crate `ancord-core`, 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake` (+40 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.20
Nodes (22): addContact(), boot(), createIdentity(), deleteConversation(), enterReady(), go(), loadOlder(), lockNow() (+14 more)

### Community 50 - "ws.ts"
Cohesion: 0.18
Nodes (17): startNext(), Signer, timeoutSignal(), log(), RegCtx, broadcastDirectory(), Candidate, MAX_REPORTED (+9 more)

### Community 51 - "WorkerError"
Cohesion: 0.20
Nodes (4): WorkerError, Api, Protocol, Transport

### Community 52 - "src/e2e.rs"
Cohesion: 0.06
Nodes (40): ConnectionState, Connecting, Offline, Online, Contact, ConversationPage, ConversationSummary, DeliveryState (+32 more)

### Community 53 - "registry.ts"
Cohesion: 0.16
Nodes (18): Config, BeginInput, NodeSummary, capacityOf(), isLive(), parseClaimed(), parseLoad(), Score (+10 more)

### Community 54 - "Step"
Cohesion: 0.40
Nodes (4): Step, Frame, Line, Quit

### Community 56 - "routing.ts"
Cohesion: 0.33
Nodes (8): ClientGeo, distanceKm(), haversineKm(), pickP2C(), selectNodes(), Snapshot, SnapshotNode, weightedIndex()

### Community 57 - "13. Klient (Android a desktop)"
Cohesion: 0.13
Nodes (17): 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.14 Milníky fáze D, 13.15 Mimo rozsah v1 klienta, 13.1 Cíle a pořadí priorit, 13.2 Volba technologie (+9 more)

### Community 58 - "Profile"
Cohesion: 0.07
Nodes (21): ProofError, BadRequest, InvalidProof, Guard, MAX_BITS, payload(), stamp(), STAMP_LEN (+13 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 60 - "b64u_encode"
Cohesion: 0.11
Nodes (17): local.sh script, profile_id(), blocking(), from_record(), handoff(), put_local(), required_bits(), to_record() (+9 more)

### Community 63 - ".nowS"
Cohesion: 0.12
Nodes (11): ensureSchedule(), slotsOf(), CloudflareClient, consumeNonce(), deregister(), releaseResources(), requireKnownNode(), sweepRegistrations() (+3 more)

### Community 64 - "services"
Cohesion: 0.14
Nodes (13): name, services, volumes, node1-data, node2-data, node3-data, node4-data, node5-data (+5 more)

### Community 65 - "Chat.svelte"
Cohesion: 0.13
Nodes (22): app, contactName(), connLabel, initials(), shortAddress(), stateLabel, timeFull(), timeShort() (+14 more)

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
Cohesion: 0.18
Nodes (9): contacts_reject_garbage_self_and_duplicates(), identity_contacts_and_settings_survive_a_restart(), memory_core_cannot_send(), offline_messages_are_queued_paged_and_survive_a_restart(), qr_is_an_svg(), ready(), ready_keys(), safety_number_is_the_same_on_both_sides() (+1 more)

### Community 70 - "b64uEncode"
Cohesion: 0.06
Nodes (65): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+57 more)

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

### Community 75 - "identity.rs"
Cohesion: 0.08
Nodes (9): System, Claimed, Config, HOSTNAME_ID_CHARS, hostname_of(), KEY_FILE, LoadMeter, measure() (+1 more)

### Community 76 - "Shared"
Cohesion: 0.16
Nodes (4): Inner, save_contact_rec(), Shared, update_message()

### Community 77 - "node4"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 78 - "node5"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 79 - "node6"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 80 - "config.ts"
Cohesion: 0.12
Nodes (16): base, cache, ChallengeKind, DeepPartial, isObject(), merge(), ProfileName, PROFILES (+8 more)

### Community 81 - "x-node"
Cohesion: 0.22
Nodes (9): x-node, build, cap_drop, image, read_only, restart, security_opt, stop_grace_period (+1 more)

### Community 82 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

### Community 86 - "compilerOptions"
Cohesion: 0.15
Nodes (12): compilerOptions, module, moduleResolution, noUncheckedIndexedAccess, noUnusedLocals, skipLibCheck, strict, target (+4 more)

### Community 87 - "Core"
Cohesion: 0.12
Nodes (21): Core, address_get(), address_qr(), AppSink, connection_state(), contact_add(), contact_list(), contact_mark_verified() (+13 more)

### Community 88 - "run"
Cohesion: 0.18
Nodes (15): load_user(), Cli, conversation_files(), load_push(), load_session(), now(), online_session(), OnlineCtx (+7 more)

### Community 89 - "pair"
Cohesion: 0.18
Nodes (12): both_sides_agree_and_the_one_time_prekey_is_consumed(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), Initiation, keys_survive_a_roundtrip_through_the_file(), pair(), profile_checks_time_and_proof_of_work(), push_ids_are_stable_and_bound_to_the_mailbox(), rotation_deletes_old_prekeys_so_old_handshakes_cannot_be_opened_later() (+4 more)

### Community 90 - "admit_unchecked"
Cohesion: 0.17
Nodes (13): open(), admission(), admit(), admit_unchecked(), AllFailed, Connection, connect_any(), default_url() (+5 more)

### Community 91 - "ancord"
Cohesion: 0.25
Nodes (7): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura

### Community 92 - "screens.test.ts"
Cohesion: 0.43
Nodes (6): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start()

### Community 95 - "user.rs"
Cohesion: 0.12
Nodes (12): ENVELOPE_HEAD, ENVELOPE_VERSION, expand(), NOW, parse_address(), PROFILE_MAX_AGE_S, safety_number(), SPK_KEEP_S (+4 more)

### Community 96 - "server.rs"
Cohesion: 0.16
Nodes (6): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, SKEW_S

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.29
Nodes (6): ancord klient (Android a desktop), Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14), Síť

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 100 - ".lock"
Cohesion: 0.18
Nodes (3): lock_err(), locked_core_exposes_nothing(), passphrase_profile_stays_locked_until_the_right_passphrase()

### Community 101 - "channel_roundtrip_detects_reorder_replay_and_tamper"
Cohesion: 0.24
Nodes (5): channel_roundtrip_detects_reorder_replay_and_tamper(), derive(), different_transcript_yields_different_keys(), Ephemeral, frame_counter_exhaustion_is_fatal()

### Community 106 - "reach.rs"
Cohesion: 0.20
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 107 - "cluster_with_worker"
Cohesion: 0.26
Nodes (10): anchors_send_signed_wake_only_for_new_messages_with_push_id(), cluster_with_worker(), local_coalescing_limits_wakes_per_push_id(), mock_worker(), MockWorker, online_recipient_gets_message_pushed_and_no_wake(), stale_presence_falls_back_to_wake(), subscribe() (+2 more)

### Community 108 - "TestNode"
Cohesion: 0.27
Nodes (9): directory(), node_behind_on_epoch_pulls_directory_from_peer_after_403(), publish(), rejects_when_directory_is_missing_or_too_small(), signed_directory(), spawn_node(), spawn_node_full(), spawn_node_with_worker() (+1 more)

### Community 109 - "peer_channel_rejects_plaintext_tamper_replay_and_unknown_session"
Cohesion: 0.38
Nodes (5): envelope(), manual_handshake(), ManualSession, peer_channel_rejects_plaintext_tamper_replay_and_unknown_session(), post_x()

## Knowledge Gaps
- **126 isolated node(s):** `@tauri-apps/cli`, `@tsconfig/svelte`, `@types/node`, `jsdom`, `svelte-check` (+121 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 862 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **14 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `13. Klient (Android a desktop)`, `push.test.ts`, `b64uEncode`, `TestNode`?**
  _High betweenness centrality (0.278) - this node is a cross-community bridge._
- **What connects `@tauri-apps/cli`, `@tsconfig/svelte`, `@types/node` to the rest of the system?**
  _126 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Melo v2 – architektura` be split into smaller, more focused modules?**
  _Cohesion score 0.07407407407407407 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `Melo v2 – architektura`, `pairwise.py`, `json`, `ancord node`, `13. Klient (Android a desktop)`?**
  _High betweenness centrality (0.193) - this node is a cross-community bridge._
- **Should `src-tauri/src/store.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05965324713488099 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `scheduler.ts`, `push.test.ts`, `index.ts`, `snapshot.ts`, `push/store.ts`, `TestNode`, `registration.ts`, `push_id`?**
  _High betweenness centrality (0.148) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.12424242424242424 - nodes in this community are weakly interconnected._