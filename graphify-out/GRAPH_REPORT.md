# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 176 files · ~130,114 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 30 file(s) not represented in the graph (top: .toml 18, (none) 6, .xml 2)

## Summary
- 2757 nodes · 7413 edges · 112 communities (92 shown, 20 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 410 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ad14d2af`
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
- space.rs
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
- sign.rs
- handler.rs
- json
- ticket.rs
- VirtualList.svelte
- core.rs
- snapshot.ts
- NodeState
- peer.rs
- push/store.ts
- push_id
- app.svelte.ts
- ws.ts
- DirStore
- contract.rs
- UiEvent
- .with
- local-forward.py
- ws.ts
- src/e2e.rs
- Profile
- Šest nodů v Dockeru (C3)
- b64u_encode
- ancord-node/src/lib.rs
- canonical.rs
- two_clients_chat_live_offline_and_without_duplicates
- services
- Chat.svelte
- tauri.conf.json
- options
- NODE_NAME
- .contact_add
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
- online_session
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh
- compilerOptions
- Core
- run
- super
- path
- ancord
- screens.test.ts
- RO
- scoring.ts
- user.rs
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
- eventually
- .drop

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 78 edges
3. `DataError` - 59 edges
4. `Core` - 56 edges
5. `Session` - 54 edges
6. `b64uEncode()` - 54 edges
7. `unix_now()` - 53 edges
8. `b64uDecodeLen()` - 45 edges
9. `HttpError` - 45 edges
10. `NodeSocket` - 44 edges

## Surprising Connections (you probably didn't know these)
- `5.4 Push notifikace` --references--> `push_id()`  [INFERRED]
  ARCHITECTURE.md → spec/reference/pairwise.py
- `12.4 Pořadí prací a milníky` --references--> `CloudflareClient`  [INFERRED]
  ARCHITECTURE.md → worker/src/cloudflare.ts
- `1.3 Podpisy a doménová separace` --references--> `Domain`  [INFERRED]
  spec/PROTOCOL.md → worker/src/crypto.ts
- `12.2 Pravidla práce` --references--> `unwrap()`  [INFERRED]
  ARCHITECTURE.md → worker/src/errors.ts
- `Konvence` --references--> `unwrap()`  [INFERRED]
  CLAUDE.md → worker/src/errors.ts

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

## Communities (112 total, 20 thin omitted)

### Community 0 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (32): 10. Profily dev/prod a bootstrap, 11. Fáze, 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.14 Milníky fáze D, 13.15 Mimo rozsah v1 klienta (+24 more)

### Community 1 - "src-tauri/src/store.rs"
Cohesion: 0.06
Nodes (50): FAST, FAST, auto(), AutoDeviceKey, CONTEXT, device_mode_reports_no_passphrase(), DeviceKey, Env (+42 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.14
Nodes (38): ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth(), Dim, DIM_COL (+30 more)

### Community 3 - "push.test.ts"
Cohesion: 0.08
Nodes (55): vitest, createCloudflareClient(), sharedMockClient(), base, cache, ChallengeKind, DeepPartial, isObject() (+47 more)

### Community 4 - "index.ts"
Cohesion: 0.10
Nodes (53): verifyNodeRequest(), clock, workerSigner(), ErrorCode, errorResponse(), HttpError, STATUS, unwrap() (+45 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "Session"
Cohesion: 0.10
Nodes (29): StorePersist, update_message(), create_user(), load_session(), load_user(), lock_file(), save_session(), save_user() (+21 more)

### Community 8 - "ancord-cli/src/main.rs"
Cohesion: 0.10
Nodes (14): Cli, Route, conversation_files(), load_push(), load_session(), main(), now(), OnlineCtx (+6 more)

### Community 9 - "data.rs"
Cohesion: 0.05
Nodes (72): profile_id(), ack(), ack_local(), anchors_in(), blocking(), check_ts(), DataError, BadRequest (+64 more)

### Community 10 - "ratchet.py"
Cohesion: 0.17
Nodes (18): 13.2 Ratchet (klient ↔ klient), b64u(), bootstrap_seed(), dh(), header_aad(), header_key0(), header_nonce(), hkdf() (+10 more)

### Community 11 - "channel.rs"
Cohesion: 0.06
Nodes (38): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, channel_roundtrip_detects_reorder_replay_and_tamper(), ChannelError, BadPublicKey, Decrypt, Exhausted (+30 more)

### Community 12 - "package.json"
Cohesion: 0.07
Nodes (34): dependencies, @tauri-apps/api@^2.12.4, devDependencies, jsdom@^30.1.3, svelte@^5.57.3, svelte-check@^4.7.7, @sveltejs/vite-plugin-svelte@^7.3.1, @tauri-apps/cli@^2.12.4 (+26 more)

### Community 13 - "RealCloudflareClient"
Cohesion: 0.09
Nodes (12): apply, cf(), Envelope, main(), ApiEnvelope, CloudflareClient, isManagedHostname(), MANAGED_COMMENT (+4 more)

### Community 14 - "Registry"
Cohesion: 0.11
Nodes (13): Result, toResult(), SignedDirectory, AuthCtx, BeginInput, BeginResult, ConfirmInput, ConfirmResult (+5 more)

### Community 15 - "b64uDecodeLen"
Cohesion: 0.15
Nodes (16): Signer, verifyKey(), verifySig(), b64uDecode(), b64uDecodeLen(), fromUtf8(), issueTicket(), sortKeys() (+8 more)

### Community 16 - "vectors.rs"
Cohesion: 0.26
Nodes (24): b64u_decode(), b64u_decode_array(), verify_ticket(), arr(), bob_profile_push_id(), canonical(), channel_client_node(), channel_node_node() (+16 more)

### Community 17 - "registration.ts"
Cohesion: 0.14
Nodes (26): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu), genesisSet() (+18 more)

### Community 18 - "App.svelte"
Cohesion: 0.10
Nodes (16): hasMain, ThemeChoice, { title, onback, hideBackOnWide = false, right }, Name, { name, size = 22 }, paths, confirmDelete, contact (+8 more)

### Community 19 - "space.rs"
Cohesion: 0.07
Nodes (32): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+24 more)

### Community 20 - "WorkerClient"
Cohesion: 0.08
Nodes (12): WorkerError, Api, Protocol, Transport, hostname_matches(), register(), Registration, BeginResponse (+4 more)

### Community 21 - "backend.ts"
Cohesion: 0.13
Nodes (21): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+13 more)

### Community 22 - "ancord-core/src/net.rs"
Cohesion: 0.09
Nodes (32): Channel, Accepted, address(), admission(), admit(), admit_unchecked(), Connection, connect_any() (+24 more)

### Community 23 - "tests/e2e.rs"
Cohesion: 0.07
Nodes (9): ADMIN, admin_nodes(), forward(), free_port(), NodeProc, Proc, spawn_node(), worker_dir() (+1 more)

### Community 24 - "n2n.rs"
Cohesion: 0.21
Nodes (17): call(), call_once(), Established, Refused, Session, Exchange, exchange(), Done (+9 more)

### Community 25 - "b64uEncode"
Cohesion: 0.13
Nodes (19): requestPayload(), blobKey(), BwDir, ensureBlob(), signBwToken(), verifyBwToken(), hmacSha256(), b64uEncode() (+11 more)

### Community 26 - "user.rs"
Cohesion: 0.06
Nodes (45): safety(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), both_sides_agree_and_the_one_time_prekey_is_consumed(), dh(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION (+37 more)

### Community 27 - "client.rs"
Cohesion: 0.08
Nodes (34): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+26 more)

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.06
Nodes (57): unix_now(), HOSTNAME_ID_CHARS, hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected(), activate() (+49 more)

### Community 31 - "Cmd"
Cohesion: 0.10
Nodes (20): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Publish (+12 more)

### Community 32 - "src-tauri/src/net.rs"
Cohesion: 0.10
Nodes (15): session_conv(), Cmd, Flush, Typing, Link, MAX_BACKOFF_S, NetConfig, PING_EVERY (+7 more)

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.08
Nodes (35): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+27 more)

### Community 34 - "control.rs"
Cohesion: 0.08
Nodes (15): heartbeat(), KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run() (+7 more)

### Community 35 - "TestNode"
Cohesion: 0.17
Nodes (7): RFC-8785, canonicalBytes(), canonicalJson(), deletePush(), registerPush(), nowS(), TestNode

### Community 36 - "pairwise.py"
Cohesion: 0.11
Nodes (13): ContactRec, identity_is_created_once_and_address_has_the_scheme(), IDENTITY_KEY, MAX_TEXT, msg_key(), Network, now(), now_ms() (+5 more)

### Community 37 - "sign.rs"
Cohesion: 0.12
Nodes (17): auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_HEARTBEAT, DOMAIN_N2N_RESP, DOMAIN_PROOF, DOMAIN_PUSH_REG, DOMAIN_REGISTER (+9 more)

### Community 38 - "handler.rs"
Cohesion: 0.30
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 39 - "json"
Cohesion: 0.09
Nodes (24): 13.4 Jádro: crate `ancord-core`, b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame() (+16 more)

### Community 40 - "ticket.rs"
Cohesion: 0.14
Nodes (14): DOMAIN_DIRECTORY, DOMAIN_TICKET, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody, TicketError, BadSignature (+6 more)

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
Cohesion: 0.15
Nodes (9): Guard, MAX_BITS, payload(), stamp(), STAMP_LEN, stamp_nonce(), stamp_valid(), TS_SKEW_S (+1 more)

### Community 44 - "snapshot.ts"
Cohesion: 0.14
Nodes (24): Config, timeoutSignal(), RegCtx, DirectoryBody, DirectoryNode, eligibleForDirectory(), Candidate, excludedIds() (+16 more)

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (32): handoff_if_needed(), purge_expired(), run(), InSessions, add_subscriber(), after_store(), announce(), coalesce() (+24 more)

### Community 46 - "peer.rs"
Cohesion: 0.14
Nodes (27): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+19 more)

### Community 47 - "push/store.ts"
Cohesion: 0.11
Nodes (27): NodeAuth, aesGcmDecrypt(), aesGcmEncrypt(), Env, PushJob, queue(), AdmitOptions, AdmitResult (+19 more)

### Community 48 - "push_id"
Cohesion: 0.06
Nodes (56): 13.8 Push (Android), 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), ancord node, Bezpečnostní poznámky (+48 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.20
Nodes (22): addContact(), boot(), createIdentity(), deleteConversation(), enterReady(), go(), loadOlder(), lockNow() (+14 more)

### Community 50 - "ws.ts"
Cohesion: 0.15
Nodes (21): b64u_encode(), ack_inbox(), lookup_profile(), NodeSocket, now_s(), process_inbox(), publish_guard(), publish_profile() (+13 more)

### Community 51 - "DirStore"
Cohesion: 0.16
Nodes (6): DirStore, label_of(), Step, Frame, Line, Quit

### Community 52 - "contract.rs"
Cohesion: 0.15
Nodes (14): Contact, ConversationPage, ConversationSummary, DeliveryState, Delivered, Pending, Queued, Unknown (+6 more)

### Community 53 - "UiEvent"
Cohesion: 0.14
Nodes (12): ConnectionState, Connecting, Offline, Online, UiEvent, ConnectionState, ContactRequest, DeliveryState (+4 more)

### Community 56 - "ws.ts"
Cohesion: 0.20
Nodes (17): Json, ensureSchedule(), liveConnectedNodes(), slotsOf(), startNext(), broadcastDirectory(), ensureDirectory(), epochOf() (+9 more)

### Community 57 - "src/e2e.rs"
Cohesion: 0.29
Nodes (6): NODES, online(), received(), state_of(), texts(), wait()

### Community 58 - "Profile"
Cohesion: 0.12
Nodes (14): ProofError, BadRequest, InvalidProof, is_for(), MAX_OTKS, parse_pk(), pow_nonce(), Profile (+6 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 61 - "ancord-node/src/lib.rs"
Cohesion: 0.18
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 62 - "canonical.rs"
Cohesion: 0.30
Nodes (7): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value()

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

### Community 69 - ".contact_add"
Cohesion: 0.25
Nodes (10): contacts_reject_garbage_self_and_duplicates(), conv_id(), identity_contacts_and_settings_survive_a_restart(), memory_core_cannot_send(), offline_messages_are_queued_paged_and_survive_a_restart(), qr_is_an_svg(), ready(), safety_number_is_the_same_on_both_sides() (+2 more)

### Community 70 - "b64uEncode"
Cohesion: 0.06
Nodes (68): keyOf(), main(), out, PKCS8, seedOf(), write(), advance(), advanceNative() (+60 more)

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
Cohesion: 0.15
Nodes (7): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S

### Community 76 - "Shared"
Cohesion: 0.20
Nodes (3): Awaiting, Inner, Shared

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

### Community 82 - "online_session"
Cohesion: 0.25
Nodes (4): Online, online_session(), valid_name(), next_online_message()

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

### Community 86 - "compilerOptions"
Cohesion: 0.15
Nodes (12): compilerOptions, module, moduleResolution, noUncheckedIndexedAccess, noUnusedLocals, skipLibCheck, strict, target (+4 more)

### Community 87 - "Core"
Cohesion: 0.12
Nodes (25): Core, address_get(), address_qr(), apply_protection(), AppSink, connection_state(), contact_add(), contact_list() (+17 more)

### Community 88 - "run"
Cohesion: 0.24
Nodes (3): lock_err(), locked_core_exposes_nothing(), passphrase_profile_stays_locked_until_the_right_passphrase()

### Community 89 - "super"
Cohesion: 0.18
Nodes (4): base32_encode(), anchors(), deterministic_and_bounded(), weight()

### Community 90 - "path"
Cohesion: 0.27
Nodes (3): LoadMeter, measure(), Measured

### Community 91 - "ancord"
Cohesion: 0.25
Nodes (7): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura

### Community 92 - "screens.test.ts"
Cohesion: 0.43
Nodes (6): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start()

### Community 94 - "scoring.ts"
Cohesion: 0.36
Nodes (8): capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf(), trustOf(), utilOf()

### Community 95 - "user.rs"
Cohesion: 0.21
Nodes (8): leading_zero_bits(), solve(), valid(), compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start()

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

## Knowledge Gaps
- **118 isolated node(s):** `ancord-client`, `None`, `Locked`, `Ready`, `Queued` (+113 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 822 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **20 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `b64uEncode`, `push.test.ts`, `TestNode`, `json`?**
  _High betweenness centrality (0.275) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `NodeState` (e.g. with `OutSessions` and `Reach`) actually correct?**
  _`NodeState` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ancord-client`, `None`, `Locked` to the rest of the system?**
  _118 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Melo v2 – architektura` be split into smaller, more focused modules?**
  _Cohesion score 0.06628787878787878 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `ratchet.py`, `5. Control plane (Cloudflare Worker, TypeScript)`, `json`?**
  _High betweenness centrality (0.209) - this node is a cross-community bridge._
- **Should `src-tauri/src/store.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.059609730018711574 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `scheduler.ts`, `push.test.ts`, `index.ts`, `TestNode`, `b64uEncode`, `snapshot.ts`, `push/store.ts`, `b64uDecodeLen`, `registration.ts`, `push_id`, `ws.ts`?**
  _High betweenness centrality (0.172) - this node is a cross-community bridge._