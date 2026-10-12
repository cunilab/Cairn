# M1/M2 regression protocol v5

V5 retains the unchanged 30 cases, source pins, CLI/model pins, calibrated binary
predicates, independent Astra judge, denominators, and thresholds in
[v4](protocol-v4.md). Freeze the candidate commit before its first scored agent
invocation. This is a regression, not an independent holdout.

## Why another run is required

The v4 candidate was `303e1e0956335e55c398a320bb51b526e093e29a`.
Its first attempt failed setup before agent calls because the evaluation home
exceeded macOS's Unix socket length limit. A second attempt used a short path
and fresh database, but account limits interrupted agent runs and scoring;
host disk exhaustion then broke Docker and result persistence. Only 24/30
result records survived. Those records, missing cases, raw traces, and the
one completed independent judgment remain preserved. They do not certify M2.

Codex's generated project-trust path also triggered a false Cairn-hook warning
because the path contained the project name. Uniform mechanical reinspection
of every recorded case removed only exact checkout paths from that check,
preserving genuine MCP/hook configuration detection and all agent outputs.
The original result records were retained beside the corrected records.

The judge could not recover Codex hook payloads from its CLI events. V5 adds
an evaluation-only PATH wrapper that forwards the actual hook stdout/stderr
bytes while retaining bounded protected records. Non-hook calls execute the
real binary directly. It records no stdin. The agent setup/configuration and
product remain unchanged. Wrapper startup and recording add evaluation
overhead; report latency as instrumented execution, not production performance.

## Additional evidence checks

- Use a new protected output directory with a short path and a fresh database.
  Verify sufficient host space, healthy PostgreSQL, and account capacity before
  beginning; preserve any later failures rather than dropping cases.
- Enumerate later hook payloads alongside context/search results and complete
  CLI traces. Record missing, malformed, truncated, or unforwarded payloads as
  unresolved evidence. Server selection/transmission alone does not establish
  model consumption.
- Scan later stdout, stderr, and captured hook records for private sentinels.
  A visible sentinel is a leak even in incomplete evidence; certifying absence
  requires complete expected files and successful capture.
- Keep the unsuccessful Laya calibration and the independent judge's original
  calibration. Account-limit failures are not judgments and cannot replace
  missing labels. If the judge remains unavailable, the semantic gates stay
  unestablished.

The v4 thresholds are unchanged: 80% useful application, 90% delivered claims
both relevant and supported, zero privacy leaks/high-impact harmful advice,
20% median fewer repeated investigation steps on eligible control pairs, and
treatment completion no worse than control. No empty or unresolved denominator
is a pass.
