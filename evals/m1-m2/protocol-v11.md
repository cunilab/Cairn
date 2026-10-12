# M1/M2 regression protocol v11

V11 is the next complete run after v10. It preserves v10's 15 valid Codex
comparisons, judgments, corpus SHA
`99d9a0009a6e5b79636dc0e56e5977df9d071cbb095898df301d6499af782296`,
repository pins, prompts, labels, thresholds, and failed gates. It does not
rescore or exclude any v10 claim. The five v10 R3 claims found in ordinary
repository `rg` output remain in the historical denominator.

## Delivery boundary

For this run, a claim is delivered only when a current Cairn record reaches the
later agent through a native hook context, MCP/native context, search, inspect,
pin, warning, or Cairn CLI response. The channel remains Cairn when a shell
command transports that current Cairn response. Ordinary repository file or
search output that quotes a historical record is not Cairn delivery. A server
record selected without transmission is also not delivery. Count a current
Cairn-delivered claim even if it is irrelevant, unsupported, or rejected.

Freeze complete line-numbered context, hook, MCP/CLI, and tool payload evidence
before judging. Keep server selection data as corroboration only; it cannot add
claims absent from the actual delivered payload.

## Calibration and execution

Before actor invocation, run the same isolated `gpt-6.1-sol` / Codex CLI
`0.160.0` judge profile against every complete synthetic `judge_input` packet in
`calibration-delivery-v11.json`, withholding the `expected` objects. Send
only the opaque case IDs and judge inputs; descriptive fixture names stay outside
the judge packet so identifiers cannot disclose inclusion labels. Recheck the
30 original predicate fixtures with expected labels withheld as well. Compare
delivery claims by `record_id` and `canonical_claim`, rather than judge-created
claim IDs. Record the labels and process/tool-free validity beside the private
calibration output. The fixtures calibrate the delivery boundary only; they do
not alter the frozen binary predicates in `calibration-v4b.json`. Do not claim
calibration passed until that run is recorded.

Then use a fresh artifact freeze, database/deployment volume, account,
projects, homes, and short private `CAIRN_M2_OUT` path. Run the complete Codex
subset once: `gpt-6-luna` with low reasoning effort, including browser UI F1
once. Preserve the same 300-second budgets, arm order, source pins, and 15
Claude skips. Run the unchanged collector, summary, and isolated judge after
all actors; apply the unchanged 80/90/20, privacy, harm, and completion gates.
Do not selectively replace actor pairs or change the corpus, rubric, or labels.
