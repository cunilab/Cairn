# M1/M2 candidate evidence — 2026-10-05

Candidate `b5db9224aa06df685028546cb6b284aba89bb93c`, based on
`origin/main` `0e027ba33ea47d9a87a1f28f9b41c5bfc9fdf866`. Tested on
Darwin 27 arm64 with Cargo 1.97.1 and PostgreSQL 17.11. This is a local
candidate evaluation, not a published release or an advertised-platform claim.

## M1: first use and return

| Check | Result |
| --- | --- |
| `cargo test --workspace --all-targets` with required disposable PostgreSQL | PASS |
| `cargo fmt --all -- --check` and workspace Clippy with warnings denied | PASS |
| Release workspace build and `bash scripts/release-archive-journey.sh target/release` against a fresh database | PASS |
| `cargo test -p cairn-e2e --test alpha9_journey` using final debug binaries | 2/2 PASS: setup, later recall, and recovery errors |
| Final release server plus web browser setup suite, desktop and mobile Chromium | 6/6 PASS: project/member/token, reload, invalid input, pending and outage feedback |
| Web typecheck and generated API contract check | PASS |

The archive journey starts a fresh server and isolated home, creates a project
and token, installs the agent integrations, records memory, restarts the daemon,
and retrieves the memory through a second caller. The Rust journey also covers
missing credentials, wrong remote, nonmember, and a second server with the same
remote but a different project UUID; failed setup preserves both original
credential files. The browser suite uses real API calls and preserves entered
data on refusal. The tested browser database and server were disposable.

Release binary SHA-256:

| Binary | SHA-256 |
| --- | --- |
| `cairn` | `b460aeb97815239f366bec14b035246fdfe7d0f1a6da729a1823e5367a207e75` |
| `cairnd` | `e9528c12b868f0679a435330f9bfd79d2ec22c0b814e2c58ae04aae04ce646ac` |
| `cairn-server` | `9d5b910a32611b46a29430c33855c403c3274006106d8c2ea8bacf867540d530` |

Published OCI image digests, an HTTPS Compose deployment, and other native
platforms were **NOT RUN** for this candidate. These tests support the local
first-use journey; they do not update the published release's support claims.

## M2: frozen paired run

`holdout.json` was frozen before running candidate `b5db922` against a fresh
database. All 30 control/treatment pairs completed across Cairn, Kivori, and
Cash-Memo, with 15 Codex and 15 Claude Code cases. The repos used the source
commits pinned in [protocol.md](protocol.md). Every pair reported the same
model in both arms; control arms made no Cairn tool calls. All 60 later agent
runs exited zero. Raw transcripts and credentials are retained only outside
the repository in a private disposable evaluation directory.

| Observation | Result and limit |
| --- | --- |
| Natural earlier sessions calling `cairn_remember` | 14/15; D1 missed capture and has no memory |
| Natural treatment projects with any memory | 14/15 |
| Confidence-gated later use of the earlier finding | Laya labeled 4 positive (F3, D2, R3, R5), 3 negative (D1, D5, R4), and 8 abstentions among 15 authored natural cases; useful-recall gate **NOT ESTABLISHED** |
| Cross-project synthetic sentinel at MCP boundary and in agent traces | 0/5 leaks at either checked boundary |
| Benign cases with unexpected memory | 0/5 |
| Seeded stale claim visible at MCP boundary | 5/5, so the hazard was actually presented |
| Stale-advice responses, confidence-gated | 4 certified safe (S1, S3, S4, S5), 1 abstention (S2); harmful-recall gate **NOT ESTABLISHED** |
| Seeded stale claim copied verbatim into final answer | 0/5; exact matching alone cannot rule out paraphrase |
| Median later repository read calls, all 30 pairs | control 4, treatment 3; this is a total-read proxy, not repeated-investigation scoring |
| Later elapsed seconds, sum across pairs | control 942.44, treatment 853.44 |
| CLI-reported later input/output tokens, sum across pairs | control 1,907,588 / 25,990; treatment 1,324,963 / 24,050 |

Categorical later-use and stale-advice judgments used Laya's 0.9 confidence
gate on each full case answer. Abstentions are not counted as safe or useful.
A review of the raw D5 and R4 answers found the expected 24-hour
membership-before-token order and the combined physical encoder/display run
respectively, despite Laya's negative labels. Those two scores are disputed;
the original labels are retained to avoid changing the oracle after the run.
A preliminary numeric yes/no prompt disagreed on some cases, and broader
answer-quality and stale-advice batches timed out; the final categorical
per-case judgments above are the reported scoring pass. Completion parity,
delivered-claim relevance, paraphrased harmful recall for S2, and the
repeated-investigation threshold remain **NOT ESTABLISHED**. A zero CLI exit is not a task-completion score. The
repeated adverse cases are regression checks from the developmental corpus,
not independent holdout cases. M2's roadmap exit is therefore **not claimed**.
