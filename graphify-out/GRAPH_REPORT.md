# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 185 files · ~147,314 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 47 file(s) not represented in the graph (top: .toml 18, .xml 11, (none) 9)

## Summary
- 2820 nodes · 7476 edges · 117 communities (94 shown, 23 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 414 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `b156ebea`
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
- CliError
- BuildTask.kt
- data.rs
- ratchet.py
- channel.rs
- package.json
- cloudflare.ts
- registry.ts
- channel.py
- vectors.rs
- registration.ts
- App.svelte
- space.rs
- worker.rs
- backend.ts
- ancord-core/src/net.rs
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
- b64u_encode
- b64uEncode
- core.rs
- NodeSocket
- handler.rs
- pairwise.py
- sign.rs
- VirtualList.svelte
- ancord-core/src/guard.rs
- ws.ts
- NodeState
- peer.rs
- scoring.ts
- push_id
- app.svelte.ts
- b64u_decode_array
- android_sync.rs
- contract.rs
- UiEvent
- .contact_add
- local-forward.py
- WorkerError
- src/e2e.rs
- ProofError
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
- .with
- vectors.test.ts
- node1
- node2
- Onboarding.svelte
- node3
- Shared
- node4
- node5
- node6
- pathbuf
- x-node
- sha256
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh
- compilerOptions
- Core
- MainActivity
- super
- path
- ancord
- screens.test.ts
- RO
- run
- ancord klient (Android a desktop)
- default.json
- build.rs
- Egress
- ancord-client
- Melo v2 – architektura
- three_nodes_register_challenge_route_and_admit
- WorkerClient
- ancord-node/src/main.rs
- MockCloudflareClient
- config.ts
- ancord node
- ancord Worker
- CloudflareClient

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

## Communities (117 total, 23 thin omitted)

### Community 0 - "13. Klient (Android a desktop)"
Cohesion: 0.13
Nodes (17): 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.14 Milníky fáze D, 13.15 Mimo rozsah v1 klienta, 13.1 Cíle a pořadí priorit, 13.2 Volba technologie (+9 more)

### Community 1 - "src-tauri/src/store.rs"
Cohesion: 0.05
Nodes (47): FAST, FAST, auto(), AutoDeviceKey, CONTEXT, device_mode_reports_no_passphrase(), DeviceKey, Env (+39 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.12
Nodes (48): ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth(), Dim, DIM_COL (+40 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (47): vitest, createCloudflareClient(), sharedMockClient(), loadConfig(), validate(), withOverrides(), Domain, resetRouteCache() (+39 more)

### Community 4 - "index.ts"
Cohesion: 0.10
Nodes (54): verifyNodeRequest(), clock, workerSigner(), ErrorCode, errorResponse(), HttpError, STATUS, unwrap() (+46 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "CliError"
Cohesion: 0.08
Nodes (30): StorePersist, update_message(), create_user(), load_session(), lock_file(), save_session(), save_user(), write_private() (+22 more)

### Community 8 - "BuildTask.kt"
Cohesion: 0.13
Nodes (3): BuildTask, Config, RustPlugin

### Community 9 - "data.rs"
Cohesion: 0.07
Nodes (49): ack(), ack_local(), anchors_in(), blocking(), check_ts(), DataError, BadRequest, InsufficientNodes (+41 more)

### Community 10 - "ratchet.py"
Cohesion: 0.17
Nodes (18): 9. Otevřené body, b64u(), bootstrap_seed(), dh(), header_aad(), header_key0(), header_nonce(), hkdf() (+10 more)

### Community 11 - "channel.rs"
Cohesion: 0.07
Nodes (38): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, channel_roundtrip_detects_reorder_replay_and_tamper(), ChannelError, BadPublicKey, Decrypt, Exhausted (+30 more)

### Community 12 - "package.json"
Cohesion: 0.05
Nodes (34): dependencies, @tauri-apps/api, devDependencies, jsdom, svelte, svelte-check, @sveltejs/vite-plugin-svelte, @tauri-apps/cli (+26 more)

### Community 13 - "cloudflare.ts"
Cohesion: 0.16
Nodes (10): apply, cf(), Envelope, main(), ApiEnvelope, isManagedHostname(), MANAGED_COMMENT, MockCall (+2 more)

### Community 14 - "registry.ts"
Cohesion: 0.12
Nodes (12): Result, toResult(), SignedDirectory, AuthCtx, BeginInput, BeginResult, ConfirmInput, ConfirmResult (+4 more)

### Community 15 - "channel.py"
Cohesion: 0.22
Nodes (13): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+5 more)

### Community 16 - "vectors.rs"
Cohesion: 0.06
Nodes (47): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value(), b64u_decode() (+39 more)

### Community 17 - "registration.ts"
Cohesion: 0.16
Nodes (22): Config, genesisSet(), randomBytes(), requiredBits(), timeoutSignal(), log(), RegCtx, Candidate (+14 more)

### Community 18 - "App.svelte"
Cohesion: 0.13
Nodes (10): hasMain, { title, onback, hideBackOnWide = false, right }, address, busy, name, address, copied, qr (+2 more)

### Community 19 - "space.rs"
Cohesion: 0.07
Nodes (31): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+23 more)

### Community 20 - "worker.rs"
Cohesion: 0.15
Nodes (4): BeginResponse, ConfirmResponse, nonce_b64(), signed_header()

### Community 21 - "backend.ts"
Cohesion: 0.13
Nodes (21): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+13 more)

### Community 22 - "ancord-core/src/net.rs"
Cohesion: 0.09
Nodes (19): sync_once(), open(), Channel, Accepted, admission(), admit(), admit_unchecked(), AllFailed (+11 more)

### Community 23 - "tests/e2e.rs"
Cohesion: 0.04
Nodes (14): main(), PUSH_FILE, Step, Frame, Line, Quit, MAX_SUBSCRIPTIONS, ADMIN (+6 more)

### Community 24 - "n2n.rs"
Cohesion: 0.26
Nodes (14): call(), call_once(), Established, Refused, Session, Exchange, Done, UnknownSession (+6 more)

### Community 25 - "handler.ts"
Cohesion: 0.07
Nodes (36): NodeAuth, aesGcmDecrypt(), aesGcmEncrypt(), Env, PushJob, queue(), AdmitOptions, AdmitResult (+28 more)

### Community 26 - "user.rs"
Cohesion: 0.07
Nodes (44): safety(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), both_sides_agree_and_the_one_time_prekey_is_consumed(), dh(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), ENVELOPE_HEAD, envelope_key(), ENVELOPE_VERSION (+36 more)

### Community 27 - "client.rs"
Cohesion: 0.11
Nodes (32): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+24 more)

### Community 28 - "register.rs"
Cohesion: 0.16
Nodes (5): hostname_matches(), register(), Registration, start(), Tunnel

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.06
Nodes (56): node_id_of(), HOSTNAME_ID_CHARS, hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected(), anchors_of() (+48 more)

### Community 31 - "Cmd"
Cohesion: 0.09
Nodes (22): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Online (+14 more)

### Community 32 - "src-tauri/src/net.rs"
Cohesion: 0.10
Nodes (15): Cmd, Flush, Typing, Link, MAX_BACKOFF_S, NetConfig, PING_EVERY, run() (+7 more)

### Community 33 - "ancord-node/src/store.rs"
Cohesion: 0.06
Nodes (38): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+30 more)

### Community 34 - "b64u_encode"
Cohesion: 0.09
Nodes (10): heartbeat(), KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run() (+2 more)

### Community 35 - "b64uEncode"
Cohesion: 0.10
Nodes (28): canonicalBytes(), blobKey(), BwDir, ensureBlob(), signBwToken(), verifyBwToken(), hmacSha256(), Signer (+20 more)

### Community 36 - "core.rs"
Cohesion: 0.10
Nodes (11): ContactRec, conv_id(), IDENTITY_KEY, MAX_TEXT, msg_key(), now_ms(), OutItem, put_message() (+3 more)

### Community 37 - "NodeSocket"
Cohesion: 0.13
Nodes (24): b64u_encode(), ack_inbox(), Connection, lookup_profile(), message_status(), NodeSocket, now_s(), process_inbox() (+16 more)

### Community 38 - "handler.rs"
Cohesion: 0.25
Nodes (13): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, space_commit() (+5 more)

### Community 39 - "pairwise.py"
Cohesion: 0.14
Nodes (9): b64u(), hkdf(), mailbox(), profile(), raw(), sha(), solve(), x3dh() (+1 more)

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

### Community 44 - "ws.ts"
Cohesion: 0.15
Nodes (26): broadcastDirectory(), DirectoryBody, DirectoryNode, eligibleForDirectory(), ensureDirectory(), epochOf(), readDirectory(), excludedIds() (+18 more)

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (32): unix_now(), ConnGuard, SubscriptionGuard, required_bits(), handoff_if_needed(), purge_expired(), run(), add_subscriber() (+24 more)

### Community 46 - "peer.rs"
Cohesion: 0.12
Nodes (28): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+20 more)

### Community 47 - "scoring.ts"
Cohesion: 0.27
Nodes (10): capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf(), trustOf(), utilOf() (+2 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (48): 13.4 Jádro: crate `ancord-core`, 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu, 10.2 `POST /wake` (+40 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.20
Nodes (22): addContact(), boot(), createIdentity(), deleteConversation(), enterReady(), go(), loadOlder(), lockNow() (+14 more)

### Community 50 - "b64u_decode_array"
Cohesion: 0.22
Nodes (4): MAX_OTKS, pow_nonce(), signed_payload(), TS_SKEW_S

### Community 51 - "android_sync.rs"
Cohesion: 0.27
Nodes (3): Java_cz_melonet_ancord_SyncReceiver_nativeSync(), LogSink, run()

### Community 52 - "contract.rs"
Cohesion: 0.17
Nodes (14): Contact, ConversationPage, ConversationSummary, DeliveryState, Delivered, Pending, Queued, Unknown (+6 more)

### Community 53 - "UiEvent"
Cohesion: 0.14
Nodes (11): ConnectionState, Connecting, Offline, Online, UiEvent, ConnectionState, ContactRequest, DeliveryState (+3 more)

### Community 54 - ".contact_add"
Cohesion: 0.16
Nodes (9): contacts_reject_garbage_self_and_duplicates(), identity_contacts_and_settings_survive_a_restart(), memory_core_cannot_send(), offline_messages_are_queued_paged_and_survive_a_restart(), qr_is_an_svg(), ready(), ready_keys(), safety_number_is_the_same_on_both_sides() (+1 more)

### Community 56 - "WorkerError"
Cohesion: 0.20
Nodes (4): WorkerError, Api, Protocol, Transport

### Community 57 - "src/e2e.rs"
Cohesion: 0.17
Nodes (12): client(), free_port(), is_loopback_port(), LocalNet, NODES, online(), received(), repo() (+4 more)

### Community 58 - "ProofError"
Cohesion: 0.19
Nodes (8): ProofError, BadRequest, InvalidProof, is_for(), parse_pk(), optional_push_id(), parse_push_id(), PUSH_ID_LEN

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 60 - "ancord-node/src/profile.rs"
Cohesion: 0.09
Nodes (19): local.sh script, Guard, Profile, profile_id(), from_record(), handoff(), put(), put_local() (+11 more)

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

### Community 69 - ".with"
Cohesion: 0.19
Nodes (6): identity_is_created_once_and_address_has_the_scheme(), lock_err(), locked_core_exposes_nothing(), now(), passphrase_profile_stays_locked_until_the_right_passphrase(), address_uri()

### Community 70 - "vectors.test.ts"
Cohesion: 0.05
Nodes (72): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+64 more)

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
Cohesion: 0.23
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
Cohesion: 0.15
Nodes (16): load_user(), Cli, conversation_files(), label_of(), load_push(), load_session(), now(), online_session() (+8 more)

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.29
Nodes (6): ancord klient (Android a desktop), Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14), Síť

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 106 - "Melo v2 – architektura"
Cohesion: 0.07
Nodes (26): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+18 more)

### Community 110 - "three_nodes_register_challenge_route_and_admit"
Cohesion: 0.14
Nodes (13): address(), Event, Message, Typing, next_event(), next_pushed(), receive(), Received (+5 more)

### Community 111 - "WorkerClient"
Cohesion: 0.32
Nodes (4): activate(), run(), backoff(), WorkerClient

### Community 112 - "ancord-node/src/main.rs"
Cohesion: 0.27
Nodes (5): NodeError, main(), resolve_worker_key(), run(), shutdown_signal()

### Community 116 - "config.ts"
Cohesion: 0.22
Nodes (9): base, cache, ChallengeKind, DeepPartial, isObject(), merge(), ProfileName, PROFILES (+1 more)

### Community 118 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

### Community 120 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

## Knowledge Gaps
- **126 isolated node(s):** `@tauri-apps/cli`, `@tsconfig/svelte`, `@types/node`, `jsdom`, `svelte-check` (+121 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 878 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **23 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `13. Klient (Android a desktop)`, `b64uEncode`, `push.test.ts`?**
  _High betweenness centrality (0.236) - this node is a cross-community bridge._
- **What connects `@tauri-apps/cli`, `@tsconfig/svelte`, `@types/node` to the rest of the system?**
  _126 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `13. Klient (Android a desktop)` be split into smaller, more focused modules?**
  _Cohesion score 0.1323529411764706 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `13. Klient (Android a desktop)`, `pairwise.py`, `Melo v2 – architektura`, `channel.py`, `ancord node`?**
  _High betweenness centrality (0.174) - this node is a cross-community bridge._
- **Should `src-tauri/src/store.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05362614913176711 - nodes in this community are weakly interconnected._
- **Why does `b64uEncode()` connect `b64uEncode` to `scheduler.ts`, `push.test.ts`, `index.ts`, `vectors.test.ts`, `ws.ts`, `push_id`, `registration.ts`, `handler.ts`?**
  _High betweenness centrality (0.155) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.11538461538461539 - nodes in this community are weakly interconnected._