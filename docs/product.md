# Cairn product

Cairn helps coding agents resume useful work across sessions without asking developers to repeat project decisions, failed approaches, and working procedures. It captures supported, bounded activity during ordinary work and allows explicit remembering when an important decision is not observable. Later sessions receive relevant, attributable guidance; people can inspect and correct it.

## Users and scope

- **Developers** need a later agent session to continue a repository with little routine memory management.
- **Project members** need to inspect evidence, uncertainty, and corrections before relying on shared knowledge.
- **Operators** need to deploy Cairn, grant access, understand failures, and recover data.

The initial audience is solo developers and small self-hosted teams. Claude Code and Codex are the primary native journeys. OpenCode and generic MCP clients have narrower, explicitly reported capabilities. Project continuity is the main job; personal and team knowledge support it.

## Core workflow

1. An operator deploys the service, creates an account and project, and grants membership.
2. A developer runs `cairn setup` in a repository whose remote matches that project. Setup verifies access and installs supported agent integration without replacing user-owned configuration.
3. Safe agent events are queued locally and delivered to the server. Supported knowledge is consolidated there; an agent or member may explicitly remember a decision.
4. A later authorized session receives bounded context or an honest empty, cached, denied, or unavailable result. A member can inspect origin and correct stale knowledge.

First value means a later task uses relevant prior context. Creating a record alone does not establish value.

## Product requirements

These nine outcomes retain the current product contract. They are acceptance obligations, not claims that every capability is complete in the current release.

| ID | Outcome and acceptance boundary |
| --- | --- |
| PRD-01 | **Start without guesswork.** A fresh operator and developer can deploy, provision access, connect a matching repository, inspect accepted knowledge, and receive later recall through documented steps. Invalid or nonmember matches do not alter foreign state. |
| PRD-02 | **Work with the existing agent.** Setup preserves unrelated and user-edited configuration, supports safe rerun, and reports only demonstrated agent capabilities. Conflicts and unsupported automation are visible. |
| PRD-03 | **Remember useful work safely.** Supported decisions, failures, and procedures gain attribution and evidence; benign work invents no fact. Explicit memory follows the same privacy and evidence rules. Unsafe raw material is refused without exposure. |
| PRD-04 | **Recover interrupted work.** Admitted operations retain actor, project, and stable identity through retry; the server records one canonical effect. Pending, refused, and lost work are reported accurately. |
| PRD-05 | **Trust and correct memory.** Origin, evidence, uncertainty, conflict, and correction history are inspectable. Authorization separates project membership, personal ownership, and team governance. Reuse never automatically verifies a claim in another context. |
| PRD-06 | **Resume with relevant context.** Later callers receive authorized, bounded guidance within a configured budget. Simultaneous sessions never borrow each other's identity; stale cache shows its age; absence, outage, and delivery are distinct. |
| PRD-07 | **Understand what happened.** People can inspect memory, sessions, handoffs, and delivery state, then take supported corrective action. Lists disclose bounds and interfaces explain loading, pending, success, refusal, and failure. |
| PRD-08 | **Keep knowledge through change.** Upgrade, transfer, and restore preserve accepted data and references without widening access. Legacy records receive explicit dispositions. Logical transfer excludes credentials and is separate from physical backup. |
| PRD-09 | **Rely on advertised support.** Installed binaries, integration advice, platform claims, operating limits, and release evidence agree. Missing verification is reported as a limitation. |

## Invariants

- PostgreSQL is the canonical shared authority. The local edge retains delivery and bounded cache state, never an independent knowledge truth.
- Credentials determine the actor. Similar Git remotes, paths, applicability facts, or a request body's account field cannot grant access.
- Raw prompts, transcripts, diffs, command output, credentials, and unbounded payloads do not enter safe memory transfer. Free-text fields are screened; their existence alone does not make unsafe content structurally impossible.
- An applicability fact determines where knowledge may be relevant. It is separate from a knowledge record's `topic_key` and never grants authorization.
- Accepted work keeps identity through retries. Local admission, server acceptance, actual context delivery, and an agent's use of context are different facts.
- Unavailable or conflicting evidence is shown honestly. Cairn does not invent memories, observation IDs, delivery receipts, or certainty.
- Cairn failure must not block coding indefinitely; bounded delay, explicit queue limits, and recoverable failure are part of the product.

## Non-goals

Cairn is not a full conversation archive, reasoning reconstruction engine, task manager, self-registration service, hosted SaaS, or autonomous judge of truth. It does not promise useful memory from every session, full offline shared authority, or new vector/model infrastructure without measured need.

## Success criteria

Evaluate complete first-use and returning-session journeys, then paired realistic tasks with and without Cairn. Measure useful recall, irrelevant recall, harmful recall, repeated investigation, privacy/authorization failures, and recoverability. A release cannot claim success from passing unit tests or record creation alone. [Validation](validation.md) owns the proof; [roadmap](roadmap.md) owns future outcomes.
