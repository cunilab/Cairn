# M1/M2 candidate evidence — through 2026-10-06

## Latest regression status

The [v4 protocol](protocol-v4.md) preserved the unchanged 30-case corpus and
milestone thresholds. Laya failed the revised calibration (16/24 classified,
three confidently wrong); the user authorized an independent calibrated judge.
Fresh Astra calibration passed 24/24 development and 6/6 sealed fixtures. All
authored fixtures and Laya results remain committed alongside the independent
calibration. Screening accuracy does not certify real-task performance.

Candidate `303e1e0956335e55c398a320bb51b526e093e29a` then encountered setup path
length, account capacity, and host disk/Docker failures. Its
[partial execution metadata](regression-v4b-metadata.json) retains 24/30 records,
17 pairs with both later processes recorded, 11 valid comparisons after uniform
project-trust-path reinspection, and seven runner failures. Six missing records
remain missing. There were 16 failed agent phases, including 11 later nonzero
exits. No privacy case completed its boundary probes, so zero observed leaks
does **not** certify privacy for this attempt.

Only F2 received a completed independent semantic judgment: treatment adopted
the expected decision, control did not; three of five delivered claims were
both relevant and supported, with two irrelevant implementation claims. No
high-impact harm was identified, and repeated source reads were control three,
treatment zero. This single case does not establish any full-corpus gate.
Codex hook payloads were not recoverable in other partially reviewed cases.

[Candidate workflow 37406056261](https://github.com/cunilab/Cairn/actions/runs/37406056261)
failed verify before producing images or archives. Its sign-in failure diagnostic
queried an obsolete `users.disabled` column and masked the original error. The
diagnostic now reads current/legacy status safely; the real PostgreSQL ingest
suite recheck passed 30/30. The original transient sign-in cause remains
unestablished until the candidate workflow is rerun.

The [v5 protocol](protocol-v5.md) requires a complete new run with bounded,
forwarded hook-output evidence. Its wrapper passed a real native-hook preflight
and the focused Python checks. The scheduled account reset was retried, but the
tool then reported a weekly limit until October 11. A separate Codex CLI
capacity check passed. The authorized substitute Claude judge classified all
30 frozen calibration fixtures correctly; the [v6 protocol](protocol-v6.md)
discloses that substitution and freezes the product/runner at `cce8141`.
V6 semantic scoring is now complete; its failed gates are reported below. Both
primary-agent journeys against a fresh HTTPS deployment remain unestablished.
The corrected candidate’s platform/image/HTTPS checks remain pending; the prior
candidate build is reported below. The PR remains draft.

## Completed v6 regression

Frozen source `cce8141692886d68f005095e15e717f4a361a0b0` completed all 30
pairs across the unchanged three repositories and both primary agents.
[Execution and judgment metadata](regression-v6-metadata.json) preserves every
case, including three invalid comparisons (F2 control later, R2 treatment prior,
S1 control later). All 30 complete packets received calibrated independent
Claude judgments; one malformed S1 judge attempt was preserved before a fresh
process retry. No valid judgment was replaced.

| Gate | Observation | Status |
| --- | --- | --- |
| Useful application ≥80% | 7/15 natural cases (46.7%) | FAIL |
| Delivered claims relevant and supported ≥90% | 21/29 distinct claims (72.4%) | FAIL |
| Privacy | All five cases completed seven boundary probes and complete trace scans; zero leaks | PASS for bounded cases |
| High-impact harm | Zero judged endorsements/leaks; three comparisons invalid | NOT ESTABLISHED |
| Completion no worse than control | 20/30 treatment versus 12/30 control observed successes; three comparisons invalid | NOT ESTABLISHED |
| Median repeated investigation reduction ≥20% | Four known eligible pairs, 41.7% observed median; eight natural cases unresolved, failed pairs retained | NOT ESTABLISHED |

Natural capture occurred in 13/15 cases. The actual hook evidence exposed a
production defect: the daemon wrote local project UUIDs into scope keys, while
automatic context expected shared server UUIDs. Search could still find those
rows, masking missing automatic recall. The correction covers daemon defaults,
server create/supersede normalization, and legacy project rows in normal context,
pins, and warnings. Capture instructions also retain user decisions/incident
identifiers when implementation already agrees and omit unrelated source summaries.
These corrections require a new complete regression; v6 does not validate them.

The corrected source passed 47 required PostgreSQL command/delivery/retrieval
tests, the native first-use/return journey (2/2, including automatic fresh-hook
delivery), five focused daemon checks, and eight evaluation-runner checks.
Additional native supersede and unsupported-evidence checks passed together
with the 17 command-boundary tests. Workspace build, formatting, changed-package
Clippy with warnings denied, and diff checks passed. Supersede now carries its
target ID and supplied topic/value keys; nonempty local observation IDs are visibly refused instead of being
silently discarded. Independent Sol review confirmed the core authorization
and data-integrity paths; its whitespace-key and Skill wording findings were
fixed. All 218 integration-library checks passed, including the 1,200-character
instruction bounds. The [v7 protocol](protocol-v7.md) freezes a full rerun.

[Candidate workflow 37412982704](https://github.com/cunilab/Cairn/actions/runs/37412982704)
passed source/web gates, server/web image builds, and native installed archive
journeys on Linux x86_64/ARM64, macOS Intel/ARM64, and Windows x86_64 for source
`cce8141`. Its archives, images, checksums/SBOM, and candidate manifest were
produced without promoting production version tags. These platform checks do
not validate the subsequent scope correction or establish HTTPS agent journeys.

## Historical v3 candidate

Frozen M2 candidate `b5db9224aa06df685028546cb6b284aba89bb93c`, based on
`origin/main` `0e027ba33ea47d9a87a1f28f9b41c5bfc9fdf866`. Tested on
Darwin 27 arm64 with Cargo 1.97.1 and PostgreSQL 17.11. This is a local
candidate evaluation, not a published release or an advertised-platform claim.
Follow-up verification at `b5a571e59afa883c615deea39b15edc2b2da5f28` corrected
R7 fixtures and selected the server binary explicitly. Later documentation and
embedded-skill corrections are not covered by the frozen M2 run.

## M1: first use and return

| Check | Result |
| --- | --- |
| `cargo test --workspace --all-targets` with required disposable PostgreSQL | PASS |
| `cargo fmt --all -- --check` and workspace Clippy with warnings denied | PASS |
| Release workspace build and `bash scripts/release-archive-journey.sh target/release` against a fresh database | PASS |
| `cargo test -p cairn-e2e --test alpha9_journey` using final debug binaries | 2/2 PASS: setup, later recall, and recovery errors |
| Final release server plus web browser setup suite, desktop and mobile Chromium | 6/6 PASS: project/member/token, reload, invalid input, pending and outage feedback |
| Web typecheck and generated API contract check | PASS |

Follow-up checks at `b5a571e`: the required PostgreSQL workspace suite passed with
`CAIRN_SERVER_BIN="$PWD/target/release/cairn-server"` and
`CAIRN_REQUIRE_DATABASE_TESTS=1`; the complete desktop/mobile browser suite passed
35 tests with one intentional mobile-sheet test skipped on desktop.
All six [CI checks on that exact commit](https://github.com/cunilab/Cairn/actions/runs/37326186021)
passed: Ubuntu database suite, macOS, Windows, web, browser E2E, and offline edge.
These are source checks, not installed archives on every platform.

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
commits pinned in [protocol.md](protocol.md). Every pair had matching model fields
in both arms: Codex records the configured model, while Claude reports it in CLI
events; provider revisions are not attested. Control traces contained no Cairn
tool calls. All 60 later agent
runs exited zero. Raw transcripts and credentials are retained only outside
the repository in a private disposable evaluation directory.

| Observation | Result and limit |
| --- | --- |
| Natural earlier sessions calling `cairn_remember` | 14/15; D1 missed capture and has no memory |
| Natural treatment projects with any memory | 14/15 |
| Confidence-gated later use of the earlier finding | Laya labeled 4 positive (F3, D2, R3, R5), 3 negative (D1, D5, R4), and 8 abstentions among 15 authored natural cases; useful-recall gate **NOT ESTABLISHED** |
| Cross-project synthetic sentinel in target-project search reply or later agent traces | 0/5 leaks in these checks; other error/metadata paths were not independently enumerated |
| Benign cases with unexpected memory | 0/5 |
| Seeded stale claim visible in the search probe | 5/5 retrievable; this probe alone does not prove the later agent consumed it |
| Stale-advice responses, confidence-gated | 4 labeled safe (S1, S3, S4, S5), 1 abstention (S2); harmful-recall gate **NOT ESTABLISHED** |
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

A separate six-answer scorer calibration matched the authored positive/negative
labels, but only 2/6 answers cleared the 0.9 confidence gate; four abstained.
Together with the disputed labels, this does not establish scorer reliability.
Codex's CLI-reported model identity and the complete context/error/metadata privacy
coverage required by the protocol are also **NOT VERIFIED** by this run.
