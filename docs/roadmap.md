# Cairn roadmap

**Direction:** help coding agents remember useful project context across sessions without requiring users to manage memory routinely.

This is a sequence of product outcomes, not a release schedule. Alpha.9 is the current published release; [CHANGELOG](../CHANGELOG.md) records what shipped. A checked item means that foundation exists, while each milestone's exit still requires the [validation evidence](validation.md). Engineering tasks and defects belong in [GitHub issues](https://github.com/cunilab/Cairn/issues).

## M0 — Foundation

**Outcome:** Cairn has the technical foundation for durable agent memory.

- [x] Safe structured capture and durable delivery
- [x] Server-owned knowledge, evidence, and retrieval
- [x] Claude Code and Codex native integration foundations
- [x] Personal and team knowledge with governance
- [x] Thin SQLite edge, PostgreSQL authority, and web control plane

**Exit:** the foundation has shipped and its releases are recorded in the changelog.

## M1 — Works end to end

**Outcome:** a new user can deploy Cairn, connect an authorized repository, work, return later, and use remembered context.

- [x] Documented same-origin deployment and browser project provisioning
- [x] Repository matching, membership checks, and `cairn setup`
- [x] Agent activity reaches inspectable memory and later recall
- [x] Release source, archives, containers, and deployment identity agree
- [ ] Recheck the full first-use and return journey for each supported release and primary agent
- [ ] Make all setup and runtime failures actionable in that journey

**Exit:** a fresh user completes deployment → project → setup → work → return → useful recall without undocumented fixes. Alpha.9 supplied candidate and published smoke evidence for major parts of this journey; a new candidate must prove it again.

[Current candidate evidence](../evals/m1-m2/results.md) supports the local first-use and return journey. Fresh published images, HTTPS Compose deployment, and other native platforms remain unrun for this candidate; the full M1 exit is open.

## M2 — Recall can be trusted

**Outcome:** Cairn returns useful, supported context and handles irrelevant, stale, conflicting, and unauthorized information honestly.

- [ ] Evaluate realistic work across repositories and later sessions
- [ ] Preserve useful decisions, procedures, and failed approaches
- [ ] Surface uncertainty and conflict without choosing unsupported truth
- [ ] Keep project and account boundaries intact
- [ ] Measure useful, irrelevant, and harmful recall; improve from measured failures

**Exit:** paired evidence shows Cairn helps later work and does not create unacceptable harmful recall.

The [current candidate](../evals/m1-m2/results.md) completed 30 paired tasks across three repositories and both primary agents. Capture occurred in 14/15 natural cases, but useful-recall, completion, delivered-claim quality, harmful-recall, and repeated-investigation gates are not established. Execution alone does not close M2.

## M3 — Safe and recoverable

**Outcome:** interruptions do not lose accepted work, corrupt identity, or leave operators guessing.

- [ ] Prove offline, retry, restart, saturation, and delivery-state behavior
- [ ] Prove credential revocation and cross-project/session isolation
- [ ] Prove upgrades, physical backup and restore, and recoverable deployment defaults
- [ ] Make health and recovery actions understandable

**Exit:** common failures can be diagnosed and recovered without inspecting Cairn internals.

## M4 — Everyday product

**Outcome:** normal operation can be understood and managed through supported UI and agent flows.

- [ ] Complete Memory and Session browsing, search, and stable pagination
- [ ] Explain why a memory exists and allow correction, supersession, or retirement
- [ ] Show integration health and clear pending, success, and error states
- [ ] Make account and member management usable on desktop, mobile, and keyboard

**Exit:** normal work needs no raw UUIDs, database access, or undocumented API calls.

## M5 — Stable 0.1

**Outcome:** Cairn has a documented operating envelope and a release candidate ready for promotion.

- [ ] Measure supported scale and define retention/deletion behavior
- [ ] Verify supported OS, browser, agent, and installation matrix
- [ ] Define upgrade compatibility and rehearse release and rollback
- [ ] Complete a real-world pilot and document limitations

**Exit:** a validated release candidate is ready for v0.1.0.

## Later

Additional integrations, semantic retrieval, embeddings or vector storage, richer analytics, hosted service, larger-team administration, and a broader human CLI need product evidence before entering this sequence.
