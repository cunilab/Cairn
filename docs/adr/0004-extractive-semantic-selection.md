# 0004 — Select exact excerpts inside matching records

**Status:** Accepted; candidate implemented and mechanical regressions passed; live semantic validation pending.
**Recorded:** 2026-10-09.

## Context

V13 delivered useful decisions together with 11 irrelevant implementation claims
in four mixed records. The claims were supported; their task relevance failed.
Task queries select matching records but leave this failure mechanism intact.
Astra reviewed the captures and advised correcting selection before another full
15-case run.

## Decision

Use a server-owned, task-conditioned semantic selector on authorized, eligible
candidate records. The provider returns record IDs and exact source quotations.
Cairn resolves unique UTF-8 byte spans, rejects unknown IDs, invented text,
ambiguous matches, overlapping spans and malformed responses, and returns only
validated excerpts in source order. Omitted records are abstentions. Preserve
original records for archive inspection.

Apply the same gate to working search, context and detail access; an alternate
working route must not return the original body. Keep source attribution and
record revision, but exclude unselected free-text support summaries. Recheck
authorization, eligibility and revision after inference. Bound inputs, outputs,
concurrency and request time. Provider failures or invalid selections produce an
actionable refusal with no whole-record fallback.

Bind delivery to policy `task_excerpts_v1` and the normalized task-query digest.
Record source and selected-content hashes and byte spans in provenance, including
context transmission traces. Schema 10 provides that trace representation;
older schemas must refuse excerpt working recall rather than omit provenance.

## Alternatives and consequences

Further capture wording changes and cleaner development captures do not prove
that mixed-record delivery is fixed. Batch capture or author-written summaries
alone cannot independently select relevant claims. Keyword sentence filtering
does not establish semantic relevance or preserve necessary qualifiers.

Exact quotation checks prevent invented text; they do not prove that omitting a
qualifier, negation or attribution is harmless. A real calibrated judge must
evaluate mixed, interleaved, qualified, no-match and outage cases. Mechanical mock
tests establish extraction, authorization and failure behavior only. Freeze the
corrected source and repeat the unchanged full evaluation after those checks.

## Evidence

[Preserved V13 and development results](../../evals/m1-m2/results.md) document
the failed quality gate, whole-record regression and Astra's recommendation.
No live provider result or new full milestone pass is established by this ADR.

Independent review found and corrected a pinned-body bypass, unbounded inference
capacity waiting and mismatched search/selector limits. A subsequent PostgreSQL
test paused inference and revoked membership; context still returned a record
because `ReaderContext` held a request-start membership snapshot. Rebinding the
session did not refresh that snapshot. Context now performs a fresh membership
query after inference, alongside source revision/eligibility checks. Preserve the
paused-provider regression: source review alone missed this authorization bug.

## Groq development screen

The user authorized temporary Groq testing with `openai/gpt-oss-120b`, with a
later model replacement planned. Five live synthetic server cases had valid
source hashes and byte spans. The independently calibrated judge (42/42 reused
fixtures) rejected one excerpt: it removed “The user requires” while retaining
account-specific conditions. Capture basis `user_report` cannot distinguish
reported requirements from reported implementation. The failure is preserved.

Astra recommended one general clarification of the existing faithfulness rule:
select complete, self-contained source statements, including who requires or
reports them and whether they express requirements, proposals or implementation.
Prefer whole sentences and adjacent statements when needed; abstain if that
would require unrelated claims. Automatic sentence expansion was rejected because
it could reintroduce irrelevant claims; sentence-boundary validation alone cannot
handle cross-sentence attribution. Validate once on unseen development cases;
if this model still fails, stop its qualification rather than tune repeatedly.
The full workload and thresholds remain unchanged. This change needs new frozen
source/artifact/deployment evidence; the prior workflow is cancelled.

The unseen six-case follow-up retained all required qualifiers, but the independent
judge found an irrelevant sidebar claim delivered alongside the backup requirement.
All six cases passed mechanical provenance; five passed relevance. The temporary
Groq model is not qualified. Stop its qualification as planned; require another
model to pass development checks before full milestone evidence. Both screens
and the harness-only invalid attempt remain in [development evidence](../../evals/m1-m2/groq-development-metadata.json).
