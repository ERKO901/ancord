# ancord

**A decentralized network for end-to-end encrypted messaging.** Anyone can run a node, nobody has to trust them, and their resources are measured. Cloudflare acts only as a control plane (registration, node verification, load balancing, push notifications). Messages and client connections go straight to the nodes.

> **Status: pre-alpha.** The Worker, node and client are implemented and tested locally; nothing is deployed to production yet. The protocol and formats are not stable, and the project has not had an independent security audit. Do not use it for sensitive communication.

## Principles

- **Open network:** trust comes from measurement (CPU, disk space, bandwidth, peer witnesses), age and reputation, not from the operator's identity.
- **Cloudflare stays out of the data plane:** the Worker tells a client once which node to connect to (`/route`, with a signed ticket). After that the client talks only to the node.
- **Hidden IPs:** every node sits behind its own Cloudflare Tunnel (`node-<id>.<domain>`), so only Cloudflare knows its IP. A node never learns a user's IP.
- **Nodes see only ciphertext** under blinded mailbox identifiers. Messages are replicated to several anchors (HRW hashing per epoch, quorum writes). Encryption is X3DH + Double Ratchet.
- **Secure channel on top of TLS:** a custom handshake for client↔node and node↔node traffic, so Cloudflare cannot see mailbox IDs or tickets after terminating TLS.
- **Parameters live in configuration**, not in code (profiles `dev` / `prod` / `open`).

## Architecture

```
Client ─(1) /route─► Worker (TS) + Durable Object + KV + Queues + R2   [Cloudflare]
   │                       │
   │ (2) top-3 nodes       │ Tunnel + DNS node-<id>.<domain>
   │     + ticket          ▼
   └─(3) WSS + ticket─► Node A ◄── signed requests ──► Node B, Node C   [Rust]
```

Details are in [`ARCHITECTURE.md`](ARCHITECTURE.md) (source of truth), the protocol in [`spec/PROTOCOL.md`](spec/PROTOCOL.md) and the threat model in [`spec/THREAT_MODEL.md`](spec/THREAT_MODEL.md).

## Repository layout

| Folder | Contents |
|---|---|
| [`worker/`](worker/README.md) | Cloudflare Worker (TypeScript): registration, node verification, scoring, routing, push |
| [`node/`](node/README.md) | Node (Rust), the shared `ancord-core` library and the file-based CLI client `ancord-cli` |
| [`client/`](client/README.md) | Graphical client for desktop and Android (Tauri 2 + Svelte 5) |
| [`spec/`](spec) | Protocol specification, threat model, reference implementations and test vectors shared by Worker and node |
| [`dev/`](dev/README.md) | Docker Compose with 6 nodes behind Cloudflare Tunnel, plus scripts for running locally |

The Worker and the node share no code, only the specification and the test vectors in `spec/test-vectors/`.

## Quick start (local, no Cloudflare needed)

Requires Node.js, Rust (stable) and `jq`.

```bash
cd worker && npm install && npm run check   # typecheck + tests (vitest, Miniflare/workerd)
cd ../node && cargo clippy --all-targets -- -D warnings && cargo test
cd ../dev && ./local.sh 3                   # Worker (wrangler dev) and 3 nodes locally
```

`cargo test` includes an end-to-end test that starts `wrangler dev` and 3 nodes (disable with `ANCORD_SKIP_E2E=1`). CLI usage is described in [`node/README.md`](node/README.md), the graphical client in [`client/README.md`](client/README.md).

## Status

| Phase | Scope | State |
|---|---|---|
| A | Worker: registration, challenges, routing, push | implemented, tested locally |
| B | Rust node: data plane, secure channel, presence, push, X3DH + ratchet | implemented, tested locally |
| C | Deployment, DNS, 6 nodes in Docker | prepared, not deployed |
| D | Graphical client (desktop, Android) | in progress (D1–D5 done, D6 Android partial) |

Milestones and acceptance criteria are in [`ARCHITECTURE.md`](ARCHITECTURE.md) §12.4 and §13.14. Out of scope: iOS and a token economy.

> Note: the detailed documentation (`ARCHITECTURE.md`, `spec/`, per-folder READMEs) is currently written in Czech. English translations are planned.

## Security

The design assumes untrusted nodes and treats Cloudflare as an observer. Threats and mitigations are in [`spec/THREAT_MODEL.md`](spec/THREAT_MODEL.md). Please report vulnerabilities privately through GitHub Security Advisories (the repository's *Security* tab), not as public issues.

## Contributing

Pull requests are welcome. Ground rules:

- Spec and test vectors come before implementation; if code and [`ARCHITECTURE.md`](ARCHITECTURE.md) disagree, update the document first.
- Rust: `cargo clippy --all-targets -- -D warnings` and `cargo test` must pass cleanly, with no `unwrap` on production paths.
- TypeScript: `strict` mode, tests with vitest. A protocol format change means running `npm run vectors` and updating the node.
- Never commit secrets (API tokens, tunnel tokens, private keys).

## License

[MIT](LICENSE)
