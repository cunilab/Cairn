# 0005 — Configure inference separately from deployment

**Status:** Accepted; implementation in progress; live provider pending.
**Recorded:** 2026-10-09.

## Context

The semantic selector requires server-side inference. The user deploys Cairn
through Dokploy MCP and authorized adding configuration there after the work is
done. They will provide the inference endpoint, model and credential arrangement
later. Dokploy deployment does not itself identify an inference provider.

## Decision

Implement an OpenAI-compatible chat-completions connection using server-only
`CAIRN_INFERENCE_BASE_URL`, `CAIRN_INFERENCE_MODEL` and
`CAIRN_INFERENCE_API_KEY`. The base URL includes the provider's version prefix
(normally `/v1`); append `/chat/completions` once. Require HTTPS, except loopback
HTTP for local tests. Reject URL credentials, queries and fragments; disable
redirects. Do not emit credentials, provider prompts or raw provider errors.

All three settings absent permits startup, archive access and unqueried
continuity. Working recall with candidate records refuses to proceed without a
configured selector. Partial or malformed configuration fails startup. Do not
silently use an evaluation CLI's OAuth session as a production server dependency.

Complete implementation, mechanical tests and independent review before asking
for the provider details again. After the user supplies them privately, run live
semantic checks and the required exact-source milestone evidence. Configure the
deployment through Dokploy after this work; do not commit a credential or promote
a production release tag.

## Alternatives and consequences

An existing 9Router service is a possible OpenAI-compatible endpoint, not a
selected or validated provider. Its model ID and usable authentication remain
unconfirmed. No new inference SDK or local CLI dependency is needed: the server
already uses `reqwest`. Configurable transport does not establish semantic
quality. Until a real provider passes the gates, M2 and PR merge readiness remain
unproved.

## Evidence

The user's replies authorize Dokploy configuration after completion and defer
provider details. [0004](0004-extractive-semantic-selection.md) records the
selector contract; [candidate evidence](../../evals/m1-m2/results.md) preserves
the current missing live validation.
