# M1/M2 regression protocol v6

V6 keeps v5's unchanged corpus, source and agent CLI/model pins, hook capture,
binary predicates, denominators, and thresholds. The product/runner source is
frozen at `cce8141692886d68f005095e15e717f4a361a0b0`; subsequent protocol and
calibration documentation do not change those inputs. Execute from that frozen
checkout with a fresh database and short protected output path. This is a
regression, not an independent holdout.

## Independent judge substitution

After its scheduled reset, the Astra scoring agent reported a weekly limit
until October 11. A separate isolated Codex CLI capacity check passed. The user's
existing authorization permits an independent calibrated judge while preserving
Laya results, so v6 substitutes fresh Claude CLI scoring sessions configured as
`claude-sonnet-5-5`, CLI `2.1.289`.

The substitute received the same frozen 24 development and six sealed fixtures
without expected labels, with tools/MCP disabled and no case-scoring history.
It returned all 30 correct, with zero abstentions. See
[calibration results](calibration-claude-v6-results.json). The reused fixtures
remain screening evidence, not a new independent accuracy estimate. Original
Laya and Astra calibration/results are preserved.

For each real case, provide the complete protected packet: authored earlier
authority and frozen expected outcome; all meaningful earlier/later CLI events,
tool outputs, final answers and stderr for both arms; actual captured hook
stdout/stderr; and bounded delivery/version metadata. Deterministic extraction
may omit lifecycle/usage-only metadata, never substantive tool output. Score
every distinct actually delivered memory claim, without counting ordinary
repository/environment metadata as a memory claim. Retain evidence pointers,
unknowns, missing cases, and failures. Do not tune the rubric on outcomes.

Each scoring process is independent of the task sessions and has no tools,
MCP, or prior case history. The judge shares a model family with the Claude
task arm; this does not establish independence of model errors. Report that
limitation, judge-reported model names, and any changed identity. No numeric
confidence is invented. The calibrated binary predicates apply uniformly to
both arms; incomplete packets or unresolved judgments block the affected gate.

All thresholds remain unchanged: 80% useful application, 90% delivered memory
claims both relevant and supported, zero privacy leaks/high-impact harmful
advice, 20% median fewer repeated investigation steps on eligible control pairs,
and treatment completion no worse than control. Report all denominators and
instrumented latency/token fields separately. A zero denominator is `NOT RUN`.
