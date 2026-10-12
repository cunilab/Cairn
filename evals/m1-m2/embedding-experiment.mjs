#!/usr/bin/env node
// A standalone, frozen experiment. It never participates in Cairn retrieval.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { isAbsolute, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const CASES_PATH = resolve(SCRIPT_PATH, '..', 'embedding-experiment-cases.json');
const MODEL = 'Xenova/all-MiniLM-L6-v2';
const DIMENSIONS = 384;
const DTYPE = 'q8';
const SEGMENTATION_POLICY = 'Intl.Segmenter(en, sentence), exact UTF-8 byte spans';

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function usage(message) {
  throw new Error(`${message}\nUsage: CAIRN_EMBEDDING_RUNTIME=/absolute/path/to/transformers.mjs CAIRN_EMBEDDING_CACHE=/private/cache node evals/m1-m2/embedding-experiment.mjs --out /absolute/private/output-directory`);
}

function args() {
  const values = process.argv.slice(2);
  if (values.length === 1 && values[0] === '--self-check') return { selfCheck: true };
  if (values.length !== 2 || values[0] !== '--out' || !isAbsolute(values[1])) usage('--out must be an absolute, new directory');
  return { out: values[1] };
}

function requireRuntime() {
  const runtime = process.env.CAIRN_EMBEDDING_RUNTIME;
  const cache = process.env.CAIRN_EMBEDDING_CACHE;
  const revision = process.env.CAIRN_EMBEDDING_REVISION;
  if (!runtime || !isAbsolute(runtime)) usage('CAIRN_EMBEDDING_RUNTIME must be an absolute module path');
  if (!cache || !isAbsolute(cache)) usage('CAIRN_EMBEDDING_CACHE must be an absolute private cache directory');
  if (!revision) usage('CAIRN_EMBEDDING_REVISION must pin the model revision');
  return { runtime, cache, revision };
}

function sentenceUnits(scenario) {
  const segmenter = new Intl.Segmenter('en', { granularity: 'sentence' });
  return scenario.sources.flatMap((source) => {
    const units = [];
    for (const segment of segmenter.segment(source.text)) {
      if (!segment.segment.trim()) continue;
      const startByte = Buffer.byteLength(source.text.slice(0, segment.index), 'utf8');
      const endByte = startByte + Buffer.byteLength(segment.segment, 'utf8');
      const selected = Buffer.from(source.text, 'utf8').subarray(startByte, endByte).toString('utf8');
      if (selected !== segment.segment) throw new Error(`UTF-8 span round-trip failed for ${scenario.id}/${source.id}`);
      units.push({
        id: `${source.id}:${units.length}`,
        source_id: source.id,
        text: segment.segment,
        start_byte: startByte,
        end_byte: endByte,
        source_content_sha256: sha256(source.text),
        selected_content_sha256: sha256(selected),
        metadata: source.metadata,
      });
    }
    if (units.length === 0) throw new Error(`No complete sentence for ${scenario.id}/${source.id}`);
    return units;
  });
}

function tokens(text) {
  return (text.toLowerCase().match(/[\p{L}\p{N}]+/gu) || []);
}

function bm25(query, units) {
  const docs = units.map((unit) => tokens(unit.text));
  const lengths = docs.map((doc) => doc.length);
  const averageLength = lengths.reduce((sum, value) => sum + value, 0) / docs.length;
  const documentFrequency = new Map();
  for (const document of docs) for (const token of new Set(document)) documentFrequency.set(token, (documentFrequency.get(token) || 0) + 1);
  const queryTokens = [...new Set(tokens(query))];
  return docs.map((document, index) => {
    const frequency = new Map();
    for (const token of document) frequency.set(token, (frequency.get(token) || 0) + 1);
    return queryTokens.reduce((score, token) => {
      const tf = frequency.get(token) || 0;
      if (!tf) return score;
      const idf = Math.log(1 + (docs.length - (documentFrequency.get(token) || 0) + 0.5) / ((documentFrequency.get(token) || 0) + 0.5));
      return score + idf * (tf * 2.2) / (tf + 1.2 * (1 - 0.75 + 0.75 * lengths[index] / averageLength));
    }, 0);
  });
}

function best(scores, units, cutoff) {
  let index = 0;
  for (let candidate = 1; candidate < scores.length; candidate += 1) if (scores[candidate] > scores[index]) index = candidate;
  const score = scores[index];
  return score >= cutoff ? { unit: units[index], score } : { unit: null, score };
}

function nextUp(value) {
  if (!Number.isFinite(value)) return Infinity;
  if (value === 0) return Number.MIN_VALUE;
  const bytes = new ArrayBuffer(8);
  const view = new DataView(bytes);
  view.setFloat64(0, value);
  let bits = view.getBigUint64(0);
  bits += value > 0 ? 1n : -1n;
  view.setBigUint64(0, bits);
  return view.getFloat64(0);
}

function calibrate(results) {
  const positives = results.filter((result) => result.expected_unit_ids.length > 0);
  const negatives = results.filter((result) => result.expected_unit_ids.length === 0);
  const negativeMaximum = Math.max(...negatives.map((result) => result.embedding_top.score));
  const candidates = [...new Set([nextUp(negativeMaximum), ...positives.map((result) => result.embedding_top.score).filter((score) => score > negativeMaximum)])].sort((a, b) => a - b);
  const attempts = candidates.map((cutoff) => {
    const eligibleHits = positives.filter((result) => result.expected_unit_ids.includes(result.embedding_top.unit.id) && result.embedding_top.score >= cutoff).length;
    const negativeDeliveries = negatives.filter((result) => result.embedding_top.score >= cutoff).length;
    return { cutoff, eligible_hits: eligibleHits, eligible_total: positives.length, negative_deliveries: negativeDeliveries };
  }).filter((attempt) => attempt.negative_deliveries === 0);
  if (attempts.length === 0) return { status: 'calibration_failed', reason: 'no numeric cutoff abstains on every calibration no-match', negative_maximum: negativeMaximum };
  attempts.sort((left, right) => right.eligible_hits - left.eligible_hits || left.cutoff - right.cutoff);
  const chosen = attempts[0];
  if (chosen.eligible_hits !== positives.length) return { status: 'calibration_failed', reason: 'positive and negative examples cannot fully separate with one numeric cutoff', negative_maximum: negativeMaximum, chosen };
  return { status: 'calibrated', negative_maximum: negativeMaximum, chosen };
}

function serialiseScenario(scenario, units, lexicalScores, embeddingScores, embeddingCutoff) {
  const lexicalTop = best(lexicalScores, units, Number.MIN_VALUE);
  const embeddingTop = best(embeddingScores, units, embeddingCutoff);
  return {
    id: scenario.id,
    query: scenario.query,
    records: scenario.sources.map((source) => ({ id: source.id, content: source.text, content_sha256: sha256(source.text), attribution: source.metadata })),
    units,
    arms: {
      bm25: { top_score: lexicalTop.score, delivered: lexicalTop.unit ? [lexicalTop.unit] : [] },
      embedding: { top_score: embeddingTop.score, delivered: embeddingTop.unit ? [embeddingTop.unit] : [] },
    },
    provenance_valid: units.every((unit) => unit.selected_content_sha256 === sha256(unit.text) && unit.end_byte > unit.start_byte),
  };
}

function validateCorpus(corpus) {
  if (corpus.calibration.length !== 8 || corpus.evaluation.length !== 24) throw new Error('Frozen corpus must contain 8 calibration and 24 evaluation scenarios');
  const ids = new Set();
  for (const scenario of [...corpus.calibration, ...corpus.evaluation]) {
    if (ids.has(scenario.id)) throw new Error(`Duplicate scenario ${scenario.id}`);
    ids.add(scenario.id);
    const units = sentenceUnits(scenario);
    const unitIds = new Set(units.map((unit) => unit.id));
    for (const expected of scenario.expected_unit_ids) if (!unitIds.has(expected)) throw new Error(`Unknown expected unit ${expected} in ${scenario.id}`);
    if (scenario.eligible !== undefined && scenario.eligible !== (scenario.expected_unit_ids.length > 0)) throw new Error(`Eligibility label disagrees with expected units in ${scenario.id}`);
  }
}

async function main() {
  const options = args();
  const rawCases = readFileSync(CASES_PATH, 'utf8');
  const corpus = JSON.parse(rawCases);
  validateCorpus(corpus);
  if (options.selfCheck) {
    process.stdout.write(`${JSON.stringify({ status: 'self_check_passed', calibration_cases: corpus.calibration.length, evaluation_cases: corpus.evaluation.length, segmentation_policy: SEGMENTATION_POLICY })}\n`);
    return;
  }
  const { out } = options;
  const { runtime, cache, revision } = requireRuntime();
  mkdirSync(out, { recursive: false, mode: 0o700 });
  const runStartedAt = performance.now();
  const { pipeline, env } = await import(runtime);
  env.cacheDir = cache;
  const startupStartedAt = performance.now();
  const extractor = await pipeline('feature-extraction', MODEL, { dtype: DTYPE, cache_dir: cache, revision });
  const startupMs = Math.round(performance.now() - startupStartedAt);
  async function vectors(texts) {
    const output = await extractor(texts, { pooling: 'mean', normalize: true });
    const rows = Array.from(output.data);
    if (rows.length !== texts.length * DIMENSIONS) throw new Error(`Expected ${DIMENSIONS}-dimension normalized embeddings`);
    return texts.map((_, index) => rows.slice(index * DIMENSIONS, (index + 1) * DIMENSIONS));
  }
  function score(scenario, units, query, documentVectors, cutoff) {
    const embeddingScores = documentVectors.map((vector) => vector.reduce((sum, value, index) => sum + value * query[index], 0));
    const lexicalScores = bm25(scenario.query, units);
    const initial = serialiseScenario(scenario, units, lexicalScores, embeddingScores, cutoff);
    return { ...initial, _embedding_scores: embeddingScores, _expected_unit_ids: scenario.expected_unit_ids };
  }
  const prepared = [...corpus.calibration, ...corpus.evaluation].map((scenario) => ({ scenario, units: sentenceUnits(scenario) }));
  const embeddingStartedAt = performance.now();
  const queryVectors = await vectors(prepared.map(({ scenario }) => scenario.query));
  const documentVectors = await vectors(prepared.flatMap(({ units }) => units.map((unit) => unit.text)));
  const batchedEmbeddingMs = Math.round(performance.now() - embeddingStartedAt);
  let vectorOffset = 0;
  const allResults = prepared.map(({ scenario, units }, index) => {
    const scoringStartedAt = performance.now();
    const unitVectors = documentVectors.slice(vectorOffset, vectorOffset + units.length);
    vectorOffset += units.length;
    return { ...score(scenario, units, queryVectors[index], unitVectors, -Infinity), scoring_and_ranking_ms: Number((performance.now() - scoringStartedAt).toFixed(3)) };
  });
  const calibrationRaw = allResults.slice(0, corpus.calibration.length);
  const calibrationForCutoff = calibrationRaw.map((result) => ({ expected_unit_ids: result._expected_unit_ids, embedding_top: { unit: result.units[result._embedding_scores.indexOf(Math.max(...result._embedding_scores))], score: Math.max(...result._embedding_scores) } }));
  const calibration = calibrate(calibrationForCutoff);
  const cutoff = calibration.status === 'calibrated' ? calibration.chosen.cutoff : Infinity;
  const calibrationResults = calibrationRaw.map((result) => {
    const top = best(result._embedding_scores, result.units, cutoff);
    return { ...result, arms: { ...result.arms, embedding: { top_score: top.score, delivered: top.unit ? [top.unit] : [] } } };
  });
  const evaluationResults = allResults.slice(corpus.calibration.length).map((result) => {
    const top = best(result._embedding_scores, result.units, cutoff);
    return { ...result, arms: { ...result.arms, embedding: { top_score: top.score, delivered: top.unit ? [top.unit] : [] } } };
  });
  for (const result of [...calibrationResults, ...evaluationResults]) {
    delete result._embedding_scores;
    delete result._expected_unit_ids;
  }
  const report = {
    schema: 1,
    experiment: 'standalone local sentence-unit retrieval comparison',
    status: calibration.status,
    limitation: 'This reports retrieval and provenance only. It does not score semantic claim quality, useful recall, or qualifier preservation and cannot establish an M2 gate.',
    frozen_corpus_sha256: sha256(rawCases),
    script_sha256: sha256(readFileSync(SCRIPT_PATH)),
    segmentation_policy: SEGMENTATION_POLICY,
    model: { id: MODEL, revision, dtype: DTYPE, pooling: 'mean', normalized: true, dimensions: DIMENSIONS, runtime_module_sha256: sha256(readFileSync(runtime)) },
    calibration,
    startup_ms: startupMs,
    batched_embedding_ms: batchedEmbeddingMs,
    elapsed_ms: Math.round(performance.now() - runStartedAt),
    calibration_results: calibrationResults,
    evaluation_results: evaluationResults,
  };
  writeFileSync(resolve(out, 'embedding-experiment-results.private.json'), `${JSON.stringify(report, null, 2)}\n`, { mode: 0o600, flag: 'wx' });
  process.stdout.write(`${JSON.stringify({ status: report.status, output: resolve(out, 'embedding-experiment-results.private.json'), elapsed_ms: report.elapsed_ms, calibration: report.calibration })}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error.stack || error.message}\n`);
  process.exitCode = 1;
});
