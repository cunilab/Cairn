# 0001 — Bind native calls to framework session identity

**Status:** Accepted; implemented in V13 (`58107d9`).
**Recorded:** 2026-10-09, retrospectively.

## Context

Repeated instruction changes did not reliably correlate native Codex MCP calls
with captured lifecycle activity. Model-supplied session keys could be invented.
Astra identified session identity as an infrastructure boundary.

## Decision

For the verified Codex CLI `0.160.0` profile, use actual per-call framework thread
metadata as the native session key, overriding a model-supplied key. Validate
explicit Cairn session UUIDs against that key, project and worktree. Refuse missing
or malformed metadata. MCP cannot claim a native agent to create lifecycle
sessions. Other client profiles retain their explicit identity path.

## Alternatives and consequences

More wording changes cannot establish caller identity. A global CLI upgrade would
change the evaluation runtime; use the private pinned installation instead.
Local framework metadata establishes provenance, not authentication. Membership
and credential checks remain necessary. New native CLI profiles need verification
before receiving this treatment.

## Evidence

The [V13 protocol](../../evals/m1-m2/protocol-v13.md) and
[V13 results](../../evals/m1-m2/results.md#final-v13-candidate--m1-passed-m2-claim-quality-failed)
preserve the pinned upstream implementation, actual local-call proof,
independent review and installed-artifact journey. That evidence applies to
`58107d9`; it does not prove later source revisions.
