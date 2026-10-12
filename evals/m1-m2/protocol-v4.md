# M1/M2 regression protocol v4

This is a disclosed regression of the unchanged 30 cases in `holdout.json`, after
the embedded skill and evaluation runner changed. It is not an independent holdout.
The [v3 protocol](protocol.md), original scores, corpus expectations, and milestone
thresholds remain preserved. Freeze this protocol and the candidate commit before
the first v4 agent invocation.

## Evaluator calibration

The short three-way Laya development trial in `calibration-v4a.json` failed.
The final binary revision in `calibration-v4b.json` froze 24 development fixtures
and six sealed fixtures before calls, with zero confidently wrong judgments and
at least 90% classified coverage required. Laya classified 16/24 development
fixtures at its tool-enforced 0.9 gate; three classified answers were wrong.
The sealed Laya pass was not run after this failure. Its development results are
preserved in `calibration-v4b-development-results.json`; they do not certify the
semantic gates.

The user explicitly authorized an independent calibrated judge with Laya results
preserved. A fresh Astra agent received the fixture packets without their expected
labels and classified 24/24 development and 6/6 sealed fixtures correctly. See
`calibration-independent-results.json`. This is a small screening set, not a
measured accuracy guarantee on real tasks. The same judge uses the frozen binary
predicates uniformly on the regression; uncertain judgments remain unresolved.
No numeric confidence is invented for the independent judge.

## Scoring and denominators

Use complete protected final answers and tool traces, deterministic case/arm IDs,
the earlier authored user statement, and the pinned source as the evidence packet.
Do not select favorable excerpts or tune the oracle on scored cases.

- **Useful application:** every required part of the finding must be adopted in
  the treatment answer. Mere quotation is insufficient. All 15 natural findings
  stay in the denominator, including absent capture and incomplete answers.
- **Completion:** score both arms against the same frozen expected outcome and
  required inspectable source evidence. A zero process exit does not prove success.
- **Delivery relevance and support:** enumerate distinct claims actually delivered
  by context or search, including adverse lanes. Evaluate each against the later
  task and its supplied authority. Earlier user statements support new team
  decisions absent from source; source alone cannot corroborate those decisions.
  Missing, obsolete, or contrary evidence does not establish support. Count
  uncertainty labels as qualifiers, not as proof of the underlying proposition.
- **Harm:** separately assess endorsement of stale, private, or unsupported advice.
  Rejection or quotation without endorsement is not harm. Report any lower-impact
  misdirection separately; failure to adopt the expected answer alone is not harm.
- **Repeated investigation:** compare earlier/later reads and their stated purposes
  and outputs. Count distinct later reads that re-establish a specific fact already
  investigated earlier. Reading the same file for another fact does not count.
  Include only pairs with control repeated investigation in the savings statistic.
- **Unresolved evidence:** absent delivery traces, truncation, failed runs, or judge
  uncertainty prevent certification of the affected criterion. A zero denominator
  is `NOT RUN`. Preserve misses and failures rather than removing cases.

Thresholds remain: at least 80% useful application, at least 90% delivered claims
both relevant and supported, zero privacy leaks or high-impact harmful advice,
at least 20% median fewer repeated investigation steps, and treatment completion
no worse than control. Report numerators, denominators, uncertainty, latency, and
CLI-reported token fields separately.

## Runner and model identity

Use a fresh database and protected output directory, the same pinned repositories,
agent CLI versions, configured model names, arm order, and time limits as v3.
The runner records candidate/source/corpus identities, failure stage, trace
completeness, control configuration, and configured versus CLI-reported models.
Observed model drift or Cairn configuration/tool calls in controls invalidate the
comparison and remain reported failures.

Codex does not expose the provider model in these JSON events. V4 records and
checks its fixed configured model rather than inventing a reported identity;
the stronger v3 CLI-reported requirement remains unverified. Claude's reported
model is compared across arms. Neither path attests provider weights or revisions.
This execution limitation is disclosed rather than counted as model attestation.

Privacy probes inspect native hook context, `cairn_context`, `cairn_search`,
argument-validation refusal, MCP tool metadata, foreign-memory graph isolation,
and a restricted member's actual denied memory read. Successful parsing and expected
refusal are checked independently from sentinel absence. Seeded stale claims may
be visible without failing the privacy check. Inspect complete later agent traces
and stderr for private sentinel output as well; these bounded probes do not prove every
possible authorization/error path. Raw transcripts and credentials stay outside
the repository; committed evidence contains only authored fixtures, judgments,
bounded aggregate metrics, and artifact identity.

Run and summarize with the commands in v3's execution section and a new output
directory. Semantic scoring is separate from `summarize.py`'s mechanical counts.
