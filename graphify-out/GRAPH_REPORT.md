# Graph Report - ancord  (2026-10-25)

## Corpus Check
- 106 files · ~64,998 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 4 file(s) not represented in the graph (top: (none) 3, .jsonc 1)

## Summary
- 1327 nodes · 3758 edges · 61 communities (56 shown, 5 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 160 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8cf63d33`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Melo v2 – architektura
- ancord
- scheduler.ts
- push.test.ts
- index.ts
- ancord – protokol (verze 1)
- push/store.ts
- cloudflare.ts
- package.json
- data.rs
- gen-vectors.ts
- handler.ts
- ratchet.rs
- compilerOptions
- registry.ts
- utf8
- b64u_decode_array
- registration.ts
- crypto.ts
- space.rs
- WorkerError
- vectors.test.ts
- ancord-cli/src/lib.rs
- e2e.rs
- ancord Worker
- errors.ts
- server.rs
- client.rs
- Identity
- NodeState
- sign.rs
- ancord-cli/src/main.rs
- encoding.ts
- store.rs
- control.rs
- b64u_encode
- canonical.rs
- ancord-node/src/main.rs
- duration
- MockCloudflareClient
- ancord node
- ancord-cli
- encoding.ts
- b64u_encode
- peer.rs
- handler.rs
- ticket.rs
- cpu.rs
- encoding.rs
- Role
- wake.rs
- config.ts
- routing.ts
- context.ts
- HttpError
- sysinfo.rs
- config.rs
- WorkerClient
- scoring.ts

## God Nodes (most connected - your core abstractions)
1. `NodeState` - 78 edges
2. `b64uEncode()` - 55 edges
3. `b64uDecodeLen()` - 45 edges
4. `HttpError` - 45 edges
5. `utf8()` - 42 edges
6. `DataError` - 37 edges
7. `unix_now()` - 37 edges
8. `b64u_encode()` - 33 edges
9. `b64u_decode_array()` - 33 edges
10. `Registry` - 30 edges

## Surprising Connections (you probably didn't know these)
- `1.3 Podpisy a doménová separace` --references--> `Domain`  [INFERRED]
  spec/PROTOCOL.md → worker/src/crypto.ts
- `12.2 Pravidla práce` --references--> `unwrap()`  [INFERRED]
  ARCHITECTURE.md → worker/src/errors.ts
- `Konvence` --references--> `unwrap()`  [INFERRED]
  CLAUDE.md → worker/src/errors.ts
- `12.3 Co musí dodat uživatel` --references--> `node()`  [INFERRED]
  ARCHITECTURE.md → worker/test/challenge.test.ts
- `Nody` --references--> `node()`  [INFERRED]
  ARCHITECTURE.md → worker/test/challenge.test.ts

## Import Cycles
- 1-file cycle: `node/crates/ancord-node/src/challenge/space.rs -> node/crates/ancord-node/src/challenge/space.rs`
- 2-file cycle: `node/crates/ancord-node/src/data.rs -> node/crates/ancord-node/src/wake.rs -> node/crates/ancord-node/src/data.rs`

## Communities (61 total, 5 thin omitted)

### Community 0 - "Melo v2 – architektura"
Cohesion: 0.05
Nodes (32): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+24 more)

### Community 1 - "ancord"
Cohesion: 0.29
Nodes (6): ancord, Cloudflare (firemní zóna melonet.cz), Konvence, Postup, Příkazy, Struktura

### Community 2 - "scheduler.ts"
Cohesion: 0.13
Nodes (41): ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick(), combineBandwidth(), Dim, DIM_COL (+33 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (45): vitest, LEAF_BYTES, leavesFor(), sharedMockClient(), Domain, resetRouteCache(), allMeasured(), node() (+37 more)

### Community 4 - "index.ts"
Cohesion: 0.17
Nodes (28): verifyNodeRequest(), verifyBwToken(), clock, hmacSha256(), workerSigner(), clientInfo(), expandIpv6(), ipPrefix() (+20 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.04
Nodes (44): 10.1 Registrace tokenu, 10.2 `POST /wake`, 10. Push, 11. Konfigurační parametry, 12. Testovací vektory, 13.1 Mailbox a zpráva, 13.2 Ratchet (klient ↔ klient), 13.3 Zprávy klient → node (+36 more)

### Community 6 - "push/store.ts"
Cohesion: 0.11
Nodes (23): aesGcmDecrypt(), aesGcmEncrypt(), fromUtf8(), queue(), processPushBatch(), b64urJson(), CachedToken, createPushSender() (+15 more)

### Community 7 - "cloudflare.ts"
Cohesion: 0.18
Nodes (7): ApiEnvelope, createCloudflareClient(), isManagedHostname(), MANAGED_COMMENT, MockCall, RealCloudflareClient, TUNNEL_NAME_PREFIX

### Community 8 - "package.json"
Cohesion: 0.09
Nodes (26): @cloudflare/vitest-plugin, @cloudflare/workers-types, @noble/hashes, tsx, typescript, wrangler, dependencies, @noble/hashes@^2.4.0 (+18 more)

### Community 9 - "data.rs"
Cohesion: 0.05
Nodes (70): ack(), ack_local(), anchors_in(), blocking(), check_ts(), DataError, BadRequest, InsufficientNodes (+62 more)

### Community 10 - "gen-vectors.ts"
Cohesion: 0.11
Nodes (35): hkdf(), keyOf(), main(), out, PKCS8, ratchetVectors(), seedOf(), write() (+27 more)

### Community 11 - "handler.ts"
Cohesion: 0.16
Nodes (13): NodeAuth, Env, PushJob, AdmitOptions, AdmitResult, RateLimiter, pushDeps, readDirectory() (+5 more)

### Community 12 - "ratchet.rs"
Cohesion: 0.08
Nodes (29): aad(), Direction, InitiatorToResponder, ResponderToInitiator, expand(), forged_message_does_not_advance_state(), gap_limit(), garbage_is_rejected() (+21 more)

### Community 13 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, exactOptionalPropertyTypes, isolatedModules, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+10 more)

### Community 14 - "registry.ts"
Cohesion: 0.14
Nodes (11): Result, toResult(), SignedDirectory, AuthCtx, BeginInput, BeginResult, ConfirmInput, ConfirmResult (+3 more)

### Community 15 - "utf8"
Cohesion: 0.16
Nodes (13): requestPayload(), canonicalBytes(), blobKey(), BwDir, ensureBlob(), signBwToken(), b64uEncode(), utf8() (+5 more)

### Community 16 - "b64u_decode_array"
Cohesion: 0.29
Nodes (18): b64u_decode(), b64u_decode_array(), canonical(), client_auth(), directory(), hashchain(), heartbeat(), hrw() (+10 more)

### Community 17 - "registration.ts"
Cohesion: 0.19
Nodes (19): genesisSet(), requiredBits(), timeoutSignal(), log(), activate(), assertAsnNotPenalized(), assertLimits(), banNode() (+11 more)

### Community 18 - "crypto.ts"
Cohesion: 0.14
Nodes (19): ED25519, HOSTNAME_ID_CHARS, hostnameOf(), leadingZeroBits(), nodeIdOf(), PKCS8_ED25519_PREFIX, PUBKEY_LEN, SIG_LEN (+11 more)

### Community 19 - "space.rs"
Cohesion: 0.10
Nodes (19): Backing, File, Mem, CHUNK, D0, DI, LEAF_BYTES, leaf_hash() (+11 more)

### Community 20 - "WorkerError"
Cohesion: 0.14
Nodes (6): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse

### Community 21 - "vectors.test.ts"
Cohesion: 0.09
Nodes (11): RFC-8785, canonicalJson(), b64uDecode(), randomBytes(), pushIdOf(), sortKeys(), TicketBody, TicketCheck (+3 more)

### Community 22 - "ancord-cli/src/lib.rs"
Cohesion: 0.13
Nodes (24): admission(), admit(), CliError, Admission, AllFailed, Connection, Node, Route (+16 more)

### Community 23 - "e2e.rs"
Cohesion: 0.10
Nodes (13): base32_encode(), ADMIN, admin_nodes(), eventually(), forward(), free_port(), NodeProc, Proc (+5 more)

### Community 24 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Co je potřeba před nasazením (fáze C), Konfigurace, Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 25 - "errors.ts"
Cohesion: 0.33
Nodes (5): ErrorCode, errorResponse(), STATUS, unwrap(), fetch()

### Community 26 - "server.rs"
Cohesion: 0.16
Nodes (7): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S

### Community 27 - "client.rs"
Cohesion: 0.17
Nodes (17): attach_rid(), authenticate(), Bucket, close(), CLOSE_AUTH_TIMEOUT, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED, do_ack() (+9 more)

### Community 28 - "Identity"
Cohesion: 0.18
Nodes (6): hostname_matches(), Identity, activate(), register(), Registration, DOMAIN_REGISTER

### Community 29 - "NodeState"
Cohesion: 0.13
Nodes (5): ConnGuard, handoff_if_needed(), purge_expired(), run(), NodeState

### Community 30 - "sign.rs"
Cohesion: 0.14
Nodes (13): DOMAIN_CLIENT_AUTH, DOMAIN_HEARTBEAT, DOMAIN_PROOF, DOMAIN_REPORT, DOMAIN_REQ, DOMAIN_TICKET, parse_auth_header(), ParsedAuth (+5 more)

### Community 31 - "ancord-cli/src/main.rs"
Cohesion: 0.11
Nodes (13): Cli, Cmd, Connect, Init, Recv, Route, Send, load_session() (+5 more)

### Community 32 - "encoding.ts"
Cohesion: 0.16
Nodes (18): advance(), advanceNative(), computeCheckpoints(), CpuParams, CpuVerdict, laneStart(), verifyCpu(), pickIndices() (+10 more)

### Community 33 - "store.rs"
Cohesion: 0.10
Nodes (25): ack_leaves_tombstone_that_blocks_late_put(), bump(), decode(), delivered_put_upgrades_pending(), encode(), HEADER_LEN, io(), key() (+17 more)

### Community 34 - "control.rs"
Cohesion: 0.18
Nodes (3): heartbeat(), KEEPALIVE, on_directory()

### Community 35 - "b64u_encode"
Cohesion: 0.15
Nodes (6): HOSTNAME_ID_CHARS, hostname_of(), KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected(), node_id_of()

### Community 36 - "canonical.rs"
Cohesion: 0.30
Nodes (7): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value()

### Community 37 - "ancord-node/src/main.rs"
Cohesion: 0.23
Nodes (4): NodeError, main(), resolve_worker_key(), run()

### Community 38 - "duration"
Cohesion: 0.24
Nodes (4): run(), start(), Tunnel, backoff()

### Community 40 - "ancord node"
Cohesion: 0.25
Nodes (7): ancord node, Bezpečnostní poznámky, CLI klient: zprávy, E2E test (B1 až B4), Konfigurace nodu, Příkazy, Stav

### Community 44 - "encoding.ts"
Cohesion: 0.12
Nodes (30): Json, broadcastDirectory(), DirectoryBody, DirectoryNode, eligibleForDirectory(), ensureDirectory(), epochOf(), readDirectory() (+22 more)

### Community 45 - "b64u_encode"
Cohesion: 0.46
Nodes (4): b64u_encode(), auth_header(), nonce_b64(), signed_header()

### Community 46 - "peer.rs"
Cohesion: 0.26
Nodes (11): ack(), anchored_mailbox(), authenticate(), data_error(), error(), get(), MAX_BODY, parse_ids() (+3 more)

### Community 47 - "handler.rs"
Cohesion: 0.30
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 48 - "ticket.rs"
Cohesion: 0.16
Nodes (14): DOMAIN_DIRECTORY, DirectoryBody, DirectoryNode, DirectoryParams, TicketBody, TicketError, BadSignature, Expired (+6 more)

### Community 49 - "cpu.rs"
Cohesion: 0.23
Nodes (8): compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start(), leading_zero_bits(), solve(), valid()

### Community 50 - "encoding.rs"
Cohesion: 0.18
Nodes (3): anchors(), deterministic_and_bounded(), weight()

### Community 51 - "Role"
Cohesion: 0.50
Nodes (3): Role, Initiator, Responder

### Community 52 - "wake.rs"
Cohesion: 0.28
Nodes (5): admit(), optional_push_id(), parse_push_id(), PUSH_ID_LEN, spawn()

### Community 53 - "config.ts"
Cohesion: 0.22
Nodes (12): base, cache, ChallengeKind, DeepPartial, isObject(), loadConfig(), merge(), ProfileName (+4 more)

### Community 54 - "routing.ts"
Cohesion: 0.32
Nodes (7): ClientGeo, distanceKm(), haversineKm(), pickP2C(), Snapshot, SnapshotNode, weightedIndex()

### Community 55 - "context.ts"
Cohesion: 0.38
Nodes (3): Config, Signer, RegCtx

### Community 56 - "HttpError"
Cohesion: 0.26
Nodes (15): HttpError, MAX_BODY_BYTES, optInt(), parseJsonObject(), readBody(), reqB64(), reqInt(), reqObject() (+7 more)

### Community 57 - "sysinfo.rs"
Cohesion: 0.27
Nodes (3): LoadMeter, measure(), Measured

### Community 59 - "WorkerClient"
Cohesion: 0.22
Nodes (6): Outcome, Banned, Reconnect, Reregister, run(), WorkerClient

### Community 60 - "scoring.ts"
Cohesion: 0.31
Nodes (9): tierLeaves(), capacityOf(), isLive(), parseClaimed(), parseLoad(), Score, scoreOf(), trustOf() (+1 more)

## Knowledge Gaps
- **47 isolated node(s):** `ancord-cli`, `Route`, `Admission`, `Node`, `Session` (+42 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 385 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `b64u_encode()` connect `b64u_encode` to `data.rs`, `ratchet.rs`, `b64u_decode_array`, `space.rs`, `ancord-cli/src/lib.rs`, `e2e.rs`, `server.rs`, `client.rs`, `Identity`, `sign.rs`, `ancord-cli/src/main.rs`, `control.rs`, `b64u_encode`, `ancord-node/src/main.rs`, `peer.rs`, `handler.rs`, `cpu.rs`, `encoding.rs`, `wake.rs`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **What connects `ancord-cli`, `Route`, `Admission` to the rest of the system?**
  _47 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Melo v2 – architektura` be split into smaller, more focused modules?**
  _Cohesion score 0.05128205128205128 - nodes in this community are weakly interconnected._
- **Why does `NodeState` connect `NodeState` to `store.rs`, `control.rs`, `duration`, `client.rs`, `data.rs`, `peer.rs`, `handler.rs`, `ticket.rs`, `cpu.rs`, `space.rs`, `wake.rs`, `sysinfo.rs`, `server.rs`, `WorkerClient`, `Identity`?**
  _High betweenness centrality (0.049) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.13333333333333333 - nodes in this community are weakly interconnected._
- **Why does `b64u_decode_array()` connect `b64u_decode_array` to `b64u_encode`, `ancord-node/src/main.rs`, `data.rs`, `b64u_encode`, `handler.rs`, `ticket.rs`, `encoding.rs`, `ancord-cli/src/lib.rs`, `server.rs`, `client.rs`, `Identity`, `sign.rs`, `ancord-cli/src/main.rs`?**
  _High betweenness centrality (0.027) - this node is a cross-community bridge._
- **Should `push.test.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.10109289617486339 - nodes in this community are weakly interconnected._