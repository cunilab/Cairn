# M1/M2 candidate evidence — through 2026-10-08

## Final v13 candidate — validation pending

Astra's root-cause review separates infrastructure session identity, optional
model-owned capture and model-owned answer fidelity. V13 binds the observed
Codex CLI `0.160.0` profile to its actual per-call framework thread identity,
which overrides invented model session keys. Explicit Cairn UUIDs remain
validated against that key, project and worktree. Missing or malformed native
metadata is refused; other client profiles retain explicit selection. MCP
cannot start a native lifecycle session by claiming a different agent label.
Local-client metadata is not authentication.

A non-corpus actual pinned CLI → Cairn stdio → mock-daemon check passed with
an invented model key overridden by the actual thread identity. Read-only
inspection confirmed that this identity matches both real native hook sessions
in the preserved deployment. Independent review passed after the lifecycle
bypass was closed. The durable capture and recall guidance corrections remain
scoped to project findings, lasting policies and relevant detail fidelity.

The final [V13 protocol](protocol-v13.md) keeps the corpus, prompts, profiles,
budgets, calibration and thresholds fixed. Freeze once and execute one fresh
complete run. Do not start another wording-tuning cycle after that run. Passing
requires actual persisted capture and useful application, not a capture
acknowledgement or the presence of memory alone.

This strategy is consistent with primary-source guidance on
[unambiguous agent tools and actionable errors](https://www.anthropic.com/engineering/writing-tools-for-agents),
[small, high-signal context](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents),
and [evaluating both agent harness and final state](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents).
Those sources inform the design; they do not establish Cairn's gate results.
The full V13 calibration, candidate workflow, deployment and semantic results
are pending. [Two invalid predicate calls](invalid-v13-calibration-metadata.json)
are preserved: a new private wrapper omitted the frozen response instructions.
No semantic calibration pass, delivery fixture or actor run is counted from
those attempts. The prospective correction restores the exact original input
and uses the pinned CLI's output-schema support to require the declared shape;
labels and gates remain unchanged. **PR #66 remains draft.**

## Preserved v12 candidate — useful recall and quality failed

Frozen source `9e48676b452804a1f14834f868f9f25385afed0d` passed all six PR
checks. Its Ubuntu workspace suite passed **1,252 tests**, with zero failures or
ignored tests. Local verification passed 310 affected-package tests, formatting,
Clippy with warnings denied, and 14 runner checks. Independent review covered
typed session recovery, capture scope, and prospective delivery provenance.

The [v12 protocol](protocol-v12.md) preserves the corpus, repository pins,
actor prompts, arm order, 300-second budgets and thresholds. Both Codex arms
remain GPT-6-Luna / low through CLI `0.160.0`. The isolated `gpt-6.1-sol` judge
passed [all 42 calibration fixtures](calibration-codex-v12-results.json) before
actors: 30 reused predicates, seven reused delivery fixtures and five new
operational-provenance fixtures. All 13 calibration transport calls passed
isolation and raw-event checks, with no tools or compaction. Exact local input
reconstruction and acknowledged injection do not establish provider readback.

[Candidate workflow 37736051149](https://github.com/cunilab/Cairn/actions/runs/37736051149)
passed all 17 jobs, including both images, the manifest and all five installed
archive journeys. Archive checksums, native identities and actual fresh
deployment revisions were verified. The first full actor attempt produced
[46 provider quota failures](invalid-v12-quota-metadata.json), zero valid
comparisons and no judgments; all original outputs remain preserved.

After the reported reset, non-corpus readiness probes passed for both models
using a private CLI `0.160.0`. A durable checkout restored the exact frozen
source after temporary checkout cleanup. The same candidate assets were
reverified, and a new deployment, volume, account, projects and output directory
begin a complete retry. The original 42 valid calibration fixtures are reused
with unchanged judge instructions, transport and model settings. The retry stops
on renewed quota exhaustion; no valid comparison or judgment is replaced.

The complete retry passed browser provisioning and ordinary setup, including nine
absolute native hook commands, fresh empty personal/team domains and verified
membership/token ownership. All **15 comparisons, 46 actor phases and 15
judgments** are valid, with verified runtime identities. A packet collection
failure caused by a missing credential environment variable was preserved;
collection and judging resumed without rerunning actors. All 15 judge RPC logs
passed isolation, input and compaction checks. See [bounded candidate proof](candidate-v12-metadata.json)
and [complete gate results](regression-v12-metadata.json).

| Gate | V12 observation | Status |
| --- | --- | --- |
| Useful natural application ≥80% | 5/8 (62.5%) | FAIL |
| Delivered claims relevant and supported ≥90% | 26/30 (86.7%) | FAIL |
| Privacy | Three complete boundary probes and trace scans, zero observed leaks | PASS for bounded cases |
| High-impact harm | Zero judged harmful endorsements/leaks | PASS |
| Completion no worse than control | Treatment 11/15 versus control 7/15 | PASS |
| Median repeated investigation reduction ≥20% | Six eligible pairs, median 100%; no unknown natural counts | PASS |

F1 did not capture the prior user decision: its later persisted source finding
is not evidence of prior user-choice capture, and useful recall failed. The
shared M1 return journey therefore **failed**. F5's attempted capture used an
invented vendor session key and failed. R1 preserved a repeated failure count
in memory but omitted that count when answering. The four quality failures are
supported but irrelevant temporary workflow statements; all remain in the
original denominator. No valid V12 observation is relabelled or replaced.

Independent Astra review recommends general corrections to session recovery,
durable capture scope and fidelity when applying recall. V13 will evaluate
those corrections prospectively with the same corpus, prompts, profiles,
budgets and gates. No pass is projected. Claude remains skipped by user
instruction; original two-agent milestone exits remain open. **PR #66 remains
draft and is not ready to merge.**

## Preserved v11 candidate — quality gate failed

Frozen source `6982de122ffca71a500a048358e7a6f207543ac9` passed all six PR
checks. Its Ubuntu workspace suite passed **1,251 tests**, with zero failures or
ignored tests. [Candidate workflow 37708344830](https://github.com/cunilab/Cairn/actions/runs/37708344830)
passed all 17 jobs, including all five installed archive journeys. The manifest,
five archive checksums, SBOM, native executable identities, immutable image
digests, and actual fresh deployment revisions were verified. See the
[bounded candidate evidence](candidate-v11-metadata.json).

A fresh HTTPS deployment, volume, account and projects completed the browser UI
project/token journey and ordinary setup before all **15 Codex comparisons**.
All **46 actor phases** succeeded with complete traces and verified runtime
identities. Actors used **GPT-6-Luna / low**, unchanged corpus and repository
pins, arm order, prompts and 300-second budgets. All eight natural cases
captured persisted memory. F1 is counted once as shared M1/M2 evidence: accepted
capture, two persisted eligible `user_report` records, browser token owner,
later transmission and independently judged useful application passed. A
pre-actor Docker health verification race was preserved separately; zero actors
ran in that failed verification attempt.

[Semantic calibration](calibration-codex-v11-results.json) passed all 30 reused
predicate fixtures and seven delivery-boundary fixtures before actors. Expected
labels were withheld, delivery IDs were opaque, and no unknown labels or tool
calls occurred. After all actors, 14 judge calls completed; F1 was rejected
before evaluation because its complete prompt was **1,235,147 characters**,
above the CLI's 1,048,576-character limit. Lossless outer JSON compaction was
still too large and was not invoked as a retry.

The [ordered-input transport screen](calibration-codex-v11-transport-results.json)
then passed the same 37 fixtures, including an oversized predicate prompt and
arbitrary boundaries across delivery packets. All exact input parts were
injected as ordered user messages before one generation: no intermediate model
responses, tools, compaction, omitted text or scoring changes. Only F1's invalid
judge call was retried, with the unchanged `gpt-6.1-sol` / CLI `0.160.0` profile.
Its chunks reconstructed the source prompt exactly locally; provider input was
not read back or independently hashed. All ten retained RPC logs were checked
for tools and both compaction event forms, with zero observed. All 14 valid judgments remained
byte-for-byte unchanged; the original rejection and retry hashes are preserved.
All **15 judgments** are now valid. See [full bounded results](regression-v11-metadata.json).

| Gate | V11 observation | Status |
| --- | --- | --- |
| Useful natural application ≥80% | 7/8 (87.5%) | PASS |
| Delivered claims relevant and supported ≥90% | 21/29 (72.4%) | FAIL |
| Privacy | Three complete boundary probes and trace scans, zero observed leaks | PASS for authorized bounded cases |
| High-impact harm | Zero judged harmful endorsements/leaks | PASS |
| Completion no worse than control | Treatment 13/15 versus control 5/15 | PASS |
| Median repeated investigation reduction ≥20% | Six eligible pairs, median 100%; no unknown natural counts | PASS |

The eight failing quality claims remain in the denominator: one supported lab
observation and two source summaries judged irrelevant; one overbroad unsupported
checklist assertion; two unsupported archival propositions explicitly requested
with `purpose: inspect` and rejected by the actor; and two operational recovery
messages judged irrelevant and unsupported. Inspection qualifiers and rejection
do not remove substantive archival claims from the metric.

Independent Astra review recommends narrow general corrections to typed session
recovery arguments, archival inspection guidance, and capture scope. It also
identified inconsistent applicability to operational error messages: clarify
that boundary prospectively, calibrate both empty-memory errors and errors
accompanying actual delivery, then rerun the complete subset. **V11 remains a
failure; none of its valid observations will be retrospectively excluded or
replaced.** No pass is projected from the proposed corrections.

Actor and judge share a model family; provider revisions are not attested.
Reused fixtures screen consistency and are not a fresh holdout. Claude's 15
cases remain skipped by user instruction, so original two-agent milestone exits
remain open. **PR #66 remains draft and is not ready to merge.**

## Preserved v10 candidate

The [v10 protocol](protocol-v10.md) freezes the next complete Codex-only run
with GPT-6-Luna at low effort in both actor arms and the calibrated judge
unchanged. Local setup identity, attested capture, recall compatibility,
dependency invalidation, transfer, and deletion corrections have received
independent review. Real older archive/server checks reject capture and all
ordinary reuse operations without queued writes, memory changes, or legacy
context delivery. The final workspace build, formatting, and Clippy with warnings
denied passed; the 14 runner tests and web contract/type/lint checks passed.
The full workspace suite passed **1,250 tests**, with zero failures or ignored
tests and a disposable PostgreSQL database configured. All six PR CI checks
passed on source `284275115644c34f95b00f00f18e4dfa5e265d62`.
[Candidate workflow 37606436010](https://github.com/cunilab/Cairn/actions/runs/37606436010)
passed all 17 jobs, including all five installed archive journeys. Its manifest,
five archive checksums, SBOM, immutable image digests, and deployed image revisions
were verified. [Bounded candidate evidence](candidate-v10-metadata.json) records
these checks separately from the runtime and semantic gates.

The first fresh runtime attempt passed browser project/token provisioning and
the empty personal/team baseline, but ordinary setup failed before any actors
ran: its artificially long private `CAIRN_HOME` exceeded macOS's Unix socket
path limit. The original response and failed records are preserved as an
infrastructure-invalid attempt, outside the scored corpus. The private harness
now reports setup refusal before reading nonexistent hook files; its focused
failure check passed. Astra approved a shorter documented `CAIRN_M2_OUT` and
a new deployment volume/account, with every generated socket path checked
before execution (maximum 73 bytes; macOS `SUN_LEN` is 104 bytes).

The new full [v10 Codex run](regression-v10-metadata.json) completed all 15
comparisons and all 15 isolated judgments, with 46 successful actor phases,
complete private traces, ordinary setup, and runtime identities verified.
Its first F1 project/token used the actual browser UI and is counted once as
shared M1/M2 evidence. Membership/token identity, remote binding, empty
personal/team domains, trusted HTTPS, accepted capture, persisted `user_report`
authorship, actual transmission, and judged useful return all passed. Successful
short-path setup does not establish support for arbitrary custom home lengths.

| Gate | V10 observation | Status |
| --- | --- | --- |
| Useful natural application ≥80% | 4/8 (50%); four capture misses retained | FAIL |
| Delivered claims relevant and supported ≥90% | Original judge: 13/18 (72.2%) | FAIL, provenance issue identified |
| Privacy | Three cases, complete probes and trace scans, zero observed leaks | PASS for authorized bounded cases |
| High-impact harm | Zero judged harmful endorsements/leaks | PASS |
| Completion no worse than control | Treatment 10/15 versus control 6/15 | PASS |
| Median repeated investigation reduction ≥20% | Seven eligible pairs, median 50%; no unresolved natural counts | PASS |

Independent Astra inspection found that all five failing quality claims were
ordinary repository `rg` output quoting historical release evidence in R3,
while current Cairn context/search returned no memory. Historical quotations
were conflated with current Cairn delivery. The original judgment, denominator,
and failing score remain preserved; v10 is not certified. Before the next run,
clarify delivery provenance and calibrate current delivery versus historical
quotations, shell transports, and selection without transmission.

The four capture misses exposed conflicting native and canonical capture
guidance. V11 derives native record/secrets guidance from the canonical contract,
preserves user choices when code agrees, and calibrates current delivery versus
historical source quotations before its fresh complete run. V10's original
judgments and failing gates remain unchanged.

## Preserved v9 candidate

The [v9 regression](regression-v9-metadata.json) used the verified macOS ARM
archive at source `2f1e043d2516060a3e3d61de5c571a382ee78a1d`. All 15 Codex
cases were attempted, with 11 valid comparisons and four quota-invalid pairs
preserved. The 15 Claude cases were skipped by user instruction. A calibrated,
isolated Codex judge completed all 15 packets; failed judge attempts were
preserved before retry. The [calibration](calibration-codex-v9-results.json)
matched all 30 reused fixtures, with zero unknown labels or observed tool calls.
Actor and judge share a model family; provider revisions are not attested.

Useful application was observed in all eight natural cases. Delivered-claim
quality was **25/31 (80.6%), below 90%**; four unsupported hazard claims remain
in the denominator even though the actors rejected them. Eight known eligible
natural pairs showed a 77.5% median reduction in repeated investigation.
Incomplete comparisons prevent certification of the full exit gates.

Candidate workflow [37448879539](https://github.com/cunilab/Cairn/actions/runs/37448879539)
passed all 17 jobs, including five installed native archive journeys. A fresh
deployment of its immutable server/web images passed the browser and native
smoke over trusted HTTPS. These checks cover the frozen parent source, not the
subsequent setup and reuse-policy changes. The PR remains draft pending a fresh
candidate run and evaluation of those changes.

## Preserved v8 attempt

The [v8 protocol](protocol-v8.md) requires another complete run with absolute
candidate MCP paths and pre-actor shell/binary identity checks. Earlier Codex
regressions could resolve an older global executable; their original scores
remain preserved as observations and cannot certify the frozen candidate.
V7 also exposed irrelevant implementation summaries and one control turn-limit
failure. V8 strengthens capture guidance and gives both arms identical larger
execution budgets without changing cases, models, rubric, or thresholds.
The user then instructed us to skip the Claude arm. A Codex-only v8 attempt
recorded all 15 Codex cases, but the provider quota caused every later phase to
exit nonzero; no v8 semantic score is reported. The 15 missing Claude cases,
the fresh full two-agent comparison, and semantic judging remain unrun. Fresh
HTTPS evidence remains pending. The PR is draft.

## Historical regression status

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

## Completed v7 regression (identity not attested)

Source `c53a57973a5bb01e7810ff32eafaab1f2be92070` produced all 30 case
records and 30 valid independent judgments. [Sanitized metadata](regression-v7-metadata.json)
preserves two invalid comparisons: R2 treatment prior and R4 control later.
Capture occurred in 14/15 natural cases, with 14/15 observed useful applications.
Delivered-claim quality was 43/61 (70.5%), below 90%; seven unsupported seeded
hazard claims remain in that denominator. Observed completion was control 12/30
and treatment 28/30, with zero judged high-impact endorsements or leaks. Twelve
known eligible repeated-investigation pairs had an observed median reduction
of 80.4%; F3/F5 counts remained unresolved. None of these observations closes
M2 because of invalid comparisons and the Codex runtime identity defect.

All five privacy cases completed their boundary probes and trace scans without
an observed leak. Candidate runtime certification is still missing for this run.
Earlier v6 privacy evidence likewise establishes only the bounded observed
paths, not execution of an attested candidate Codex frontend.

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
