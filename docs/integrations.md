# Agent integrations

The current human command is `cairn setup` in an authorized Git repository. It installs machine-facing `cairn hook` and `cairn mcp` adapters where supported; those hidden commands are not administration tools. [README](../README.md#deploy-and-connect) gives the setup inputs. The old `connect`, `agents`, `doctor`, `repair`, and `disconnect` CLI commands in alpha.7 guides do not exist in alpha.9.

## What each agent can do

| Agent | Current integration boundary |
| --- | --- |
| Claude Code | Native hooks and MCP support lifecycle capture and context delivery. Installed capabilities still require observation before Cairn reports them as verified on that installation. |
| Codex | Native hooks and MCP support capture and session-open context, including re-entry after compaction. Hook execution depends on repository trust; untrusted or unobserved behavior must be reported as pending rather than automatic. |
| OpenCode | MCP and supported hooks are installed, with explicit capability limits. The vendor has no session-end signal; some failure and compaction behavior is conditional, and Cairn does not claim automatic prompt-time or post-compaction delivery. |
| Generic MCP | Manual use of the five MCP tools only. Cairn does not install hooks or claim automatic capture, session boundaries, or context delivery for an arbitrary client. |

The five tools are `cairn_context`, `cairn_search`, `cairn_remember`, `cairn_session`, and `cairn_handoff`. Session and handoff calls are manual recovery controls for native integrations and the manual path for generic MCP. A tool's presence does not mean every lifecycle action is automatic. The running integration's reported capability and health are the source for its actual observed state.

## Project memory reuse

Ordinary context, search, graph, pins, and warnings reuse project records only
when they have an active capture attestation and no observed invalidation.
Archival records remain available through explicit `purpose: "inspect"`; their
presence is not support for acting on a claim. Capture accountability does not
create objective verification. Personal and team knowledge retain their existing
authority rules.

`cairn_remember` sends attested project captures through a distinct daemon
operation and server command. Older components reject these operations instead
of accepting a write without its support. A queued command is only accepted for
delivery; an older server's refusal remains visible in delivery status.
REST producers use `POST /api/projects/{id}/captures` for capture and
`POST /api/memories/{id}/captures` for replacement, with a required
`capture_attestation`. Current legacy memory routes refuse capture fields;
their archival compatibility does not establish an older server's support.
Ordinary CLI reads and native context deliveries use a distinct `project_reuse`
envelope, which older daemons reject before processing the inner operation.

An optional dependency must be eligible project memory with no dependency of
its own (one hop maximum). Revision changes, conflicts, and lifecycle changes
withhold dependent claims. Logical export/import preserves capture authorship,
support, dependency revisions, and invalidation state; credentials are excluded.
Forgetting a memory erases its authored support and source references as well
as its content. Importing a bundle with captures into a server held below schema
9 is refused before any write; upgrade and retry the same bundle.
After successful command delivery, the edge retains receipt identifiers and
status but discards the authored command body. Pending and unsuccessful bodies
remain available for delivery or retry. Existing delivered bodies are scrubbed
when local schema 16 is applied; this does not erase saved backups or transcripts.

## Ownership and setup

Setup requires a server URL, token, authorized account, and repository remote matching an existing project. It cannot create an account, membership, or access grant. It installs only supported resources for detected agents: MCP configuration, lifecycle hooks, an instruction block, and an embedded skill where applicable. The MCP invocation carries no server token; the daemon retains credentials in its protected local store.

Cairn records ownership of the exact resources and bytes it writes. Running `cairn setup` again repairs missing or byte-matching Cairn-owned content, while unrelated settings and user-edited conflicting content are preserved and reported. A clone with committed instruction files is not an installation on another machine; each developer runs setup with their own credential. Configuration managers such as CC Switch may own their own entries, and Cairn must not silently take them over.

Direct setup pins MCP and native Claude Code/Codex hook commands to the `cairn`
executable that performed setup. Moving the archive and rerunning setup updates
only registrations that still match Cairn's ownership record. Portable manager
exports retain executable lookup through PATH. OpenCode's plugin subprocesses
also still use PATH; its MCP command is pinned.

Before upgrading a running older daemon, stop its `cairnd` process in Activity
Monitor, Task Manager, or the operating system's equivalent, and wait for the
process to exit. Install the matching `cairn`/`cairnd` archive and rerun setup.
There is no public `cairn daemon stop` command. Setup checks daemon compatibility
before requesting integration writes and reports an error when the daemon
cannot confirm executable identity.

The public alpha.9 CLI has no general dry-run or disconnect command. Do not use retired alpha.7 instructions for removal. Where removal is supported through an integration flow, it must use recorded ownership and preserve user edits; otherwise inspect and remove the specific local Cairn entry with the owning agent's configuration tool. [Product](product.md) states the intended safe-removal outcome; [validation](validation.md) requires evidence for advertised support.

## Failure states

An unsupported agent version, disabled repository trust, missing hook, edited owned resource, invalid token, wrong project, or unreachable server must be explained separately. An offline cache may contain labelled, finite-age context; it is not a fresh search result or evidence that access remains valid. Agent hooks must finish within their configured budget and must never invent acceptance or delivery when they time out.
