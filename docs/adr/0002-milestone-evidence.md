# 0002 — Preserve milestone gates and failed evidence

**Status:** Accepted; full exits remain open.
**Recorded:** 2026-10-09, retrospectively.

## Context

The user requested M1 and M2 in one PR with full milestone evidence and work
through merge readiness. Passing code checks alone does not establish useful,
safe recall. Laya did not pass the revised calibration: 16/24 confidence-gated
answers, with three wrong answers among those 16.

## Decision

Keep deployment, installed-artifact journeys, paired repository tasks and
semantic gates required by [validation](../validation.md). Preserve every frozen
run, threshold, delivered-claim inventory, valid judgment and failed call.
Do not rescore valid observations or exclude inconvenient claims after a run.
Resume invalid transport attempts only under the recorded protocol.

The user authorized an independent calibrated judge as an exception to Laya
text scoring, with Laya's results retained. The current actor is the private
pinned Codex `0.160.0`, GPT-6-Luna with low effort; the calibrated semantic judge
is `gpt-6.1-sol`. Keep judge inputs isolated and complete.

The user explicitly instructed us to skip Claude. Report those cases as skipped;
the original two-agent exit is consequently not established. Prepare PR #66 for
review, without merging it or promoting a production release tag.

## Alternatives and consequences

Lowering the quality threshold, changing the corpus after seeing failures or
substituting mock-provider results for semantic judgments would change the claim
being tested. Development examples can diagnose a mechanism but cannot replace
the full evaluation. Source, runtime and artifact identities must accompany
results; older deployment evidence is historical after product changes.

## Evidence

[V13 metadata](../../evals/m1-m2/regression-v13-metadata.json) and
[preserved results](../../evals/m1-m2/results.md) record 15 valid Codex pairs
across three repositories, useful application 8/8 and claim quality 31/42
(73.8%), below the unchanged 90% target. Claude's 15 cases remain skipped.
Calibration reuse and common model-family limitations remain disclosed.
