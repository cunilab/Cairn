# 0010 — Establish capture completeness before claiming automatic memory

**Status:** Accepted by explicit user answer; implemented; mechanical checks passed; comparator qualification failed.
**Recorded:** 2026-10-10.

## Evidence

[0009](0009-bounded-native-finalization-checkpoint.md) establishes bounded native
admission bookkeeping. Its repair check passed. Its frozen F1 semantic screen
failed because agent-submitted implementation findings omitted an authored team
requirement. A successful admission checkpoint would still credit those findings.
Retrieval cannot recover intent that was never captured. Full M2 gates remain
unchanged, and the failed evidence remains recorded.

## Proposed boundary

To evaluate completeness, retain a bounded, redacted authored-task record locally
at the native input boundary, bound to the verified session and turn. A separate
capture check can compare selected findings against that record before declaring
coverage. The task record is local input to the capture process, not a project,
team or global memory and not an automatic server upload. Only selected findings
continue through existing attested commands, scope and account authorization.

The user explicitly approved this boundary on 2026-10-10. The implementation
retains at most 16384 UTF-8 bytes per task for 24 hours and 128 turns per account
and server lane. Redaction happens before local IPC and is checked again by the
daemon. Local SQLite records live under the existing protected Cairn directory,
with session deletion cascading and credential-lane replacement purging records.
Expired rows are inaccessible and removed on startup, access and minute maintenance
while the daemon runs; an offline database is cleaned when next opened. Missing or truncated input, uncertain
extraction and failed checks must remain unknown coverage; they must not force
invented findings or block session completion. Local retention does not authorize
sending task text to an external inference provider.

An implementation needs an evaluated comparator, not another unmeasured instruction.
The temporary Groq selector remains unqualified; no production provider is selected.
No capture-completeness mechanism or provider is claimed to pass here.

## Alternative

Keep agent-submitted findings as the only capture input. This preserves the
present boundary but leaves completeness dependent on the coding agent. Existing
checkpoint success is insufficient evidence for M2, and qualification stays
blocked until a new independent screen passes. Changing milestone scope would
require an explicit agreement; this proposal does not lower thresholds.

## Validation required

- Verify exact account, project, session and turn isolation for local task records.
- Verify redaction, access permissions, bounded retention and explicit incomplete input.
- Verify outage/cancellation release completion without fabricated coverage.
- Independently evaluate intent versus implementation, conditions, absent findings
  and mixed unrelated material on a frozen unseen screen.
- Only after qualification, rerun the full paired workload and fresh installed
  deployment evidence. Claude remains skipped under the existing user instruction.

## Comparator and failure boundary

The implementation uses a separately configured semantic comparator against
the actual admitted finding snapshots. It emits exact source quotes with
existing finding IDs. Output bounds and references are checked mechanically;
semantic coverage still requires independent qualification. Cache validity is
bound to task/finding hashes and a revision including the prompt, provider URL
and model. New findings invalidate previous coverage. Missing, redacted or
truncated input, invalid output and unavailable inference produce unknown.

The comparator defaults to unconfigured. Loopback processing is permitted; an
external endpoint requires explicit configuration of
`CAIRN_CAPTURE_REVIEW_ALLOW_EXTERNAL=true`. The existing temporary Groq test
authorization is used only for synthetic comparator fixtures and isolated
synthetic native journeys. No real user-task upload or production configuration
is authorized by the local-retention decision alone.

One Stop intervention requests a review and any supported missing requirements;
unknown coverage remains visible but releases finalization. An agent-authored
no-finding assertion is no longer semantic completeness credit. These changes
implement accountability; they do not assert complete or useful memory until
the comparator and unchanged M1/M2 evidence pass.

## Mechanical implementation verification

The initial local implementation passed 53 CLI tests, all 49 store tests, 64
daemon tests, the installed Skill revision test and four-package Clippy with
warnings denied. The synthetic live-provider test is explicitly excluded from
normal tests and remains a separate evidence gate. Formatting and whitespace
checks pass. Sol's independent review identified unbounded native finding
snapshots; store-level content/payload/per-turn limits and admission/ordinal
rollback regression checks now pass. Sol and Astra subsequently reached usage
limits, so independent review of the final corrections is incomplete.

## Frozen comparator screen — failed

Frozen `d3303b3a` ran twelve synthetic cases through the actual Rust comparator
with temporary Groq `openai/gpt-oss-120b`. Eleven matched the fixture predicates
and zero abstained. It caught the omitted credential-restoration intent but
falsely credited coverage when the SQLite choice lost its restriction to
isolated integration tests. False positive completeness stops qualification.
The prepared native completeness journey and full milestone workload were not
run after the failed gate. No prompt, provider or corpus iteration follows
this result. This screen is a development gate, not independent milestone
scoring. [Metadata](../../evals/m1-m2/capture-completeness-development-metadata.json)
records all cases and private evidence hashes, including the initial zero-test
filter mistake.

## Final privacy review corrections

Sol resumed the independent review and found two credential-transition races
and retained deleted text in SQLite's WAL. Credential replacement now holds
the server write lock across purge and publication; native finding admission
holds its read lock through the database transaction. Native task recording
still revalidates its lane after writing. Expiry maintenance and lane purge
checkpoint the WAL after deletion; a busy checkpoint returns `WouldBlock`
rather than claiming physical cleanup, and maintenance retries on its next tick.

Both credential concurrency regressions pass. File-backed tests verify that
task markers disappear from the database and WAL after expiry/purge, and that
a held reader causes a visible failure followed by successful cleanup retry.
The final store suite passed 51 tests, the daemon suite passed 66 with the
live comparator test excluded, and affected-package Clippy and whitespace
checks passed. Sol independently verified the correction and three focused
regressions, finding no remaining mechanical blocker in the reviewed scope.
This does not change the failed comparator screen. Remote CI passed all six
jobs on preceding head `df6adecc`; the privacy correction needs its own CI run.

The local mechanism can now expose missing authored intent, but this tested
comparator is not qualified to establish completeness. A different explicitly
configured comparator must qualify before native useful-return and full
milestone evidence can advance. Production and real-task external processing
remain unconfigured.
