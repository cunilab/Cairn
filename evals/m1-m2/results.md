# M1/M2 candidate evidence — through 2026-10-10

## Finding capture development — autonomous capture not observed

At `b526e64cbc3d80c5e5680b049eb65649b3dd18cc`, a fresh isolated F1
run used the unchanged frozen corpus, pinned Codex 0.160.0 and GPT-6-Luna/low.
The earlier treatment agent made task-bound context/search calls but no
`cairn_remember` call. Server persistence contained zero treatment memories.
The later agent also issued task queries; there was no captured decision to use.
The runner's trace-valid result is not a semantic or milestone pass.

The preregistered stop rule applies: no prompt, corpus or provider iteration,
no unseen-case qualification or full workload advancement, and no semantic
rescoring. This identifies a remaining automatic-capture problem; it does not
show that the new batch API or the selector failed on separately captured facts.
The batch's 24 unit tests, lost-ack regression, actual MCP/fake-socket adapter,
contract rendering and Skill revision checks pass. Independent Sol review passed
after correcting blank fields and duplicate replay risk.

The installed alpha.8 client/daemon mismatch was repaired with matching alpha.9
binaries. A real context MCP call now renders without `missing field briefing`,
but durable server knowledge remained unavailable; full runtime health is not
claimed. [Metadata](finding-capture-development-metadata.json) preserves source,
binary and private evidence hashes. M1/M2 remain open; Claude remains skipped.

## Local embedding development — advance criteria failed

The frozen experiment at `73959a1206d2a3782c93a4e0db679b01cde78b74`
compared BM25 with q8 local MiniLM on identical automatic sentence units.
Separate eight-case calibration passed: four positive hits and no deliveries
on four no-match cases. All 24 development cases preserved exact UTF-8 spans.
The isolated independent judge scored all 48 actual outputs, with no unknowns.

| Arm | Relevant and supported claims | Useful retrieval / eligible sources | Condition/authority corruption |
| --- | --- | --- | --- |
| BM25 | 21/26 (80.8%) | 10/20 (50%) | 1 |
| Local embeddings | 20/22 (90.9%) | 10/20 (50%) | 1 |

Embeddings met the 90% claim-quality screen but failed the 80% useful-retrieval
screen and zero-corruption requirement. Both arms selected a migration permission
without its preceding production-only scope. Other useful-retrieval failures
omitted needed exceptions, actor information or complementary requirements.
The predeclared stop rule applies: no production integration or further tuning.

The cached-model run took 958 ms (332 ms model startup, 256 ms batched embedding);
these are standalone measurements, not production latency. The judge reused
42/42 calibrated relevance/support predicates. Useful retrieval is a synthetic
proxy, not separately calibrated agent application or full milestone evidence.
[Metadata](local-embedding-development-metadata.json) preserves source/runtime
identities, denominators, transport proof and private evidence hashes.
[ADR 0006](../../docs/adr/0006-local-embedding-experiment.md) records the outcome.
M2 remains open; Claude remains skipped.

## Groq development — model qualification stopped

The temporary `openai/gpt-oss-120b` endpoint authenticated successfully and five
synthetic cases passed the actual server's source-hash/span checks. The isolated
`gpt-6.1-sol` judge passed all 42 reused calibration fixtures, zero unknown
labels, with all 13 transport/isolation proofs verified. It rejected one output
for removing requirement attribution. Astra recommended one general instruction
clarification: choose complete self-contained source statements preserving who
requires/reports them and intent versus implementation.

At product source `5bb16df196d262c9def78115ed8d193ad7130b08`, six unseen
synthetic cases passed provenance checks. The judge found all six faithful but
only five relevant: the mixed-clause case also delivered an unrelated sidebar
claim. A harness-only incomplete attempt mishandled the valid empty result and
is preserved; the full screen repeated after correcting serialization only.
The [development metadata](groq-development-metadata.json) preserves both valid
screens and private evidence hashes. These are development cases, not milestone
comparisons or a new holdout. The model is **not qualified**; no further prompt
tuning or full actor batch ran. Candidate workflow 37951958706 was cancelled
after the source changed. A different model must qualify before full M1/M2
evidence. No production provider configuration has changed; Claude is skipped.

## Extractive selector — mechanical checks passed; live evidence pending

Product source `4c77bc5dcfe5676c73d1f17fb8f3a42e2c4b1e7d` implements the
server-owned selector recommended by Astra. Context, working search and reuse
detail share exact-source excerpt selection; archive originals remain intact.
Schema 10 records source and excerpt hashes with byte spans. Automatic pins and
warnings contain references/status only. Clients require `task_excerpts_v1`
and the matching task-query digest; legacy unconfirmed bodies are withheld.
Membership and source eligibility are checked again after inference, using a
fresh membership query rather than the cached reader context. Provider errors,
invalid output and capacity exhaustion refuse recall without whole-record fallback.

Local verification passed **465 affected unit tests**, **14 evaluation-runner
tests**, workspace Clippy with warnings denied, formatting, diff whitespace and
Compose configuration validation. Targeted PostgreSQL checks passed 81 tests
across eight groups during development. The final server passed all five
`task_recall` tests, including source/membership races and provenance constraint
rejection; the final affected journey, transfer, delivery and trace batch passed
28 tests. The other 48 targeted checks passed earlier in this development cycle.
Astra independently reviewed the selector and subsequent boundary fixes.

The first latest-head CI attempt at `2e9babd1` passed five jobs but Linux
failed during an ingest fixture's initial account sign-in. The exact test passed
locally. Independent Sol review identified a harness startup race: parallel
fixtures could probe the same free port and accept a sibling's health response
before their own child finished startup. Account creation used the intended
database while login reached the sibling. The harness now serializes listener
allocation through readiness and binds its selector before probing the server
port. The second CI attempt confirmed an endpoint mix-up: login returned the fake
selector response. With the fix, all 30 local PostgreSQL ingest tests and
end-to-end crate Clippy passed; Sol independently reviewed startup/restart
locking. Production authentication is unchanged; both failed attempts remain
preserved in [CI 37946831030](https://github.com/cunilab/Cairn/actions/runs/37946831030).

These fixtures use a fake inference endpoint and establish mechanical behavior,
not semantic quality. The collector preserves actual excerpt selection and
warns judges not to treat the original canonical record as delivered content.
No frozen evaluation result, rubric or threshold has been changed.

The [ADRs](../../docs/adr/README.md) record architecture and evaluation decisions;
the [inference guide](../../docs/inference.md) documents configuration and data
sent to the provider. The user will supply the real endpoint, model and private
credential later. No live selector evaluation, fresh full 15-case comparison,
or fresh installed release/HTTPS deployment has run for this source. Claude
remains skipped. **PR #66 remains draft and is not ready to merge.** Historical
M1 success and M2 failure below remain tied to their original revisions.

## Task-query development — code verified; M2 remains open

Source `f44448eb7a09f88b6ff829052c3661dd52fea1f3` separates automatic
continuity from project-finding delivery. Startup advertises eligible memory
availability; explicit bounded task keywords select whole eligible records.
The query travels through MCP, wire, daemon and server, with a policy and query
digest checked before delivery. Queried context bypasses the outage cache;
older unqueried project bodies and stale availability are withheld. Lexical
overlap ranks ahead of recency and the candidate limit. Archive inspection
remains available.

Independent Sol review passed after correcting relevance ordering and cached
availability. Local verification passed **390 affected unit tests and 56
PostgreSQL tests**, including the installed MCP → daemon → server path,
legacy/query-digest refusal and cache isolation. Formatting and Clippy passed.
[CI 37884859306](https://github.com/cunilab/Cairn/actions/runs/37884859306)
passed **all six jobs**; its exact-source Linux workspace run passed **1,262
tests in 60 groups**, with zero failures or ignored tests.

Two prospective development examples in separate repositories used unchanged
capture guidance, ordinary setup and the pinned GPT-6-Luna / low actor. The
isolated calibrated `gpt-6.1-sol` judge counted all actual later memory claims:
useful application **2/2**, relevant and supported claims **10/10**, completion
treatment **2/2** versus control **0/2**, and no high-impact harm. N2's
lower-impact misdirection occurred in control. All eight processes succeeded,
but only **seven phase traces were complete**: N1's control prior context
capture was incomplete, so its repeated-investigation control count is unknown.
N2 repeated investigation was control three versus treatment zero. Three failed
Cairn MCP calls remain preserved. Reused calibration, transport-helper identity,
input reconstruction, runtime identities, timings, usage and private evidence
hashes are recorded in the [development metadata](task-recall-development-metadata.json).
This is development evidence, not a full milestone evaluation or fresh holdout.

Astra inspected both judged examples and persisted captures. Both captured clean
decisions; neither exercised V13's mixed-record failure. A PostgreSQL regression
explicitly demonstrates that a matching record still delivers an appended
implementation proposition. The original V13 failure remains unchanged below.
Astra therefore advised against another full batch before correcting selection
within records. Its recommended next step is a server-owned, task-conditioned
semantic selector that returns existing relevant spans, preserves attribution
and necessary qualifiers, keeps originals intact, records actual excerpt
transmission, and abstains on failure. Both working search and context need the
same boundary. A usable inference endpoint, model and server credential
arrangement are pending; none is configured in this environment.

**PR #66 remains draft and is not ready to merge.** The full 15-case evaluation
and fresh release/HTTPS deployment have not been rerun for `f44448eb`. Claude
remains skipped by user instruction. V13's installed-artifact M1 pass is
historical evidence for `58107d9`, not proof of the new source's full journey.

## Final v13 candidate — M1 passed; M2 claim quality failed

Product source is frozen at `58107d941ce6b161542d8651ab4efc395ada8e3e`.
Astra challenged repeated instruction tuning and identified native session
identity as infrastructure. V13 binds the observed Codex CLI `0.160.0` profile
to actual per-call framework thread metadata, overriding invented model keys.
Explicit Cairn UUIDs remain validated against that key, project and worktree;
missing or malformed native metadata is refused. Other client profiles retain
explicit selection. MCP cannot spoof a native agent to start lifecycle sessions.
Local-client metadata establishes provenance, not authentication.

An [upstream identity report](https://github.com/openai/codex/issues/19937)
describes the same MCP/hook correlation problem. The fix uses the
[pinned implementation](https://github.com/openai/codex/blob/rust-v0.160.0/codex-rs/core/src/mcp_tool_call.rs),
confirmed by actual local calls. A non-corpus pinned CLI → development Cairn
stdio → mock-daemon check overrode an invented key with the actual thread;
subsequent UUID validation and lifecycle refusal were tested separately.
Independent review passed after closing the lifecycle bypass. Local checks
passed 358 affected-package tests, 14 runner checks, formatting and Clippy.

[Candidate workflow 37772527397](https://github.com/cunilab/Cairn/actions/runs/37772527397)
passed **all 17 jobs**, both images and five installed archive journeys.
Its exact-source workspace suite passed **1,256 tests**, zero failed or ignored.
Checksums, SBOM, native identities and actual fresh HTTPS deployment image
revisions were verified. All six checks passed on the evaluation-helper commit
`2d4c4f0`; its product tree matches the candidate. Reporting commits do not
change the frozen product or artifact identity.

The [final protocol](protocol-v13.md) preserves the corpus, prompts, profiles,
budgets and thresholds. Both actor arms used **GPT-6-Luna / low**, CLI `0.160.0`.
The isolated `gpt-6.1-sol` judge passed [42/42 reused calibration fixtures](calibration-codex-v13-results.json)
with zero unknown labels and all 13 transport proofs verified. Two earlier
[procedure-invalid predicate calls](invalid-v13-calibration-metadata.json)
remain preserved: a private wrapper omitted frozen response instructions.
Restoring the exact original input and constraining the response schema corrected
serialization prospectively; neither invalid call counts as a calibration pass.
Delivery judging uses the unchanged frozen helper.

The fresh browser project/token journey and ordinary setup passed, including
nine absolute native hooks, account/membership ownership and empty personal/team
domains. The complete run produced **15 valid comparisons, 46 successful phases
and 15 valid judgments**, with complete traces and verified runtime identities.
Two judgments completed before quota blocked the remaining
[13 calls](invalid-v13-judge-quota-metadata.json). After the user reported quota
recovery, a non-corpus availability check passed and only those invalid calls
resumed. All 15 actor results, 15 complete packets and both valid judgments
remained byte-for-byte unchanged; original quota calls are preserved.
An earlier readiness process ended before completion with nearly full host disk;
its cause remains unestablished and its original logs remain preserved. Generated
build/package caches were cleaned; private evidence was retained.
All 15 final judge RPC/config logs passed isolation, input and compaction checks.

| Gate | V13 observation | Status |
| --- | --- | --- |
| M1 prior capture → persisted ownership → later useful delivery | One accepted prior creation correlated to one owned eligible user-report record, actual later delivery and useful application | PASS for Codex journey |
| Useful natural application ≥80% | 8/8 (100%) | PASS |
| Delivered claims relevant and supported ≥90% | 31/42 (73.8%) | **FAIL** |
| Privacy | Three complete bounded probes and trace scans, zero observed leaks | PASS for bounded cases |
| High-impact harm | Zero judged harmful endorsements/leaks | PASS |
| Completion no worse than control | Treatment 14/15 versus control 5/15 | PASS |
| Median repeated investigation reduction ≥20% | Seven eligible pairs, median 100%; no unresolved natural counts | PASS |

The [candidate proof](candidate-v13-metadata.json) correlates F1's accepted prior
receipt with its authenticated server result ID. The owned eligible `user_report`
record appears in actual later Cairn context with matching selected revision and
independently judged useful application. Rendered-hook matching is conservative
content/budget/delivery-point correlation, not an exact trace-ID receipt.
Astra reviewed this proof before actors. F1 is counted once as shared M1/M2
evidence; unrelated later source records neither satisfy nor invalidate it.

The [complete gate results](regression-v13-metadata.json) contain no unresolved
labels or inventories. All **11 failed quality claims are supported but
irrelevant** implementation propositions bundled with useful user decisions:
D2 has three token/setup claims, D4 two recurrence-UI claims, R3 four release
workflow claims, and R5 two offline-editor claims. Astra inspected the prior
capture arguments and actual later developer payloads and confirmed these are
persisted memory claims, not operation-generated metadata or an inventory error.
Capture and selection operate on a whole record, so selecting its useful decision
also delivers all appended implementation commentary. Five failed Cairn MCP calls
across four case phases remain reported as operational evidence, including
intentional refusals; no added severity gate or silent exclusion is applied.

Primary guidance on [unambiguous agent tools](https://www.anthropic.com/engineering/writing-tools-for-agents),
[small, high-signal context](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents),
and [evaluating actual agent outcomes](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)
informed the investigation. Those sources do not establish Cairn's gates.
A structural next step is separately attributable and independently selectable
reported decisions and inspected implementation findings, with validated capture
boundaries. No further wording-tuning cycle or retrospective rescore follows this
frozen run; no valid observation is relabelled or replaced.

**PR #66 remains draft and is not ready to merge under the requested M1/M2 gates.**
M1's Codex journey passed; M2's quality gate failed. Claude remains skipped by
user instruction, so original two-agent exits remain open. Calibration is a
reused consistency screen; the corpus is developmental rather than a fresh
holdout. Actor and judge share a model family. Provider revisions and input
readback remain unattested; exact local reconstruction and acknowledged injection
do not establish either. No production tag is promoted.

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

### Native checkpoint diagnostic — 2026-10-10

Frozen `381eee31` F1 persisted three memories but no checkpoint rows: Stop
dispatch bypassed the new path and native metadata was an object, not the
assumed string. Both mechanical errors have regression fixes. The calibrated
judge marked completion and useful application `no`; 6/9 delivered claims were
relevant and supported. The authored credential-restoration requirement was
not captured. Qualification stops; repairing credit cannot recover omitted
intent. See [metadata](native-checkpoint-development-metadata.json) and
[ADR 0009](../../docs/adr/0009-bounded-native-finalization-checkpoint.md).

Frozen `043a6c2a` then passed the bounded mechanical repair check: an ordinary
OK turn acknowledged no finding after one intervention and finished; an
authored team-decision turn received exact-turn capture credit and persisted
one memory. Both keys match native turn context. See
[mechanical metadata](native-checkpoint-mechanical-metadata.json). This does
not change the failed F1 semantic result or close full milestone evidence.

### Local capture completeness — 2026-10-10

Implemented at frozen `d3303b3a` after explicit user approval of bounded
redacted local task records. Mechanical checks passed: 53 CLI tests, 49 store
tests, 64 daemon tests, Skill revision check, formatting/whitespace and affected
four-package Clippy. Independent review identified unbounded snapshots; the
content/payload/turn limits and atomic refusal regression pass. Final independent
review remains incomplete due reviewer quota.

The actual Rust comparator's frozen synthetic screen matched 11/12 fixture
predicates with zero unknowns. Groq falsely credited a finding that dropped
the restriction to isolated integration tests. Qualification stops; no native
or full semantic workload advances after failure, and no further prompt/model
iteration is claimed. This is development screening, not independent milestone
scoring. See [metadata](capture-completeness-development-metadata.json) and
[ADR 0010](../../docs/adr/0010-capture-completeness-boundary.md).

The subsequent independent privacy review found and corrected credential-lane
purge/admission races and deleted task bytes retained in SQLite WAL. Final
correction checks passed: 51 store tests, 66 daemon tests (one live-provider
test excluded), affected-package Clippy, two deterministic credential race
regressions, and file-backed deletion/busy-checkpoint retry regressions. Sol's
bounded independent review found no remaining mechanical integration blocker.
All six remote CI jobs passed preceding head `df6adecc`; correction-head CI
remains a separate gate. Semantic qualification remains failed and no full
milestone pass is claimed.
