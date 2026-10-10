# 0009 — Give each native turn one bounded capture checkpoint

**Status:** Accepted; native capability verified; implemented with mechanical checks passed; native journey and semantic gates open.
**Recorded:** 2026-10-10.

## Evidence and decision

[0008](0008-independent-finding-capture.md) passed structural checks, but its
frozen F1 diagnostic observed no capture despite delivered guidance. Changing
another description or provider would not establish the omitted action.

Use the native Codex Stop boundary to give the agent one opportunity to submit
supported durable findings, or an explicit `no_durable_finding` disposition.
The checkpoint establishes an accountable response, not truth, semantic coverage
or later usefulness. Never force invented memory. Keep the existing recall
selector and full milestone gates.

A disposable Codex 0.160.0 / GPT-6-Luna / low probe verified that Stop's block
response resumes the same session and turn, then exposes `stop_hook_active=true`.
A separate cross-channel probe verified the MCP native turn metadata and both
Stop invocations agree exactly. [Capability metadata](../../evals/m1-m2/native-finalization-capability-metadata.json)
retains private evidence hashes. [Official hook documentation](https://learn.chatgpt.com/docs/hooks)
describes Stop continuation; the pinned executable observation establishes the
profile actually used here.

## Boundaries

- Parse native session/turn UUIDs strictly and cross-check MCP metadata identities.
  Never credit caller-authored turn identity, or infer a turn from a thread's
  latest event or timestamp. This metadata is a client assertion; existing
  account, project, worktree and session checks remain the authorization boundary.
- Keep bounded local per-turn bookkeeping keyed by account, configured server
  identity, session and turn. Record intervention separately from disposition.
  Retain at most 90 days of this bookkeeping; store no prompt, output or reason.
- Credit capture only in the same SQLite transaction that durably admits its
  command. Saturation and rollback cannot create credit. Credit means at least
  one queued finding, not full batch admission, server persistence or coverage.
- Atomically permit one intervention per key. `stop_hook_active=true`, missing
  native identity, cancellation, errors and deadline expiry release finalization.
  Record a bounded missed checkpoint where possible; never loop indefinitely.
- Permit an exact-turn `no_durable_finding` response through `cairn_session`.
  An empty disposition is not a useful-memory success and cannot lower an
  established capture disposition. Generic clients retain their existing writes
  without claiming native-turn completion.

## Alternatives and verification

Instruction-only automation failed the tested ordinary task. Crediting the latest
turn could accept a delayed earlier write. A daemon-issued checkpoint token is an
alternative for runtimes without verified turn metadata; it is unnecessary for
this observed profile. A separate extraction service or raw transcript transfer
would add dependencies and violate the present capture boundary.

Verify exact identity separation, concurrent Stop claims, admission rollback,
queue saturation, recursion, outage, cancellation, and stale/closed sessions.
Then freeze a new ordinary capture/return journey and a benign empty-disposition
case. The earlier failed diagnostic remains unchanged. Mechanical checkpoint
completion cannot replace the 90% claim-quality and 80% usefulness gates, fresh
deployment evidence, or the unchanged full workload. Claude remains skipped.

## Implementation evidence

The checkpoint is implemented through native MCP metadata, the Stop hook, daemon
session validation and SQLite migration 17. Capture credit shares the command
admission transaction; no prompt or answer is stored in checkpoint bookkeeping.

Mechanical verification passed: 51 CLI tests, six checkpoint tests, ten daemon
handler tests, the generic-wire compatibility test and the installed skill
revision test. The store suite passed before the additional retention test; the
six checkpoint tests include retention and session deletion. Clippy passed for
the four affected packages with warnings denied. Astra's bounded source review
verified that all four native session/thread identifiers must agree, including
the regression test for distinct internally consistent pairs; no remaining
must-fix finding was reported. These checks do not establish automatic capture
or useful recall. The frozen ordinary native journey is the next evidence gate.
