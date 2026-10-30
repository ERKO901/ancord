# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 185 files · ~147,609 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 47 file(s) not represented in the graph (top: .toml 18, .xml 11, (none) 9)

## Summary
- 2823 nodes · 7480 edges · 117 communities (98 shown, 19 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 414 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `7296228e`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- challenge.test.ts
- src-tauri/src/store.rs
- scheduler.ts
- push.test.ts
- index.ts
- ancord – protokol (verze 1)
- ratchet.rs
- CliError
- BuildTask.kt
- data.rs
- ratchet.py
- channel.rs
- package.json
- cloudflare.ts
- Registry
- pairwise.py
- vectors.rs
- registry.ts
- App.svelte
- space_bench.rs
- pair
- backend.ts
- Online
- tests/e2e.rs
- n2n.rs
- handler.ts
- user.rs
- client.rs
- register.rs
- deploy.ts
- dataplane.rs
- Cmd
- src-tauri/src/net.rs
- ancord-node/src/store.rs
- control.rs
- b64uEncode
- core.rs
- ancord-core/src/net.rs
- b64u_encode
- UiEvent
- sign.rs
- VirtualList.svelte
- b64u_decode_array
- snapshot.ts
- NodeState
- peer.rs
- ws.ts
- push_id
- app.svelte.ts
- handler.rs
- android_sync.rs
- space.rs
- routing.ts
- .with
- local-forward.py
- 13. Klient (Android a desktop)
- src/e2e.rs
- Profile
- Šest nodů v Dockeru (C3)
- ancord-node/src/profile.rs
- reach.rs
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
- Onboarding.svelte
- node3
- Session
- node4
- node5
- node6
- identity.rs
- x-node
- SpaceTable
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh
- compilerOptions
- Core
- Step
- super
- path
- ancord
- screens.test.ts
- RO
- run
- ancord klient (Android a desktop)
- default.json
- build.rs
- WorkerError
- ancord-client
- Melo v2 – architektura
- ancord-node/src/main.rs
- MainActivity.kt
- pathbuf
- UserKeys
- cpu.rs
- config.ts
- ancord node
- ancord Worker

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 91 edges
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

## Communities (117 total, 19 thin omitted)

### Community 0 - "challenge.test.ts"
Cohesion: 0.10
Nodes (39): advance(), advanceNative(), computeCheckpoints(), CpuParams, CpuVerdict, laneStart(), verifyCpu(), buildTable() (+31 more)

### Community 1 - "src-tauri/src/store.rs"
Cohesion: 0.06
Nodes (47): FAST, FAST, auto(), AutoDeviceKey, CONTEXT, device_mode_reports_no_passphrase(), DeviceKey, Env (+39 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.13
Nodes (42): ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth(), Dim, DIM_COL (+34 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (41): vitest, sharedMockClient(), Domain, randomBytes(), resetRouteCache(), allMeasured(), node(), pendingBandwidth() (+33 more)

### Community 4 - "index.ts"
Cohesion: 0.13
Nodes (47): verifyNodeRequest(), clock, workerSigner(), b64uDecodeLen(), errorResponse(), HttpError, unwrap(), clientInfo() (+39 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "CliError"
Cohesion: 0.12
Nodes (17): create_user(), load_session(), lock_file(), save_session(), save_user(), write_private(), DirStore, label_of() (+9 more)

### Community 8 - "BuildTask.kt"
Cohesion: 0.13
Nodes (3): BuildTask, Config, RustPlugin

### Community 9 - "data.rs"
Cohesion: 0.06
Nodes (55): ack(), ack_local(), anchors_in(), blocking(), check_ts(), DataError, BadRequest, InsufficientNodes (+47 more)

### Community 10 - "ratchet.py"
Cohesion: 0.16
Nodes (18): b64u(), bootstrap_seed(), dh(), header_aad(), header_key0(), header_nonce(), hkdf(), kdf_rk() (+10 more)

### Community 11 - "channel.rs"
Cohesion: 0.06
Nodes (38): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, channel_roundtrip_detects_reorder_replay_and_tamper(), ChannelError, BadPublicKey, Decrypt, Exhausted (+30 more)

### Community 12 - "package.json"
Cohesion: 0.05
Nodes (34): dependencies, @tauri-apps/api, devDependencies, jsdom, svelte, svelte-check, @sveltejs/vite-plugin-svelte, @tauri-apps/cli (+26 more)

### Community 13 - "cloudflare.ts"
Cohesion: 0.09
Nodes (12): apply, cf(), Envelope, main(), ApiEnvelope, CloudflareClient, isManagedHostname(), MANAGED_COMMENT (+4 more)

### Community 14 - "Registry"
Cohesion: 0.10
Nodes (11): createCloudflareClient(), ErrorCode, Result, STATUS, toResult(), AuthCtx, BeginResult, ConfirmInput (+3 more)

### Community 15 - "pairwise.py"
Cohesion: 0.11
Nodes (22): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+14 more)

### Community 16 - "vectors.rs"
Cohesion: 0.06
Nodes (47): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value(), b64u_decode() (+39 more)

### Community 17 - "registry.ts"
Cohesion: 0.16
Nodes (29): Config, genesisSet(), Signer, requiredBits(), timeoutSignal(), log(), RegCtx, Candidate (+21 more)

### Community 18 - "App.svelte"
Cohesion: 0.13
Nodes (10): hasMain, { title, onback, hideBackOnWide = false, right }, address, busy, name, address, copied, qr (+2 more)

### Community 19 - "space_bench.rs"
Cohesion: 0.21
Nodes (12): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+4 more)

### Community 20 - "pair"
Cohesion: 0.21
Nodes (10): both_sides_agree_and_the_one_time_prekey_is_consumed(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), keys_survive_a_roundtrip_through_the_file(), pair(), profile_checks_time_and_proof_of_work(), rotation_deletes_old_prekeys_so_old_handshakes_cannot_be_opened_later(), safety_number_is_symmetric_and_pair_specific(), stale_profiles_are_refused() (+2 more)

### Community 21 - "backend.ts"
Cohesion: 0.13
Nodes (21): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+13 more)

### Community 22 - "Online"
Cohesion: 0.18
Nodes (10): session_conv(), soft(), Task, Online, OnlineEvent, Contact, Message, Postponed (+2 more)

### Community 23 - "tests/e2e.rs"
Cohesion: 0.04
Nodes (13): main(), PUSH_FILE, MAX_SUBSCRIPTIONS, ADMIN, admin_nodes(), eventually(), forward(), next_online_message() (+5 more)

### Community 24 - "n2n.rs"
Cohesion: 0.22
Nodes (14): call(), call_once(), Established, Refused, Session, Exchange, Done, UnknownSession (+6 more)

### Community 25 - "handler.ts"
Cohesion: 0.08
Nodes (34): NodeAuth, aesGcmDecrypt(), aesGcmEncrypt(), base32Encode(), Env, PushJob, queue(), AdmitOptions (+26 more)

### Community 26 - "user.rs"
Cohesion: 0.08
Nodes (30): addresses_parse_with_or_without_the_scheme_and_reject_garbage(), dh(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION, expand(), FirstRatchetKey, Own (+22 more)

### Community 27 - "client.rs"
Cohesion: 0.08
Nodes (34): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+26 more)

### Community 28 - "register.rs"
Cohesion: 0.14
Nodes (8): hostname_matches(), activate(), register(), Registration, run(), start(), Tunnel, backoff()

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.10
Nodes (49): unix_now(), anchors_of(), anchors_send_signed_wake_only_for_new_messages_with_push_id(), call_peer(), caller_rejects_responder_that_cannot_prove_identity(), cluster(), cluster_with_worker(), delivery_status_follows_the_message_lifecycle() (+41 more)

### Community 31 - "Cmd"
Cohesion: 0.09
Nodes (22): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Online (+14 more)

### Community 32 - "src-tauri/src/net.rs"
Cohesion: 0.08
Nodes (20): Cmd, Flush, Typing, Link, MAX_BACKOFF_S, NetConfig, PING_EVERY, run() (+12 more)

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.07
Nodes (34): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+26 more)

### Community 34 - "control.rs"
Cohesion: 0.12
Nodes (7): KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run()

### Community 35 - "b64uEncode"
Cohesion: 0.12
Nodes (16): canonicalBytes(), blobKey(), BwDir, ensureBlob(), signBwToken(), verifyBwToken(), hmacSha256(), b64uEncode() (+8 more)

### Community 36 - "core.rs"
Cohesion: 0.09
Nodes (13): ContactRec, identity_is_created_once_and_address_has_the_scheme(), IDENTITY_KEY, MAX_TEXT, msg_key(), now(), now_ms(), OutItem (+5 more)

### Community 37 - "ancord-core/src/net.rs"
Cohesion: 0.09
Nodes (39): Channel, b64u_encode(), Accepted, ack_inbox(), address(), admission(), admit(), admit_unchecked() (+31 more)

### Community 38 - "b64u_encode"
Cohesion: 0.13
Nodes (6): worker_url_for(), BeginResponse, ConfirmResponse, nonce_b64(), signed_header(), WorkerClient

### Community 39 - "UiEvent"
Cohesion: 0.09
Nodes (24): ConnectionState, Connecting, Offline, Online, Contact, ConversationPage, ConversationSummary, DeliveryState (+16 more)

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

### Community 43 - "b64u_decode_array"
Cohesion: 0.13
Nodes (10): MAX_BITS, payload(), stamp(), STAMP_LEN, stamp_nonce(), stamp_valid(), TS_SKEW_S, leading_zero_bits() (+2 more)

### Community 44 - "snapshot.ts"
Cohesion: 0.14
Nodes (25): DirectoryBody, DirectoryNode, eligibleForDirectory(), SignedDirectory, excludedIds(), BeginInput, capacityOf(), isLive() (+17 more)

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (30): handoff_if_needed(), purge_expired(), run(), add_subscriber(), after_store(), announce(), coalesce(), current_presence() (+22 more)

### Community 46 - "peer.rs"
Cohesion: 0.13
Nodes (28): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+20 more)

### Community 47 - "ws.ts"
Cohesion: 0.16
Nodes (18): Json, ensureSchedule(), slotsOf(), broadcastDirectory(), ensureDirectory(), epochOf(), readDirectory(), parseUnreachable() (+10 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (47): 13.4 Jádro: crate `ancord-core`, 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake` (+39 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.20
Nodes (22): addContact(), boot(), createIdentity(), deleteConversation(), enterReady(), go(), loadOlder(), lockNow() (+14 more)

### Community 50 - "handler.rs"
Cohesion: 0.24
Nodes (12): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, space_commit() (+4 more)

### Community 51 - "android_sync.rs"
Cohesion: 0.27
Nodes (3): Java_cz_melonet_ancord_SyncReceiver_nativeSync(), LogSink, run()

### Community 52 - "space.rs"
Cohesion: 0.14
Nodes (11): Backing, Mem, CHUNK, D0, DI, LEAF_BYTES, node_hash(), PAIR (+3 more)

### Community 53 - "routing.ts"
Cohesion: 0.33
Nodes (8): ClientGeo, distanceKm(), haversineKm(), pickP2C(), selectNodes(), Snapshot, SnapshotNode, weightedIndex()

### Community 54 - ".with"
Cohesion: 0.13
Nodes (11): contacts_reject_garbage_self_and_duplicates(), conv_id(), identity_contacts_and_settings_survive_a_restart(), Inner, memory_core_cannot_send(), offline_messages_are_queued_paged_and_survive_a_restart(), qr_is_an_svg(), ready() (+3 more)

### Community 56 - "13. Klient (Android a desktop)"
Cohesion: 0.13
Nodes (17): 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.14 Milníky fáze D, 13.15 Mimo rozsah v1 klienta, 13.1 Cíle a pořadí priorit, 13.2 Volba technologie (+9 more)

### Community 57 - "src/e2e.rs"
Cohesion: 0.11
Nodes (15): ChanSink, client(), Egress, free_port(), is_loopback_port(), LocalNet, NODES, online() (+7 more)

### Community 58 - "Profile"
Cohesion: 0.12
Nodes (13): ProofError, BadRequest, InvalidProof, is_for(), MAX_OTKS, parse_pk(), pow_nonce(), Profile (+5 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 60 - "ancord-node/src/profile.rs"
Cohesion: 0.09
Nodes (18): local.sh script, Guard, profile_id(), from_record(), handoff(), put(), put_local(), to_record() (+10 more)

### Community 61 - "reach.rs"
Cohesion: 0.18
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 62 - "server.rs"
Cohesion: 0.15
Nodes (7): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S

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

### Community 69 - "Shared"
Cohesion: 0.16
Nodes (4): lock_err(), locked_core_exposes_nothing(), passphrase_profile_stays_locked_until_the_right_passphrase(), Shared

### Community 70 - "vectors.test.ts"
Cohesion: 0.07
Nodes (43): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+35 more)

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

### Community 76 - "Session"
Cohesion: 0.14
Nodes (15): StorePersist, deliver(), keepalive_after_real_message_only_and_not_shown(), keepalive_if_due(), now_s(), pair(), Prepared, receive() (+7 more)

### Community 77 - "node4"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 78 - "node5"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 79 - "node6"
Cohesion: 0.20
Nodes (10): build, cap_drop, image, read_only, restart, security_opt, stop_grace_period, tmpfs (+2 more)

### Community 80 - "identity.rs"
Cohesion: 0.11
Nodes (7): node_id_of(), HOSTNAME_ID_CHARS, hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected()

### Community 81 - "x-node"
Cohesion: 0.22
Nodes (9): x-node, build, cap_drop, image, read_only, restart, security_opt, stop_grace_period (+1 more)

### Community 82 - "SpaceTable"
Cohesion: 0.18
Nodes (8): leaf_hash(), ref_indices(), SpaceError, BadIndex, BadLeaves, Io, SpaceTable, tree_offsets()

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

### Community 86 - "compilerOptions"
Cohesion: 0.15
Nodes (12): compilerOptions, module, moduleResolution, noUncheckedIndexedAccess, noUnusedLocals, skipLibCheck, strict, target (+4 more)

### Community 87 - "Core"
Cohesion: 0.12
Nodes (25): Core, address_get(), address_qr(), apply_protection(), AppSink, connection_state(), contact_add(), contact_list() (+17 more)

### Community 88 - "Step"
Cohesion: 0.40
Nodes (4): Step, Frame, Line, Quit

### Community 89 - "super"
Cohesion: 0.47
Nodes (3): anchors(), deterministic_and_bounded(), weight()

### Community 90 - "path"
Cohesion: 0.16
Nodes (8): Settings, ThemeChoice, Dark, Light, System, LoadMeter, measure(), Measured

### Community 91 - "ancord"
Cohesion: 0.25
Nodes (7): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura

### Community 92 - "screens.test.ts"
Cohesion: 0.43
Nodes (6): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start()

### Community 95 - "run"
Cohesion: 0.18
Nodes (15): load_user(), Cli, conversation_files(), load_push(), load_session(), now(), online_session(), OnlineCtx (+7 more)

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.29
Nodes (6): ancord klient (Android a desktop), Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14), Síť

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 100 - "WorkerError"
Cohesion: 0.18
Nodes (4): WorkerError, Api, Protocol, Transport

### Community 106 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (26): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+18 more)

### Community 109 - "ancord-node/src/main.rs"
Cohesion: 0.27
Nodes (5): NodeError, main(), resolve_worker_key(), run(), shutdown_signal()

### Community 112 - "UserKeys"
Cohesion: 0.29
Nodes (3): safety(), push_ids_are_stable_and_bound_to_the_mailbox(), UserKeys

### Community 113 - "cpu.rs"
Cohesion: 0.53
Nodes (5): compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start()

### Community 116 - "config.ts"
Cohesion: 0.24
Nodes (11): base, cache, DeepPartial, isObject(), loadConfig(), merge(), ProfileName, PROFILES (+3 more)

### Community 118 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

### Community 120 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

## Knowledge Gaps
- **126 isolated node(s):** `@tauri-apps/cli`, `@tsconfig/svelte`, `@types/node`, `jsdom`, `svelte-check` (+121 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 880 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **19 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `13. Klient (Android a desktop)`, `b64uEncode`, `push.test.ts`?**
  _High betweenness centrality (0.243) - this node is a cross-community bridge._
- **What connects `@tauri-apps/cli`, `@tsconfig/svelte`, `@types/node` to the rest of the system?**
  _126 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `challenge.test.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.10465116279069768 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `13. Klient (Android a desktop)`, `Melo v2 – architektura`, `ancord node`, `pairwise.py`?**
  _High betweenness centrality (0.171) - this node is a cross-community bridge._
- **Should `src-tauri/src/store.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05742296918767507 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `challenge.test.ts`, `scheduler.ts`, `push.test.ts`, `index.ts`, `vectors.test.ts`, `snapshot.ts`, `ws.ts`, `push_id`, `registry.ts`, `handler.ts`?**
  _High betweenness centrality (0.154) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.12896405919661733 - nodes in this community are weakly interconnected._