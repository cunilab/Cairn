# V11 full-input judge transport

This is an evaluation transport correction after actors finished. Product source,
artifacts and the semantic prompt stay frozen at `6982de122ffca71a500a048358e7a6f207543ac9`.
This document is not appended to that semantic prompt.

F1's original judge request was rejected before evaluation: the complete prompt
had 1,235,147 characters, exceeding the CLI's 1,048,576-character input limit.
Lossless outer JSON compaction still exceeded the limit; no compacted retry ran.

The pinned CLI's [app-server protocol](https://developers.openai.com/codex/app-server)
supports `thread/inject_items` to append model-visible user messages without
starting generation. [judge_transport.py](judge_transport.py) splits the exact
prompt into ordered parts, acknowledges every injection, then starts one turn.
Removing only the part headers and concatenating their bodies reconstructs the
original prompt exactly locally. Provider input is not read back or independently
hashed; this is a source reassembly check plus acknowledged ordered injections.
No input evidence is omitted, summarized or deduplicated.

Each invocation uses a fresh private home and ephemeral thread, the existing
Codex authentication, `gpt-6.1-sol`, CLI `0.160.0`, read-only sandbox and a
300-second budget. Shell, plugins, apps, multi-agent and web search are disabled.
Repository instructions are disabled and unexpected instruction sources, tool
requests, tool execution, compaction and incomplete turns invalidate the call.
Both typed compaction items and legacy `thread/compacted` notifications are
rejected. The calibrated helper originally omitted that legacy notification
guard; independent review led to an explicit guard and rechecking all ten
retained RPC logs, with zero tool or compaction events. Input framing and valid
judgments are unchanged. All RPC events, local part hashes, effective configuration and response are
retained privately. Provider model revision remains unattested.

Before retry, the transport passed all 30 unchanged binary predicate fixtures
and seven complete delivery fixtures, with expected labels withheld and opaque
IDs. The predicate screen used JSON whitespace to exceed F1's prompt length
without changing its parsed fixture objects, with a fixture crossing a chunk
boundary. Delivery packets used 1,500-character cuts to separate authority and
delivery evidence across parts. See [bounded calibration](calibration-codex-v11-transport-results.json).
This verifies transport and predicate consistency, not real-task performance.

Only the invalid F1 call was retried. The original packet and rejection were
preserved, and hashes verify that all 14 valid judgments stayed unchanged.
The [regression metadata](regression-v11-metadata.json) records the original
packet, reconstructed prompt, retry, ordered part and calibration hashes.

For a calibrated retry, use the original frozen packet and semantic instruction
files with this transport helper and the grader's `--injected-input` option:

```sh
CAIRN_M2_OUT=/private/short-eval-path python3 -B evals/m1-m2/grade.py \
  --jobs 1 --retry-failed --injected-input
```

The output and packet directories must be outside the repository and private.
The grader skips existing valid judgments and preserves invalid attempts before
retry. Do not retry a valid negative or unknown judgment.

The helper includes a runnable reconstruction and invalid-event check. Audit a
retained private RPC log without invoking the model:

```sh
python3 -B evals/m1-m2/judge_transport.py --check-log /private/path/rpc.private.jsonl
```
