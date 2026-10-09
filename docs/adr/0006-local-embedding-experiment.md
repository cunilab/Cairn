# 0006 — Test local embeddings before changing retrieval

**Status:** Proposed; bounded development experiment authorized; no production integration.
**Recorded:** 2026-10-09.

## Context

The user proposed local embeddings after temporary Groq qualification failed
attribution and then mixed-clause relevance. They authorized one bounded
experiment before another implementation change. [0004](0004-extractive-semantic-selection.md)
remains the current candidate architecture; its failed observations are preserved.

Agentmemory uses local `Xenova/all-MiniLM-L6-v2` embeddings and keyword/vector
ranking. Its source ranks observations or memory records. Ranking a whole mixed
record does not resolve Cairn's demonstrated irrelevant-claim problem.

## Proposed experiment

Use one query-independent sentence segmentation rule for every source record:
Node's `Intl.Segmenter` in English sentence mode. Preserve the exact source text,
UTF-8 byte spans, hashes and attribution as source evidence. Do not manually
curate units, split clauses by case, rewrite claims or silently attach omitted
source content to the delivered unit. Source context supplied to the judge is
not evidence that the whole record reached the agent.

Compare keyword BM25 with local normalized embedding similarity on identical
units, returning at most one unit per query. Use the q8 local MiniLM model and
an isolated, pinned evaluation runtime, with no production dependency. Freeze
separate eight-case retrieval calibration and 24 synthetic development cases
before execution. Choose abstention cutoffs once from calibration only; numerical
similarity is not a calibrated probability of relevance. Report calibration
failure if positive and negative examples cannot meet its predeclared criteria.
Do not adjust cutoffs or segmentation after seeing development judgments.

The independent calibrated judge scores actual delivered claims and whether an
eligible source finding remains useful. Include mixed records, same-sentence
mixed clauses, attribution across sentences, negation, conditions, paraphrase and
benign absence. Report misses, unjustified deliveries, qualifier/authority
corruption, abstention, startup/query latency, and all denominators. A fresh
synthetic screen is development evidence, not a full milestone pass.

## Advance and stop criteria

Advance only if the experiment meets the existing 90% relevant-and-supported
claim quality and 80% eligible useful-recall targets, with no qualifier or
authority corruption. Empty denominators cannot pass. If it fails, preserve the
results and stop this approach before production integration; do not add a chain
of fixture-specific boundary rules or tune the development set into a pass.

If it passes, independently review the smallest compatible production design,
then integrate it while preserving authorization, revision checks, source
provenance, query boundaries and safe refusal. Only evidence supporting that
replacement can supersede [0004](0004-extractive-semantic-selection.md). Freeze
new source/artifacts and run the unchanged complete Codex milestone workload and
fresh installed deployment journeys. Claude remains skipped by user instruction.

## Alternatives and uncertainty

Another hosted model remains a possible future approach, but is not selected by
this experiment. Whole-record embeddings leave mixed claims together. A local
reranker also depends on faithful input units; omit it from the first experiment
to isolate segmentation and retrieval. Safe, self-contained unit construction
is the central unproven dependency, especially within sentences and across
sentence boundaries. Passing this small experiment does not establish paired
agent task completion, privacy, or deployment support.

## Evidence

- [Groq development failures](../../evals/m1-m2/groq-development-metadata.json).
- [Agentmemory local embedding source](https://github.com/rohitg00/agentmemory/blob/da91cc05b3c79c59f6c0480f728bb9c09127e000/src/providers/embedding/local.ts).
- [Agentmemory hybrid ranking source](https://github.com/rohitg00/agentmemory/blob/da91cc05b3c79c59f6c0480f728bb9c09127e000/src/state/hybrid-search.ts).
- The user's approval to execute the bounded plan; Astra's independent advice
  preserves the existing gates and requires automatic, query-independent units.
