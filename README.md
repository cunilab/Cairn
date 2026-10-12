# Cairn

Persistent, project-aware memory for AI coding agents. Cairn captures safe, bounded activity during coding work and brings relevant decisions, failed approaches, and procedures into later sessions. People can inspect the evidence behind a memory and correct it when circumstances change.

**Current status:** the latest published release is `v0.1.0-alpha.9`; this checkout can include unshipped candidate changes. Cairn is pre-1.0; interfaces and storage may change between releases. Claude Code and Codex are the primary native agent paths. [Integration capabilities](docs/integrations.md) describe OpenCode and generic MCP limits. [Roadmap](docs/roadmap.md) describes future outcomes separately from shipped behavior.

## How it works

```text
agent hooks / MCP → local cairnd + SQLite delivery spool → cairn-server + PostgreSQL
                                                          ↑
                                                  web control plane
```

The server owns canonical knowledge, evidence, sessions, governance, retrieval, and retry receipts. The local daemon queues safe work, correlates callers, and holds a finite context cache; it is not an independent knowledge database. The web app provisions accounts and projects and lets members inspect memory and session history. [Architecture](docs/architecture.md) explains the flow, privacy boundary, and source limits.

## Deploy and connect

The supplied Compose stack serves web at `/` and API at `/api` on one origin:

```bash
cp deploy/.env.example deploy/.env
# Edit deploy/.env: PostgreSQL password, public origin, administrator credentials.
docker compose -f deploy/docker-compose.yml up -d
```

Only the proxy port is public in this example. Put TLS in front of it before exposing it outside a trusted network. The operator creates the project and grants membership in web **Settings**, then creates an API token. A repository must have a matching Git remote. From that repository:

```bash
CAIRN_SERVER_URL=https://cairn.example.com \
CAIRN_SERVER_TOKEN="$CAIRN_INSTALL_TOKEN" \
cairn setup
```

`setup` verifies the credential and membership, binds the project, installs supported agent resources, and starts `cairnd`. Rerun it to repair Cairn-owned bytes; user-edited conflicting resources are reported and preserved. It cannot grant access or create an account. Headless setup accepts protected JSON on stdin with `server_url`, `server_token`, and optional `account_id` and `web_url`. [Integrations](docs/integrations.md) explains what each agent actually supplies.

When upgrading, stop the older `cairnd` process using your operating system's
process manager and wait for it to exit before running setup with the new `cairn` and matching `cairnd`
archive. Setup reports an actionable error if the running daemon cannot confirm
that generated integrations use the current executable.

When no administrator exists, the deployment environment account named by
`CAIRN_ADMIN_EMAIL` and `CAIRN_ADMIN_PASSWORD` is created or promoted to
`admin` and `active`. Restart does not replace an existing administrator's
password. Whoever can set those variables and restart the server can always
obtain administrator access. Protect the deployment environment accordingly.
PostgreSQL physical backup and restore are operator responsibilities; web
logical import/export transfers data between healthy deployments and excludes
credentials.

## Build and check this checkout

The repository pins its Rust toolchain. Build native binaries and the web app separately:

```bash
cargo build --workspace --release
cd web && npm ci && npm run build
```

Run `cargo test --workspace --all-targets` and `cargo clippy --workspace --all-targets -- -D warnings` for Rust. PostgreSQL suites need `CAIRN_TEST_DATABASE_URL`; [validation](docs/validation.md) describes test tiers, web checks, required release evidence, and limitations. Published archives and container images are on the [releases page](https://github.com/cunilab/Cairn/releases).

## Agent interface

Supported native hooks capture lifecycle and safe structured activity. MCP exposes `cairn_context`, `cairn_search`, `cairn_remember`, `cairn_session`, and `cairn_handoff`. Explicit session and handoff calls are recovery controls for native paths and manual controls for generic MCP. Hidden `cairn hook` and `cairn mcp` commands are installed adapters, not human administration commands.

During an outage, bounded local capture can continue. A full spool reports loss or refusal; cached context shows its age and identity. Search does not pretend to be fresh, and an offline client cannot know about unseen revocation. Raw prompts, transcripts, diffs, command output, credentials, and unbounded payloads do not cross the machine boundary as safe memory.

## Documentation

- [Product](docs/product.md): promise, users, workflow, requirements, and non-goals.
- [Architecture](docs/architecture.md): current components, authority, privacy, recovery, and limits.
- [Architecture decisions](docs/adr/README.md): consequential choices, alternatives, evidence, and pending validation.
- [Roadmap](docs/roadmap.md): product outcomes from foundation to stable 0.1.
- [Integrations](docs/integrations.md): supported agent behavior and setup ownership.
- [Recall inference](docs/inference.md): candidate server configuration and provider data boundary.
- [Validation](docs/validation.md): tests, release proof, and acceptance evidence.
- [M1/M2 candidate evidence](evals/m1-m2/results.md): local journeys, paired recall results, and remaining milestone gates.

[CHANGELOG](CHANGELOG.md) records shipped changes. [SECURITY](SECURITY.md) explains private vulnerability reporting and the current trust boundary. Git tags preserve old implementation contracts and specs.
