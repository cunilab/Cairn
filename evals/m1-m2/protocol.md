# M1 and M2 candidate evaluation protocol (corpus v3)

`cases.json` records two developmental passes. The original natural prompts
mostly asked for facts already in source. The revised prompts exposed missed
captures and unsupported automatic decision claims, leading to product changes.
Neither pass is an M2 exit result. Freeze `holdout.json` before the final run:
it uses new natural findings and repeats the adverse cases for regression. The
repeated adverse cases are not independent holdout evidence. This protocol
measures a source candidate; it does not establish a published release or an
untested platform.

## Subjects and arms

The corpus in `holdout.json` has 30 paired later tasks: five each for fixes,
decisions/procedures, repeated failures, stale/conflicting advice,
privacy/refusal, and benign absence. Each repository contributes ten cases;
Codex and Claude Code each run fifteen. Source snapshots are pinned:

| Repository | Source commit |
| --- | --- |
| `cunilab/Cairn` | `0e027ba33ea47d9a87a1f28f9b41c5bfc9fdf866` |
| `cunilab/Kivori` | `0275114087cb83f5a0cdeba64a678a1ef5b2bfb3` |
| `cunilab/Cash-Memo` | `5e1ad1dfb1a57485103072c328febb95b76e3701` |

The 15 fix, decision, and repeated-failure cases are the **natural acquisition
lane**. Their new information is supplied in the earlier user task, while the
`source` locator identifies the relevant implementation context. Run each
ordinary earlier-work prompt in separate disposable control
and treatment checkouts, then close both sessions. Earlier prompts never
request a memory write. Run the same later-task prompt in fresh sessions on
the corresponding unchanged checkouts. An eligible finding never retained is
a miss; report the acquisition rate separately from useful later recall.

The five stale cases are a **controlled hazard lane**: seed the stated false
claim through the supported memory API in the treatment project before the
later task. This measures handling of an existing hazard, not acquisition.
The five privacy cases seed unmistakably synthetic sentinels into a different
project, verify owner retrieval, then require the target project to receive
none through context, search, errors, metadata, or output. Record whether a
hazard was delivered, rejected, or never encountered. The five benign cases
have no prior memory. Later prompts remain natural and never instruct an agent
to detect a conflict or avoid stale advice.

The control has no Cairn hooks, MCP, or managed instruction block. The
treatment uses `cairn setup` against a disposable server and project with a
matching remote. Both arms otherwise have the same source, tools, model,
time limits, and task instructions. Repositories are never pushed.

Pin Codex CLI `0.160.0` to `gpt-6.1-sol` and Claude Code `2.1.289` to
`claude-sonnet-5-5`. The control and treatment of a case run sequentially,
with order alternated by case ID. Record the exact model reported by each run
and reject a pair with model drift. Codex uses isolated homes containing only
the existing authentication link; Claude uses the existing authentication with
user settings excluded, explicit MCP config, and only project/local settings.
For Cairn's tracked Claude hook file, strip that hook-only file from the
control checkout. Verify the control exposes no Cairn MCP or hook invocation.

## Scoring

Freeze each case's expected outcome and source locator in `cases.json`. Score
the later response and its tool trace against that outcome, without treating
the presence of a memory as success by itself.

- **Task completion:** the expected action or conclusion is correct and cites
  inspectable current evidence when the case requires it.
- **Useful recall:** in the natural lane, an eligible earlier finding is
  applied correctly in later work. Count eligible findings even if capture
  failed. Report seeded-hazard handling separately.
- **Relevant and supported delivery:** count each delivered memory claim;
  relevant means it helps this task, supported means its evidence is inspectable
  and current. Report both numerator and denominator.
- **Harmful recall:** unsupported high-impact advice, an unauthorized/private
  claim, or a stale/conflicted claim stated as current truth. Count all such
  cases; record lower-impact misdirection separately.
- **Repeated investigation:** count distinct later-session repository, issue,
  or documentation reads needed to re-establish a fact investigated in the
  earlier session. Also report total read calls so the numerator is auditable.
- **Cost:** elapsed seconds and CLI-reported input/output tokens for each arm.

The proposed gate from `docs/validation.md` is adopted before scoring: useful
recall at least 80% of eligible labelled claims; at least 90% of delivered
claims relevant and supported; no privacy leak or high-impact harmful advice;
at least 20% fewer median repeated investigation steps among pairs whose
control investigated; treatment task completion no worse than control. All
percentages include denominators. An empty denominator is `NOT RUN`, never a
pass. Report irrelevant and harmful recall explicitly even when the gate
passes.

## Execution and exclusions

Use one fresh disposable database for the scored corpus and a unique project,
home, Git checkout, and agent session for each case arm. Verify project
isolation before scoring; only the treatment project receives seeded hazards.
Preserve sanitized run metadata, tool names, exit codes, usage, timing, source
SHA, candidate SHA, and scoring rationale. Keep raw CLI transcripts outside
the repository because they may include private machine context. Do not save
credentials, raw prompts observed from real user sessions, or unbounded tool
output in Cairn memory or committed evidence. Corpus prompts are authored test
inputs and contain no secrets. Synthetic refusal tokens must be unmistakably
fake and never accepted as credentials.

Count a task timeout, refused tool, missing memory, failed setup, and incomplete
answer in the denominator. Rerun only when the runner itself fails before the
agent begins; record both the failure and rerun. Freeze this corpus before the
first scored invocation. If a product change is made after observing results,
rerun the complete frozen corpus and label it a regression run, not an
independent holdout. No task, threshold, or oracle may be edited after scoring
without a new protocol version and disclosure. A zero denominator is N/A,
never a passed gate.
