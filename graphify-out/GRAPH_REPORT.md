# Graph Report - ancord  (2026-10-30)

## Corpus Check
- 173 files · ~124,273 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 30 file(s) not represented in the graph (top: .toml 18, (none) 6, .xml 2)

## Summary
- 2633 nodes · 7002 edges · 106 communities (89 shown, 17 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 397 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c5c3460a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- 12. Pokyny pro implementaci (pro Claude Code)
- src-tauri/src/store.rs
- scheduler.ts
- push.test.ts
- index.ts
- ancord – protokol (verze 1)
- ratchet.rs
- run
- CliError
- data.rs
- ratchet.py
- channel.rs
- package.json
- cloudflare.ts
- registry.ts
- utf8
- vectors.rs
- registration.ts
- App.svelte
- space.rs
- WorkerError
- backend.ts
- net.rs
- e2e.rs
- n2n.rs
- sign.rs
- user.rs
- client.rs
- duration
- deploy.ts
- dataplane.rs
- Cmd
- digest
- store.rs
- control.rs
- b64u_encode
- NoDisk
- WorkerError
- handler.rs
- pairwise.py
- ancord node
- VirtualList.svelte
- ancord-core/src/guard.rs
- ws.ts
- NodeState
- peer.rs
- handler.ts
- push_id
- app.svelte.ts
- config.ts
- eventually
- contract.rs
- scoring.ts
- Step
- local-forward.py
- crypto.ts
- 13. Klient (Android a desktop)
- ancord-core/src/profile.rs
- Šest nodů v Dockeru (C3)
- local.sh
- reach.rs
- Session
- ticket.rs
- services
- Chat.svelte
- tauri.conf.json
- options
- NODE_NAME
- super
- vectors.test.ts
- node1
- node2
- Onboarding.svelte
- node3
- sysinfo.rs
- WorkerClient
- node4
- node5
- node6
- pathbuf
- x-node
- ancord Worker
- scripts/tsconfig.json
- bootstrap.sh
- cleanup.sh
- compilerOptions
- core.rs
- online_session
- ancord-cli/src/main.rs
- canonical.rs
- ancord
- screens.test.ts
- RO
- MockCloudflareClient
- SeedRng
- 4. Skrytí IP
- ancord klient (Android a desktop)
- default.json
- build.rs
- 5. Control plane (Cloudflare Worker, TypeScript)
- Outcome
- ancord-client

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 150 edges
2. `CliError` - 64 edges
3. `DataError` - 59 edges
4. `b64uEncode()` - 54 edges
5. `unix_now()` - 53 edges
6. `Core` - 45 edges
7. `Session` - 45 edges
8. `b64uDecodeLen()` - 45 edges
9. `HttpError` - 45 edges
10. `NodeSocket` - 40 edges

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

## Communities (106 total, 17 thin omitted)

### Community 0 - "12. Pokyny pro implementaci (pro Claude Code)"
Cohesion: 0.33
Nodes (6): 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code)

### Community 1 - "src-tauri/src/store.rs"
Cohesion: 0.06
Nodes (48): FAST, auto(), AutoDeviceKey, CONTEXT, device_mode_reports_no_passphrase(), DeviceKey, Env, env() (+40 more)

### Community 2 - "scheduler.ts"
Cohesion: 0.11
Nodes (50): blobKey(), ensureBlob(), ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth() (+42 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (43): vitest, sharedMockClient(), loadConfig(), validate(), withOverrides(), resetRouteCache(), pushDeps, KV_DIRECTORY (+35 more)

### Community 4 - "index.ts"
Cohesion: 0.10
Nodes (53): verifyNodeRequest(), clock, workerSigner(), errorResponse(), HttpError, unwrap(), clientInfo(), expandIpv6() (+45 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.07
Nodes (26): 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace, 1.4 Chyby, 1. Konvence, 2. Identita nodu (+18 more)

### Community 6 - "ratchet.rs"
Cohesion: 0.07
Nodes (48): aad(), bootstrap_secret(), conversation_heals_after_a_state_leak(), dh(), Direction, InitiatorToResponder, ResponderToInitiator, ENC_HEADER_LEN (+40 more)

### Community 7 - "run"
Cohesion: 0.12
Nodes (18): create_user(), load_session(), load_user(), lock_file(), save_session(), save_user(), write_private(), Cli (+10 more)

### Community 8 - "CliError"
Cohesion: 0.13
Nodes (31): b64u_encode(), ack_inbox(), CliError, Admission, Connection, Node, PowRequired, Route (+23 more)

### Community 9 - "data.rs"
Cohesion: 0.06
Nodes (67): profile_id(), ack(), ack_local(), anchors_in(), blocking(), check_ts(), DataError, BadRequest (+59 more)

### Community 10 - "ratchet.py"
Cohesion: 0.19
Nodes (16): b64u(), bootstrap_seed(), header_aad(), header_key0(), header_nonce(), hkdf(), kdf_rk(), msg_key() (+8 more)

### Community 11 - "channel.rs"
Cohesion: 0.06
Nodes (38): CHAN_FRAME_AAD, CHAN_KEYS_INFO, CHAN_LABEL, channel_roundtrip_detects_reorder_replay_and_tamper(), ChannelError, BadPublicKey, Decrypt, Exhausted (+30 more)

### Community 12 - "package.json"
Cohesion: 0.07
Nodes (34): dependencies, @tauri-apps/api@^2.12.4, devDependencies, jsdom@^30.1.3, svelte@^5.57.3, svelte-check@^4.7.7, @sveltejs/vite-plugin-svelte@^7.3.1, @tauri-apps/cli@^2.12.4 (+26 more)

### Community 13 - "cloudflare.ts"
Cohesion: 0.11
Nodes (11): apply, cf(), Envelope, main(), ApiEnvelope, CloudflareClient, isManagedHostname(), MANAGED_COMMENT (+3 more)

### Community 14 - "registry.ts"
Cohesion: 0.11
Nodes (13): ErrorCode, Result, STATUS, toResult(), SignedDirectory, AuthCtx, BeginInput, BeginResult (+5 more)

### Community 15 - "utf8"
Cohesion: 0.20
Nodes (8): canonicalBytes(), b64uEncode(), issueTicket(), deletePush(), registerPush(), bringUp(), nowS(), TestNode

### Community 16 - "vectors.rs"
Cohesion: 0.28
Nodes (23): b64u_decode(), b64u_decode_array(), arr(), bob_profile_push_id(), canonical(), channel_client_node(), channel_node_node(), channel_padding() (+15 more)

### Community 17 - "registration.ts"
Cohesion: 0.18
Nodes (17): genesisSet(), requiredBits(), timeoutSignal(), probe(), activate(), assertAsnNotPenalized(), assertLimits(), consumeNonce() (+9 more)

### Community 18 - "App.svelte"
Cohesion: 0.10
Nodes (16): hasMain, ThemeChoice, { title, onback, hideBackOnWide = false, right }, Name, { name, size = 22 }, paths, confirmDelete, contact (+8 more)

### Community 19 - "space.rs"
Cohesion: 0.06
Nodes (32): attack(), Attacker, build(), CHAIN, log2_arg(), main(), open_round(), OPENINGS (+24 more)

### Community 20 - "WorkerError"
Cohesion: 0.11
Nodes (8): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse, nonce_b64(), signed_header()

### Community 21 - "backend.ts"
Cohesion: 0.13
Nodes (21): Args, installDevBackend(), Result, b64u(), createMockBackend(), digits(), fakeQr(), MockOptions (+13 more)

### Community 22 - "net.rs"
Cohesion: 0.10
Nodes (24): Channel, Accepted, admission(), admit(), admit_unchecked(), AllFailed, connect_any(), HistoryEntry (+16 more)

### Community 23 - "e2e.rs"
Cohesion: 0.07
Nodes (10): ADMIN, admin_nodes(), forward(), free_port(), NodeProc, Proc, spawn_node(), worker_dir() (+2 more)

### Community 24 - "n2n.rs"
Cohesion: 0.21
Nodes (17): call(), call_once(), Established, Refused, Session, Exchange, exchange(), Done (+9 more)

### Community 25 - "sign.rs"
Cohesion: 0.12
Nodes (17): auth_header(), DOMAIN_CHAN_CLIENT, DOMAIN_CHAN_NODE, DOMAIN_HEARTBEAT, DOMAIN_N2N_RESP, DOMAIN_PROOF, DOMAIN_PUSH_REG, DOMAIN_REGISTER (+9 more)

### Community 26 - "user.rs"
Cohesion: 0.06
Nodes (46): address(), publish_guard(), safety(), addresses_parse_with_or_without_the_scheme_and_reject_garbage(), both_sides_agree_and_the_one_time_prekey_is_consumed(), dh(), each_conversation_gets_a_fresh_secret_and_third_parties_cannot_open(), ENVELOPE_HEAD (+38 more)

### Community 27 - "client.rs"
Cohesion: 0.08
Nodes (34): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_BAD_FRAME, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED (+26 more)

### Community 29 - "deploy.ts"
Cohesion: 0.08
Nodes (21): dir, sizes, apply, blockers, cf(), Envelope, fail(), GENERATED_SECRETS (+13 more)

### Community 30 - "dataplane.rs"
Cohesion: 0.06
Nodes (58): unix_now(), bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S (+50 more)

### Community 31 - "Cmd"
Cohesion: 0.10
Nodes (20): Cmd, Connect, History, Inbox, Init, Keygen, Lookup, Publish (+12 more)

### Community 32 - "digest"
Cohesion: 0.23
Nodes (8): leading_zero_bits(), solve(), valid(), compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start()

### Community 33 - "store.rs"
Cohesion: 0.07
Nodes (39): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), decode_guard(), decode_profile(), delivered_put_upgrades_pending(), encode(), encode_guard() (+31 more)

### Community 34 - "control.rs"
Cohesion: 0.15
Nodes (3): heartbeat(), KEEPALIVE, on_directory()

### Community 35 - "b64u_encode"
Cohesion: 0.12
Nodes (6): HOSTNAME_ID_CHARS, hostname_of(), Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected()

### Community 37 - "WorkerError"
Cohesion: 0.16
Nodes (5): NodeError, main(), resolve_worker_key(), run(), shutdown_signal()

### Community 38 - "handler.rs"
Cohesion: 0.30
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 39 - "pairwise.py"
Cohesion: 0.10
Nodes (22): b64u(), ed_sign(), hchacha20(), qr(), hkdf64(), main(), frame(), pad() (+14 more)

### Community 40 - "ancord node"
Cohesion: 0.18
Nodes (10): ancord node, Bezpečnostní poznámky, CLI klient: běžné použití, CLI klient: konverzace podle veřejného klíče (jednotlivé kroky), CLI klient: zprávy se sdíleným tajemstvím, Docker, E2E test (B1 až B4), Konfigurace nodu (+2 more)

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

### Community 43 - "ancord-core/src/guard.rs"
Cohesion: 0.11
Nodes (9): Guard, MAX_BITS, payload(), stamp(), STAMP_LEN, stamp_nonce(), stamp_valid(), TS_SKEW_S (+1 more)

### Community 44 - "ws.ts"
Cohesion: 0.13
Nodes (30): Json, Config, Signer, RegCtx, broadcastDirectory(), DirectoryBody, DirectoryNode, eligibleForDirectory() (+22 more)

### Community 45 - "NodeState"
Cohesion: 0.07
Nodes (32): handoff_if_needed(), purge_expired(), run(), InSessions, add_subscriber(), after_store(), announce(), coalesce() (+24 more)

### Community 46 - "peer.rs"
Cohesion: 0.14
Nodes (27): Json, ack(), anchored_mailbox(), ApiError, deliver(), directory(), dispatch(), exchange() (+19 more)

### Community 47 - "handler.ts"
Cohesion: 0.12
Nodes (23): aesGcmDecrypt(), aesGcmEncrypt(), PushJob, queue(), processPushBatch(), b64urJson(), CachedToken, createPushSender() (+15 more)

### Community 48 - "push_id"
Cohesion: 0.07
Nodes (49): 13.4 Jádro: crate `ancord-core`, 13.8 Push (Android), 6.1 Anchory, 6.2 Šifrování a transport, 6.3 Online doručení a presence, 6.4 Identita uživatele a funkce konverzace, 6. Data plane (nody, Rust), 10.1 Registrace tokenu (+41 more)

### Community 49 - "app.svelte.ts"
Cohesion: 0.20
Nodes (22): addContact(), boot(), createIdentity(), deleteConversation(), enterReady(), go(), loadOlder(), lockNow() (+14 more)

### Community 50 - "config.ts"
Cohesion: 0.10
Nodes (17): createCloudflareClient(), base, cache, ChallengeKind, DeepPartial, isObject(), merge(), ProfileName (+9 more)

### Community 52 - "contract.rs"
Cohesion: 0.10
Nodes (23): ConnectionState, Connecting, Offline, Online, Contact, ConversationPage, ConversationSummary, DeliveryState (+15 more)

### Community 53 - "scoring.ts"
Cohesion: 0.20
Nodes (13): capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf(), trustOf(), utilOf() (+5 more)

### Community 54 - "Step"
Cohesion: 0.40
Nodes (4): Step, Frame, Line, Quit

### Community 56 - "crypto.ts"
Cohesion: 0.09
Nodes (38): NodeAuth, BwDir, signBwToken(), verifyBwToken(), Domain, ED25519, hmacSha256(), HOSTNAME_ID_CHARS (+30 more)

### Community 57 - "13. Klient (Android a desktop)"
Cohesion: 0.08
Nodes (27): 10. Profily dev/prod a bootstrap, 11. Fáze, 13.10 Platformní specifika, 13.11 Rozpočet výkonu (cíle, měří se), 13.12 Struktura `client/`, 13.13 Testování, 13.14 Milníky fáze D, 13.15 Mimo rozsah v1 klienta (+19 more)

### Community 58 - "ancord-core/src/profile.rs"
Cohesion: 0.14
Nodes (13): ProofError, BadRequest, InvalidProof, is_for(), MAX_OTKS, parse_pk(), pow_nonce(), Profile (+5 more)

### Community 59 - "Šest nodů v Dockeru (C3)"
Cohesion: 0.20
Nodes (9): dev, Failover, Kontrola, Lokální běh bez Dockeru a Cloudflare, Nový node, výměna klíče, Postup, Požadavky na server, Úklid (+1 more)

### Community 61 - "reach.rs"
Cohesion: 0.17
Nodes (5): MAX_REPORTED, PeerHealth, pull_directory(), PULL_MIN_INTERVAL_S, Reach

### Community 62 - "Session"
Cohesion: 0.18
Nodes (14): deliver(), Event, Message, Typing, keepalive_if_due(), next_event(), next_pushed(), now_s() (+6 more)

### Community 63 - "ticket.rs"
Cohesion: 0.14
Nodes (15): DOMAIN_DIRECTORY, DOMAIN_TICKET, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody, TicketError, BadSignature (+7 more)

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

### Community 69 - "super"
Cohesion: 0.14
Nodes (5): base32_encode(), anchors(), deterministic_and_bounded(), weight(), PUSH_ID_LEN

### Community 70 - "vectors.test.ts"
Cohesion: 0.06
Nodes (57): RFC-8785, keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload() (+49 more)

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

### Community 75 - "sysinfo.rs"
Cohesion: 0.27
Nodes (3): LoadMeter, measure(), Measured

### Community 76 - "WorkerClient"
Cohesion: 0.24
Nodes (7): hostname_matches(), activate(), register(), Registration, run(), backoff(), WorkerClient

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

### Community 82 - "ancord Worker"
Cohesion: 0.22
Nodes (8): ancord Worker, Konfigurace, Nasazení (fáze C1 a C2), Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu), log()

### Community 83 - "scripts/tsconfig.json"
Cohesion: 0.29
Nodes (6): ../tsconfig.json, compilerOptions, lib, types, extends, include

### Community 86 - "compilerOptions"
Cohesion: 0.15
Nodes (12): compilerOptions, module, moduleResolution, noUncheckedIndexedAccess, noUnusedLocals, skipLibCheck, strict, target (+4 more)

### Community 87 - "core.rs"
Cohesion: 0.05
Nodes (37): ContactRec, contacts_reject_garbage_self_and_duplicates(), Core, identity_contacts_and_settings_survive_a_restart(), identity_is_created_once_and_address_has_the_scheme(), IDENTITY_KEY, Inner, locked_core_exposes_nothing() (+29 more)

### Community 88 - "online_session"
Cohesion: 0.23
Nodes (5): Online, online_session(), OnlineCtx, valid_name(), next_online_message()

### Community 89 - "ancord-cli/src/main.rs"
Cohesion: 0.07
Nodes (7): Route, load_push(), main(), open(), PUSH_FILE, PushConfig, default_url()

### Community 90 - "canonical.rs"
Cohesion: 0.30
Nodes (7): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value()

### Community 91 - "ancord"
Cohesion: 0.22
Nodes (8): ancord, Cloudflare (firemní zóna melonet.cz), graphify, Konvence, Postup, Příkazy, Struktura, node()

### Community 92 - "screens.test.ts"
Cohesion: 0.43
Nodes (6): disposeListeners(), installMockBackend(), fresh(), click(), settle(), start()

### Community 96 - "4. Skrytí IP"
Cohesion: 0.33
Nodes (6): 4. Skrytí IP, Důsledky, Node bez veřejné IP, Nody, Uživatelé, Závislost na Cloudflare a cena

### Community 97 - "ancord klient (Android a desktop)"
Cohesion: 0.33
Nodes (5): ancord klient (Android a desktop), Architektura UI, Pravidla (§13.5), Příkazy, Stav proti milníkům (§13.14)

### Community 98 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 100 - "5. Control plane (Cloudflare Worker, TypeScript)"
Cohesion: 0.40
Nodes (5): 5.1 Identita a registrace nodu, 5.2 Důkazy zdrojů, 5.3 Load balancing a routing, 5.4 Push notifikace, 5. Control plane (Cloudflare Worker, TypeScript)

### Community 101 - "Outcome"
Cohesion: 0.40
Nodes (5): Outcome, Banned, Reconnect, Reregister, run()

## Knowledge Gaps
- **111 isolated node(s):** `ancord-client`, `None`, `Locked`, `Ready`, `Queued` (+106 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 801 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **17 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `wake()` connect `push_id` to `push.test.ts`, `utf8`?**
  _High betweenness centrality (0.285) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `NodeState` (e.g. with `OutSessions` and `Reach`) actually correct?**
  _`NodeState` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ancord-client`, `None`, `Locked` to the rest of the system?**
  _111 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `src-tauri/src/store.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06229797237731413 - nodes in this community are weakly interconnected._
- **Why does `push_id()` connect `push_id` to `ancord node`, `5. Control plane (Cloudflare Worker, TypeScript)`, `pairwise.py`?**
  _High betweenness centrality (0.181) - this node is a cross-community bridge._
- **Are the 52 inferred relationships involving `unix_now()` (e.g. with `authenticate()` and `do_send()`) actually correct?**
  _`unix_now()` has 52 INFERRED edges - model-reasoned connections that need verification._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.1090146750524109 - nodes in this community are weakly interconnected._