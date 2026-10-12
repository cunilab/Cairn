# M1/M2 regression protocol v10

Repeat all 15 Codex cases from the unchanged 30-case corpus after the setup and
project reuse-policy corrections. Preserve all earlier attempts and judgments.
The 15 Claude cases remain explicitly skipped by user instruction; no Claude
actor or judge is invoked. This validates the authorized Codex subset, not the
original two-agent milestone exit.

## Freeze before execution

Commit the product, canonical skill, runner, and this protocol. Record that
source SHA, the candidate workflow run, manifest, archive checksums, and immutable
server/web image digests before running actors. Use the workflow's verified
macOS ARM archive for every actor arm. Use a new database, deployment volume,
project identities, isolated homes, and protected output directory.

Keep the corpus SHA-256
`99d9a0009a6e5b79636dc0e56e5977df9d071cbb095898df301d6499af782296`,
repository pins, prompts, arm order, CLI, 300-second actor budgets, rubric, and
exit thresholds from v9. At the user's explicit request, both Codex actor arms
now use **`gpt-6-luna` with `model_reasoning_effort="low"`**. Record that model
change; v9's Sol actor results are historical comparisons, not the same profile.
The calibrated semantic judge remains `gpt-6.1-sol`. Product changes may teach agents
to capture attributable support; evaluation prompts and expected answers remain
unchanged. Do not retroactively attest old or seeded memories.

## Ordinary setup and complete delivery evidence

Use the configuration emitted by ordinary `cairn setup`. Never rewrite MCP or
hook executable paths after setup. Check generated executable identities before
actors run; a failed check invalidates the comparison. Retain the complete
private stdout/stderr, final answers, delivery snapshots, and runtime identities.

Pinned hooks bypass the earlier PATH wrapper. Report wrapper capture enabled
only when it actually wrote a log. Independently bind each Codex stdout thread
ID to exactly one session history and preserve every delivered developer
message, with source line numbers, in a private context artifact. Missing,
ambiguous, malformed, or truncated context evidence invalidates the trace.
Include those artifacts in privacy scans and semantic judge packets. Include
all Cairn claims actually delivered through context, search, explicit inspection,
pins, warnings, and tool output, including rejected unsupported claims. Server
selection without transmission is not proof of delivery.

The committed runner accepts `CAIRN_M2_BIN_DIR` for the verified archive and
`--agent codex` for the authorized subset. With the protected `CAIRN_M2_OUT`,
`CAIRN_M2_CREDENTIALS`, `CAIRN_M2_SOURCES`, and `CAIRN_M2_CASES` set, run:

```sh
python3 evals/m1-m2/run.py --agent codex --jobs 1
python3 evals/m1-m2/collect.py
python3 evals/m1-m2/summarize.py
python3 evals/m1-m2/grade.py --jobs 1
```

The collector keeps every trace line and delivered developer message. It writes
private packets with the API token redacted and reports only identifiers/sizes.
The grader uses the unchanged [judge instructions](judge-instructions.md) and
`calibration-v4b.json` predicates. It checks the pinned CLI and records tool-free
process validity; semantic labels and raw responses remain private until bounded
metadata is exported. `--retry-failed` preserves invalid attempts before retry.

## Scoring and reporting

Use the independently isolated Codex judge profile that passed the frozen v9
calibration, with the unchanged rubric. Run actors before semantic judges to
avoid sharing quota concurrently. Fresh process/home per packet; expected labels
withheld; no project MCP, shell, plugins, apps, web search, or multi-agent tools.
Keep the actor/judge model-family and provider-attestation limitations visible.
Retry only failed or malformed judge attempts with originals preserved. Do not
replace failed actor pairs selectively.

The gates remain: useful natural application ≥80%; all delivered claims both
relevant and supported ≥90%; zero privacy leaks or high-impact harm; completion
no worse than control; median repeated investigation reduction ≥20% across
eligible pairs. Preserve unknown labels and failed comparisons. A failed or
incomplete required gate cannot be marked passed. Publish bounded metadata,
labels, counts, identity hashes, and skipped cases; keep raw content private.

## M1 evidence

Use a fresh trusted HTTPS deployment of the frozen images and archive. Recheck
browser provisioning, authorization, ordinary setup, actual Codex work and
capture, and a fresh later Codex process that uses remembered context. Recheck
actionable adverse paths and all five workflow archive journeys. Record the
source and artifact identity of each check. Local debug checks and older
candidate platform results are intermediate evidence only.
