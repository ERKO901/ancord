# Graph Report - ancord  (2026-10-25)

## Corpus Check
- 64 files · ~42,439 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 5 file(s) not represented in the graph (top: (none) 4, .jsonc 1)

## Summary
- 646 nodes · 2106 edges · 27 communities (26 shown, 1 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 59 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

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
- vectors.test.ts
- space.ts
- handler.ts
- gen-vectors.ts
- compilerOptions
- registry.ts
- utf8
- directory.ts
- Registry
- b64uEncode
- node.ts
- config.ts
- encoding.ts
- scoring.ts
- routing.ts
- ancord Worker
- cpu.ts
- context.ts

## God Nodes (most connected - your core abstractions)
1. `b64uEncode()` - 54 edges
2. `b64uDecodeLen()` - 45 edges
3. `HttpError` - 45 edges
4. `utf8()` - 40 edges
5. `Registry` - 31 edges
6. `run()` - 28 edges
7. `concat()` - 27 edges
8. `one()` - 25 edges
9. `main()` - 24 edges
10. `TestNode` - 24 edges

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
- None detected.

## Communities (27 total, 1 thin omitted)

### Community 0 - "Melo v2 – architektura"
Cohesion: 0.06
Nodes (31): 10. Profily dev/prod a bootstrap, 11. Fáze, 12.1 Struktura projektu, 12.2 Pravidla práce, 12.3 Co musí dodat uživatel, 12.4 Pořadí prací a milníky, 12.5 Vývojové prostředí (`dev/`), 12. Pokyny pro implementaci (pro Claude Code) (+23 more)

### Community 1 - "ancord"
Cohesion: 0.29
Nodes (6): ancord, Cloudflare (firemní zóna melonet.cz), Konvence, Postup, Příkazy, Struktura

### Community 2 - "scheduler.ts"
Cohesion: 0.07
Nodes (80): signBwToken(), advanceNative(), verifyCpu(), ALLOWED_COLUMNS, bwAuthorize(), bwRecord, ChallengeRow, challengeTick() (+72 more)

### Community 3 - "push.test.ts"
Cohesion: 0.10
Nodes (46): vitest, sharedMockClient(), Domain, fromHex(), resetRouteCache(), allMeasured(), node(), open (+38 more)

### Community 4 - "index.ts"
Cohesion: 0.11
Nodes (51): verifyNodeRequest(), verifyBwToken(), clock, hmacSha256(), workerSigner(), b64uDecodeLen(), ErrorCode, errorResponse() (+43 more)

### Community 5 - "ancord – protokol (verze 1)"
Cohesion: 0.06
Nodes (35): 10.1 Registrace tokenu, 10.2 `POST /wake`, 10. Push, 11. Konfigurační parametry, 12. Testovací vektory, 1.1 Kódování, 1.2 Kanonické JSON, 1.3 Podpisy a doménová separace (+27 more)

### Community 6 - "push/store.ts"
Cohesion: 0.12
Nodes (22): aesGcmDecrypt(), aesGcmEncrypt(), queue(), processPushBatch(), b64urJson(), CachedToken, createPushSender(), LogPushSender (+14 more)

### Community 7 - "cloudflare.ts"
Cohesion: 0.10
Nodes (8): ApiEnvelope, CloudflareClient, isManagedHostname(), MANAGED_COMMENT, MockCall, MockCloudflareClient, RealCloudflareClient, TUNNEL_NAME_PREFIX

### Community 8 - "package.json"
Cohesion: 0.09
Nodes (26): @cloudflare/vitest-plugin, @cloudflare/workers-types, @noble/hashes, tsx, typescript, wrangler, dependencies, @noble/hashes@^2.4.0 (+18 more)

### Community 9 - "vectors.test.ts"
Cohesion: 0.13
Nodes (7): leadingZeroBits(), compareBytes(), u64le(), hashcashValid(), solveHashcash(), anchorsFor(), DOMAIN

### Community 10 - "space.ts"
Cohesion: 0.19
Nodes (19): buildTable(), buildTree(), D0, decodeBytes(), decodePath(), depthOf(), DI, leafHash() (+11 more)

### Community 11 - "handler.ts"
Cohesion: 0.16
Nodes (14): NodeAuth, Env, PushJob, AdmitOptions, AdmitResult, RateLimiter, pushDeps, readDirectory() (+6 more)

### Community 12 - "gen-vectors.ts"
Cohesion: 0.18
Nodes (14): keyOf(), main(), out, PKCS8, seedOf(), write(), requestPayload(), hostnameOf() (+6 more)

### Community 13 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, exactOptionalPropertyTypes, isolatedModules, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+10 more)

### Community 14 - "registry.ts"
Cohesion: 0.19
Nodes (12): createCloudflareClient(), loadConfig(), Result, toResult(), AuthCtx, BeginInput, BeginResult, ConfirmInput (+4 more)

### Community 15 - "utf8"
Cohesion: 0.25
Nodes (4): signingInput(), concat(), utf8(), TestNode

### Community 16 - "directory.ts"
Cohesion: 0.20
Nodes (15): broadcastDirectory(), DirectoryBody, DirectoryNode, eligibleForDirectory(), buildSnapshotNodes(), KV_SNAPSHOT, publishSnapshot(), round3() (+7 more)

### Community 18 - "b64uEncode"
Cohesion: 0.17
Nodes (12): blobKey(), BwDir, ensureBlob(), ED25519, HOSTNAME_ID_CHARS, PKCS8_ED25519_PREFIX, PUBKEY_LEN, SIG_LEN (+4 more)

### Community 19 - "node.ts"
Cohesion: 0.21
Nodes (11): RFC-8785, canonicalBytes(), canonicalJson(), Json, LEAF_BYTES, issueTicket(), deletePush(), registerPush() (+3 more)

### Community 20 - "config.ts"
Cohesion: 0.21
Nodes (11): base, cache, ChallengeKind, DeepPartial, isObject(), merge(), ProfileName, PROFILES (+3 more)

### Community 21 - "encoding.ts"
Cohesion: 0.24
Nodes (10): B64U_LOOKUP, b64uDecode(), decoder, encoder, fromUtf8(), sortKeys(), TicketBody, TicketCheck (+2 more)

### Community 22 - "scoring.ts"
Cohesion: 0.39
Nodes (7): capacityOf(), isLive(), parseLoad(), Score, scoreOf(), trustOf(), utilOf()

### Community 23 - "routing.ts"
Cohesion: 0.33
Nodes (8): ClientGeo, distanceKm(), haversineKm(), pickP2C(), selectNodes(), Snapshot, SnapshotNode, weightedIndex()

### Community 24 - "ancord Worker"
Cohesion: 0.25
Nodes (7): ancord Worker, Co je potřeba před nasazením (fáze C), Konfigurace, Příkazy, Správa (`Authorization: Bearer $ADMIN_TOKEN`), Struktura, Tajemství (`wrangler secret put`, nikdy do gitu)

### Community 25 - "cpu.ts"
Cohesion: 0.36
Nodes (6): advance(), computeCheckpoints(), CpuParams, CpuVerdict, laneStart(), u32le()

### Community 26 - "context.ts"
Cohesion: 0.47
Nodes (3): Config, Signer, RegCtx

## Knowledge Gaps
- **1 isolated node(s):** `RFC-8785`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 179 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `b64uEncode()` connect `b64uEncode` to `scheduler.ts`, `push.test.ts`, `index.ts`, `ancord – protokol (verze 1)`, `push/store.ts`, `vectors.test.ts`, `space.ts`, `gen-vectors.ts`, `utf8`, `directory.ts`, `node.ts`, `encoding.ts`, `cpu.ts`?**
  _High betweenness centrality (0.111) - this node is a cross-community bridge._
- **What connects `RFC-8785` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Melo v2 – architektura` be split into smaller, more focused modules?**
  _Cohesion score 0.0625 - nodes in this community are weakly interconnected._
- **Why does `wake()` connect `ancord – protokol (verze 1)` to `b64uEncode`, `push.test.ts`, `utf8`?**
  _High betweenness centrality (0.077) - this node is a cross-community bridge._
- **Should `scheduler.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.07166042446941323 - nodes in this community are weakly interconnected._
- **Why does `vitest` connect `push.test.ts` to `package.json`, `vectors.test.ts`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Should `push.test.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.09575012800819252 - nodes in this community are weakly interconnected._