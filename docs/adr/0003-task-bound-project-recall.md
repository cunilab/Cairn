# 0003 — Require a task before delivering project findings

**Status:** Accepted; query boundary implemented at `f44448eb`.
**Recorded:** 2026-10-09, retrospectively.

## Context

Startup continuity has no current task against which to decide whether a project
finding is relevant. Delivering all eligible memory bodies can insert unrelated
claims. Reusing cached availability also presents information as current without
being able to recheck membership or eligibility.

## Decision

Unqueried context returns bounded continuity and freshly checked authorized
memory availability, without project finding bodies. Native hooks send no query.
Working project search and explicit query context require bounded task keywords,
validated for size and credentials through MCP, daemon and server.

Bind responses to the recall policy and normalized query digest; clients refuse
missing or mismatched bindings. Queried context bypasses outage cache. Suppress
legacy unqueried bodies and their transmission trace. Do not cache availability
as a current claim. Preserve archival inspection and its eligibility disclosures.

Rank lexical overlap ahead of recency before limiting candidate records. Existing
authorization, scope, conflict and eligibility gates still apply.

## Alternatives and consequences

Recency-only candidate limiting can discard an older exact match before ranking.
Query binding prevents silent downgrade to an older response path, so mismatched
components require an actionable upgrade error. It does not make lexical
selection semantically reliable: a matching record still contains its appendix.
That limitation motivates [0004](0004-extractive-semantic-selection.md).

## Evidence

[Task-query development results](../../evals/m1-m2/results.md#task-query-development--code-verified-m2-remains-open)
record independent review, 390 affected unit tests, 56 PostgreSQL checks and
six passing CI jobs for `f44448eb`. Two clean-capture development cases yielded
10/10 relevant supported claims; neither exercised the mixed-record defect.
One control prior trace was incomplete. These results do not close M2.
