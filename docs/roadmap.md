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

[Current candidate evidence](../evals/m1-m2/results.md) records all 17 workflow
jobs, five installed archive journeys and a fresh HTTPS deployment passing for
source `58107d9`. The shared Codex first-use and return journey passed browser
provisioning, ordinary setup, prior capture, persisted ownership, actual later
delivery and independently judged useful application. Claude remains skipped
by user instruction; the original two-agent M1 exit remains open. The single PR
remains draft because M2's claim-quality gate failed.

The subsequent task-query change at `f44448eb` passed all six CI jobs and the
installed local recall regression. Its fresh release/HTTPS journey and full
milestone evaluation have not been repeated; the `58107d9` artifact evidence
does not establish the new source's exit.

## M2 — Recall can be trusted

**Outcome:** Cairn returns useful, supported context and handles irrelevant, stale, conflicting, and unauthorized information honestly.

- [x] Evaluate realistic work across repositories and later sessions
- [ ] Preserve useful decisions, procedures, and failed approaches
- [ ] Surface uncertainty and conflict without choosing unsupported truth
- [x] Keep project and account boundaries intact
- [x] Measure useful, irrelevant, and harmful recall; improve from measured failures

**Exit:** paired evidence shows Cairn helps later work and does not create unacceptable harmful recall.

Task-query development examples passed usefulness (2/2) and delivered claim
quality (10/10), but both captures were clean and did not establish a correction
for mixed records. The candidate's semantic selector passed mechanical checks
and CI. Temporary Groq testing failed attribution in its initial screen and
mixed-clause relevance after one general clarification; this model is not
qualified. [Preserved development evidence](../evals/m1-m2/groq-development-metadata.json)
records those failures.

The authorized local-embedding experiment compared keyword and embedding ranking
on identical automatic sentence units. Embeddings achieved 90.9% claim quality,
but useful retrieval was 50% and one selection lost a necessary condition.
[ADR 0006](adr/0006-local-embedding-experiment.md) preserves the failed advance
criteria. This approach stopped before production integration. M1/M2 thresholds
and full exit requirements remain unchanged. PR #66 remains draft.

The structural finding-capture revision passed mechanical checks, but an unchanged
F1 diagnostic recorded no ordinary capture. [ADR 0008](adr/0008-independent-finding-capture.md)
preserves that failure. A pinned native Stop capability probe supports the bounded
per-turn checkpoint in [ADR 0009](adr/0009-bounded-native-finalization-checkpoint.md).
Its implementation and fresh journey remain under verification; the roadmap and
full milestone thresholds are unchanged.

The [complete v13 Codex run](../evals/m1-m2/regression-v13-metadata.json) has
15 valid comparisons across three repositories and 15 valid calibrated
judgments. Useful application passed at **8/8 (100%)**, but delivered claim
quality failed at **31/42 (73.8%), below the 90% target**. All 11 failed claims
are supported but irrelevant implementation details bundled with useful user
decisions. Three bounded privacy cases observed no leaks; high-impact harm,
completion (treatment 14 versus control 5), and median repeated-investigation
reduction (100% across seven eligible pairs) passed. Findings need separately
attributable capture and independent selection; another wording iteration is
not established as a solution. All prior failures and thresholds remain
preserved; Claude's 15 cases remain explicitly skipped. M2's exit is not
established.

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

Additional integrations, production vector storage, richer analytics, hosted service, larger-team administration, and a broader human CLI need product evidence before entering this sequence. The bounded local-embedding experiment above does not establish production vector support.

The `381eee31` F1 diagnostic persisted three findings but omitted the authored
team requirement; calibrated useful application failed and claim quality was
6/9. Native dispatch/metadata bugs are repaired separately. Semantic
qualification is stopped pending an evaluated capture-completeness design or
an explicitly agreed scope change; mechanical repairs do not close M2.
