# Graph Report - ancord  (2026-10-25)

## Corpus Check
- 97 files · ~53,460 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 4 file(s) not represented in the graph (top: (none) 3, .jsonc 1)

## Summary
- 1051 nodes · 2971 edges · 44 communities (37 shown, 7 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 113 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d4f9f787`
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
- space.ts
- handler.ts
- gen-vectors.ts
- compilerOptions
- registry.ts
- utf8
- vectors.rs
- registration.ts
- crypto.ts
- space.rs
- WorkerClient
- encoding.ts
- ancord-cli/src/lib.rs
- e2e.rs
- ancord Worker
- cpu.ts
- server.rs
- client.rs
- register.rs
- NodeState
- sign.rs
- ancord-cli/src/main.rs
- handler.rs
- three_nodes_register_challenge_route_and_admit
- control.rs
- b64u_encode
- canonical.rs
- ancord-node/src/main.rs
- duration
- MockCloudflareClient
- ancord node
- ancord-cli

## God Nodes (most connected - your core abstractions)
1. `b64uEncode()` - 54 edges
2. `b64uDecodeLen()` - 45 edges
3. `HttpError` - 45 edges
4. `utf8()` - 40 edges
5. `NodeState` - 38 edges
6. `Registry` - 31 edges
7. `run()` - 28 edges
8. `concat()` - 27 edges
9. `b64u_decode_array()` - 25 edges
10. `one()` - 25 edges

## Surprising Connections (you probably didn't know these)
- `12.4 Pořadí prací a milníky` --references--> `CloudflareClient`  [INFERRED]
  ARCHITECTURE.md → worker/src/cloudflare.ts
- `1.3 Podpisy a doménová separace` --references--> `Domain`  [INFERRED]
  spec/PROTOCOL.md → worker/src/crypto.ts
- `12.2 Pravidla práce` --references--> `unwrap()`  [INFERRED]
  ARCHITECTURE.md → worker/src/errors.ts
- `Konvence` --references--> `unwrap()`  [INFERRED]
  CLAUDE.md → worker/src/errors.ts
- `12.3 Co musí dodat uživatel` --references--> `node()`  [INFERRED]
  ARCHITECTURE.md → worker/test/challenge.test.ts

## Import Cycles
- 1-file cycle: `node/crates/ancord-node/src/challenge/space.rs -> node/crates/ancord-node/src/challenge/space.rs`

## Communities (44 total, 7 thin omitted)

### Community 0 - "Melo v2 – architektura"
Cohesion: 0.06
Nodes (31): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+23 more)

### Community 1 - "ancord"
Cohesion: 0.29
Nodes (6): ancord, Cloudflare (firemní zóna melonet.cz), Konvence, Postup, Příkazy, Struktura

### Community 2 - "scheduler.ts"
Cohesion: 0.06
Nodes (88): RFC-8785, canonicalBytes(), Json, ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick() (+80 more)

### Community 3 - "push.test.ts"
Cohesion: 0.07
Nodes (62): vitest, clock, sharedMockClient(), base, cache, ChallengeKind, DeepPartial, isObject() (+54 more)

### Community 4 - "index.ts"
Cohesion: 0.13
Nodes (44): verifyNodeRequest(), workerSigner(), errorResponse(), HttpError, unwrap(), clientInfo(), expandIpv6(), ipPrefix() (+36 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.05
Nodes (36): 10.1 Registrace tokenu, 10.2 `POST /wake`, 10. Push, 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace (+28 more)

### Community 6 - "push/store.ts"
Cohesion: 0.12
Nodes (21): aesGcmDecrypt(), queue(), processPushBatch(), b64urJson(), CachedToken, createPushSender(), LogPushSender, pemToDer() (+13 more)

### Community 7 - "cloudflare.ts"
Cohesion: 0.12
Nodes (10): ApiEnvelope, CloudflareClient, createCloudflareClient(), isManagedHostname(), MANAGED_COMMENT, MockCall, RealCloudflareClient, TUNNEL_NAME_PREFIX (+2 more)

### Community 8 - "package.json"
Cohesion: 0.09
Nodes (26): @cloudflare/vitest-plugin, @cloudflare/workers-types, @noble/hashes, tsx, typescript, wrangler, dependencies, @noble/hashes@^2.4.0 (+18 more)

### Community 10 - "space.ts"
Cohesion: 0.16
Nodes (21): buildTable(), buildTree(), D0, decodeBytes(), decodePath(), depthOf(), DI, LEAF_BYTES (+13 more)

### Community 11 - "handler.ts"
Cohesion: 0.19
Nodes (9): Config, Env, PushJob, AdmitOptions, AdmitResult, RateLimiter, Cloudflare, Env (+1 more)

### Community 12 - "gen-vectors.ts"
Cohesion: 0.16
Nodes (15): keyOf(), main(), out, PKCS8, seedOf(), write(), NodeAuth, requestPayload() (+7 more)

### Community 13 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, exactOptionalPropertyTypes, isolatedModules, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+10 more)

### Community 14 - "registry.ts"
Cohesion: 0.12
Nodes (13): ErrorCode, Result, STATUS, toResult(), SignedDirectory, AuthCtx, BeginResult, ConfirmInput (+5 more)

### Community 15 - "utf8"
Cohesion: 0.21
Nodes (6): verifyKey(), b64uEncode(), deletePush(), registerPush(), nowS(), TestNode

### Community 16 - "vectors.rs"
Cohesion: 0.06
Nodes (38): b64u_decode(), b64u_decode_array(), base32_encode(), HOSTNAME_ID_CHARS, Identity, KEY_FILE, key_persists_with_private_mode(), loose_permissions_are_rejected() (+30 more)

### Community 17 - "registration.ts"
Cohesion: 0.17
Nodes (24): genesisSet(), verifySig(), b64uDecodeLen(), requiredBits(), log(), activate(), assertAsnNotPenalized(), assertLimits() (+16 more)

### Community 18 - "crypto.ts"
Cohesion: 0.17
Nodes (17): spaceT(), aesGcmEncrypt(), ED25519, HOSTNAME_ID_CHARS, leadingZeroBits(), PKCS8_ED25519_PREFIX, PUBKEY_LEN, SIG_LEN (+9 more)

### Community 19 - "space.rs"
Cohesion: 0.06
Nodes (24): Backing, File, Mem, CHUNK, D0, DI, LEAF_BYTES, leaf_hash() (+16 more)

### Community 20 - "WorkerClient"
Cohesion: 0.11
Nodes (9): WorkerError, Api, Protocol, Transport, BeginResponse, ConfirmResponse, nonce_b64(), signed_header() (+1 more)

### Community 21 - "encoding.ts"
Cohesion: 0.14
Nodes (22): blobKey(), BwDir, ensureBlob(), signBwToken(), verifyBwToken(), hmacSha256(), Signer, B64U_LOOKUP (+14 more)

### Community 22 - "ancord-cli/src/lib.rs"
Cohesion: 0.21
Nodes (12): admission(), admit(), CliError, Admission, AllFailed, Route, connect_any(), fetch_route() (+4 more)

### Community 23 - "e2e.rs"
Cohesion: 0.14
Nodes (4): ADMIN, admin_nodes(), forward(), ZONE

### Community 24 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Co je potřeba před nasazením (fáze C), Konfigurace, Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 25 - "cpu.ts"
Cohesion: 0.26
Nodes (10): advance(), advanceNative(), computeCheckpoints(), CpuParams, CpuVerdict, laneStart(), verifyCpu(), pickIndices() (+2 more)

### Community 26 - "server.rs"
Cohesion: 0.16
Nodes (8): bandwidth_source(), BW_PATH, MAX_WITNESS_BYTES, proof(), PROOF_PATH, router(), SKEW_S, DOMAIN_PROOF

### Community 27 - "client.rs"
Cohesion: 0.20
Nodes (10): authenticate(), close(), CLOSE_AUTH_TIMEOUT, CLOSE_OVERLOADED, CLOSE_UNAUTHORIZED, MAX_MESSAGE_BYTES, reject(), session() (+2 more)

### Community 28 - "register.rs"
Cohesion: 0.17
Nodes (8): hostname_matches(), activate(), register(), Registration, DOMAIN_REGISTER, run(), unix_now(), backoff()

### Community 30 - "sign.rs"
Cohesion: 0.20
Nodes (10): auth_header(), DOMAIN_REPORT, DOMAIN_REQ, parse_auth_header(), ParsedAuth, request_payload(), sign(), signing_input() (+2 more)

### Community 31 - "ancord-cli/src/main.rs"
Cohesion: 0.16
Nodes (7): default_url(), Cli, Cmd, Connect, Route, main(), run()

### Community 32 - "handler.rs"
Cohesion: 0.33
Nodes (14): answer(), bandwidth(), cpu(), MAX_BW_BYTES, MAX_CPU_ITERS, MAX_INDICES, MAX_LANES, Reply (+6 more)

### Community 33 - "three_nodes_register_challenge_route_and_admit"
Cohesion: 0.16
Nodes (9): hostname_of(), eventually(), free_port(), NodeProc, Proc, spawn_node(), three_nodes_register_challenge_route_and_admit(), worker_dir() (+1 more)

### Community 34 - "control.rs"
Cohesion: 0.16
Nodes (8): KEEPALIVE, on_directory(), Outcome, Banned, Reconnect, Reregister, run(), DOMAIN_HEARTBEAT

### Community 35 - "b64u_encode"
Cohesion: 0.22
Nodes (9): compute_checkpoints(), CpuError, BadParams, lane_checkpoints(), lane_start(), b64u_encode(), leading_zero_bits(), solve() (+1 more)

### Community 36 - "canonical.rs"
Cohesion: 0.24
Nodes (8): canonical_bytes(), canonical_json(), CanonicalError, NotSafeInteger, MAX_SAFE, write_string(), write_value(), heartbeat()

### Community 37 - "ancord-node/src/main.rs"
Cohesion: 0.23
Nodes (4): NodeError, main(), resolve_worker_key(), run()

### Community 40 - "ancord node"
Cohesion: 0.29
Nodes (6): ancord node, Bezpečnostní poznámky, E2E test (B1 + B2), Konfigurace nodu, Příkazy, Stav

## Knowledge Gaps
- **21 isolated node(s):** `ancord-cli`, `Route`, `Admission`, `Route`, `Connect` (+16 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 302 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `b64uEncode()` connect `utf8` to `scheduler.ts`, `push.test.ts`, `index.ts`, `ancord – protokol (verze 1)`, `push/store.ts`, `vectors.test.ts`, `space.ts`, `gen-vectors.ts`, `registration.ts`, `crypto.ts`, `encoding.ts`, `cpu.ts`?**
  _High betweenness centrality (0.036) - this node is a cross-community bridge._
- **What connects `ancord-cli`, `Route`, `Admission` to the rest of the system?**
  _21 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Melo v2 – architektura` be split into smaller, more focused modules?**
  _Cohesion score 0.0625 - nodes in this community are weakly interconnected._
- **Why does `wake()` connect `ancord – protokol (verze 1)` to `push.test.ts`, `utf8`?**
  _High betweenness centrality (0.033) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.055166217430368376 - nodes in this community are weakly interconnected._
- **Why does `HttpError` connect `index.ts` to `MockCloudflareClient`, `cloudflare.ts`, `gen-vectors.ts`, `registry.ts`, `registration.ts`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **Should `push.test.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.06851851851851852 - nodes in this community are weakly interconnected._