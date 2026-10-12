# 0007 — Reassess Cairn around the Agentmemory workflow

**Status:** Reassessment complete; implementation recommendations proposed, not qualified or adopted.
**Recorded:** 2026-10-10.

## User intent

The user confirmed that Agentmemory inspired Cairn. The requested additions are
team, branch, global and session memory, plus automatic project capture. The
core outcome remains ordinary coding work becoming useful context in later
sessions without routine human memory management. These additions should extend
that workflow rather than make it dependent on manual recall or a separate
hosted model for every read.

“Global” is provisionally mapped to the existing account-owned personal memory
across projects and authorized server-wide team guidance. The user has not yet
specified whether they instead need organization-specific teams or a different
cross-project sharing policy. Scope applicability must never grant access.

## Reference and comparison

Agentmemory source was inspected at
`da91cc05b3c79c59f6c0480f728bb9c09127e000`. This is a source comparison, not a
fresh installation or independent reproduction of its performance claims.

| Stage | Agentmemory reference | Cairn current candidate | Assessment |
| --- | --- | --- | --- |
| Capture | Hook observations and explicit memory; optional model compression | Safe typed events, deterministic extraction and explicit agent-authored memory | Cairn deliberately retains less raw material; general decisions and rationale cannot be inferred from lifecycle metadata alone. |
| Memory formation | Structured facts/narrative when compression is enabled; explicit content otherwise | Governed records with evidence, account attribution and lifecycle | Strong governance does not make an authored paragraph one complete independently relevant finding. |
| Retrieval | BM25/vector fusion, optional graph and reranking, multiple observations | Scope-aware lexical candidates followed by mandatory configured semantic excerpt selection for matching working recall | Model selection was added to address mixed records; its necessity and semantic reliability remain unproven. |
| Returning context | Several summaries, lessons and observations within a token budget | Native hooks have no task query; project finding bodies are withheld until an explicit query | The agent must reliably issue that query without human intervention; this deserves a direct journey check. |
| Authority | Local runtime memory and its own project/profile controls | Shared PostgreSQL, authenticated membership, thin delivery edge and corrections | Retain Cairn's sharing and authorization design; a reference retrieval implementation cannot substitute for it. |

Agentmemory's published 95.2% `recall_any@5` asks whether any gold session appears
among five results. It explicitly excludes answer generation and judging.
Cairn's M2 evaluates actual delivered claims and later usefulness. Those scores
are not interchangeable. Agentmemory also implements capture retry and memory
versioning; these are not uniquely Cairn features merely because Cairn has them.

The [sentence/top-one experiment](0006-local-embedding-experiment.md) was not a
reproduction of Agentmemory's hybrid, multiple-observation pipeline. Its failure
rejects that tested configuration, not embeddings generally.

## Tracing the measured failure

Read-only inspection of the four failed V13 packets established that each
explicit `cairn_remember` call already combined a useful user decision with
implementation details. The persisted content was identical when transmitted
at session opening, ranked first under `project_reserve`.

| Case | Original capture completion line | Original judge: supported but irrelevant claims | Original judge: useful application |
| --- | --- | --- | --- |
| D2 | 24 | 3 | Yes |
| D4 | 15 | 2 | Yes |
| R3 | 28 | 4 | Yes |
| R5 | 21 | 2 | Yes |

The [boundary audit](../../evals/m1-m2/memory-boundary-audit.json) retains content,
packet and original-judgment hashes. All eight packet/judgment hashes match the
previously published frozen metadata. No workload, judgment or threshold was
changed. Rank/transmission is not proof of model consumption; useful application
is taken from the original independent judgments.

This locates bundling at explicit memory formation, before retrieval. It does
not prove that all capture paths fail, or that splitting existing records would
pass. A single `user_report` attestation also does not independently establish
the inspected-source basis of every implementation claim bundled into it.

On this review date, a fresh minimum-depth `cairn_context` refresh in the working
repository returned `invalid_request: missing field briefing`. Session briefings
have also reported reduced context. This is an observed current integration
failure, separate from V13 and the synthetic experiments. Its underlying client,
daemon or server compatibility cause has not been diagnosed; a test count cannot
substitute for repairing and observing the real call.

## Recommended direction

1. **Keep the requested scopes and shared authority.** Preserve account/project
   boundaries, branch/session applicability, durable delivery, correction history
   and evidence. Prefer existing scope/domain types over new parallel stores.
2. **Make a complete finding the write unit.** A decision should preserve who
   requested it, its conditions and intent. Independent implementation observations
   should have their own authority and source revision. A finding may need several
   sentences; automatic sentence splitting is not a semantic guarantee. Investigate
   the existing explicit-memory contract and deterministic extraction before
   adding another extraction service.
3. **Separate activity capture from semantic memory capture.** Safe hooks can
   record supported activity. Rich decisions and failed approaches need a bounded
   agent-authored finding or a separately designed extraction path. Determine how
   the existing integration triggers that capture without asking the human to
   manage memory. Do not silently relax the raw-material privacy invariant.
4. **Assemble several complete findings within the existing budget.** Preserve
   dependencies and distinguish decisions from implementation status. Diagnose
   retrieval coverage separately from assembly precision. Local embeddings are a
   possible ranking improvement, conditional on evidence from the actual pipeline.
5. **Reassess the compulsory inference dependency.** A model may remain useful
   for selection or optional extraction, but another provider is not established
   as the next root fix. Removing the existing selector is also unqualified; it
   would restore the measured mixed-record failure unless its replacement works.

These are proposed implementation changes. ADR 0004 remains the current candidate
architecture, not a claim of semantic success. No production code, data, model
configuration or deployment changed in this reassessment.

## Validation and next implementation boundary

The next design must describe the automatic journey end to end: work → safe
capture → complete finding → authorized scope selection → assembled context →
later use. Identify changes to the MCP write contract, installed agent guidance,
hook/query behavior, server persistence/retrieval and compatibility before coding.
Choose one bounded change from that map rather than another provider trial loop.
Check installed client/daemon/server identities and restore the failing context
call before claiming the automatic returning-session journey is healthy.

Keep the existing M1/M2 gates and failed evidence. A new implementation needs
fresh source-bound evidence; the four historical records are diagnostic cases,
not a new holdout. Measure candidate retrieval, delivered claim quality and actual
agent usefulness separately. Top-five retrieval success cannot replace claim
quality; neither drilldown nor references containing substantive claims are
exempt from delivery accounting. Claude remains skipped by user instruction.

## Sources

- [Cairn product](../product.md), [current architecture](../architecture.md),
  [original V13 evidence](../../evals/m1-m2/regression-v13-metadata.json).
- [Agentmemory capture](https://github.com/rohitg00/agentmemory/blob/da91cc05b3c79c59f6c0480f728bb9c09127e000/src/functions/capture.ts),
  [compression](https://github.com/rohitg00/agentmemory/blob/da91cc05b3c79c59f6c0480f728bb9c09127e000/src/functions/compress.ts),
  [explicit memory](https://github.com/rohitg00/agentmemory/blob/da91cc05b3c79c59f6c0480f728bb9c09127e000/src/functions/remember.ts).
- [Agentmemory hybrid retrieval](https://github.com/rohitg00/agentmemory/blob/da91cc05b3c79c59f6c0480f728bb9c09127e000/src/state/hybrid-search.ts),
  [context assembly](https://github.com/rohitg00/agentmemory/blob/da91cc05b3c79c59f6c0480f728bb9c09127e000/src/functions/context.ts),
  [benchmark methodology](https://github.com/rohitg00/agentmemory/blob/da91cc05b3c79c59f6c0480f728bb9c09127e000/benchmark/LONGMEMEVAL.md).
- Cairn implementation boundaries: `crates/cairn-core/src/event.rs`,
  `crates/cairn-server/src/extract.rs`, `consolidate.rs`, `retrieve.rs`,
  `selector.rs`, and `crates/cairn/src/hook.rs`.
- Astra's independent read-only reassessment supports addressing memory formation
  and context assembly before another provider or ranking rewrite.
