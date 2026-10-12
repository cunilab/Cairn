# Recording durable knowledge

Record what a future session would otherwise have to rediscover.

## Worth recording

- **Decision** — a choice with alternatives that were rejected, and why.
- **Failure** — something that did not work, with enough detail to recognise it again.
- **Convention** — how this project does a thing, where that is not obvious from the code.
- **Procedure** — a sequence that is easy to get wrong and that you had to work out.
- **Fact** — a durable property of the system that is expensive to establish.

Keep user-supplied durable decisions and failure reports, including their identifiers and
constraints, even when the code already follows them. A later session cannot recover the
user's decision or incident history from the implementation alone. Keep the content focused
on that finding. State what was chosen or observed, including constraints and identifiers;
do not promote a proposed change or limited trial into implemented or validated behavior.
For generic clients, preserve the user finding concisely and cite a source locator when useful.
Do not append a code-path narrative, current constants, or other implementation summaries.
Those details belong in the task answer; a later session asking about the choice needs the
choice itself.

For durable project findings, preserve task-relevant requirements, counts, qualifiers,
supplied identifiers, constraints, and status at the reported scope. Keep genuine lasting
policies, including read-only restrictions, but do not record temporary task requests or
completion reports merely because they mention those details. Separate remembered intent from
current implementation. A limited observation does not establish a broader result. If independent
source inspection establishes another durable finding, record it separately with its own source
attestation; do not fold it into the user-reported decision or trial.

## Capture complete findings during ordinary work

Before finishing a task, capture the durable findings it established without
asking the human to maintain memory. Use `cairn_remember` with `action: "capture"`
and a `findings` array for up to eight project findings. Each item has its own
`type`, `content`, scope, topic/value keys and required `capture_attestation`.
The repository and current session identity belong to the enclosing call.

One item is a complete decision, observation or procedure, not necessarily one
sentence. Keep attribution, negation, counts, conditions, exceptions and ordered
steps together when they are needed to understand that finding. Put independently
useful implementation observations in separate items with `inspected_source`
support naming the revision actually read. Do not attach those observations to a
user decision under one `user_report` attestation. Existing single-record create
and supersede remain available; team policy still requires governed promotion.

For a verified native Codex turn, `user_report` content must be a verbatim complete
selected authored passage spanning complete adjacent blank-line-separated blocks.
Keep linked context, internal lines, conditions, exceptions and attribution together
in one finding; separate records must not fragment a single qualified requirement.
Do not paraphrase, crop clauses or add implementation commentary. The local daemon
checks the exact passage against this turn's safe task record; framework metadata
supplies turn identity. Absent, expired, truncated or redacted tasks, ambiguous
duplicate passages, over-limit tasks or passages larger than 2048 UTF-8 bytes
cannot support native authored-intent admission.
Report the refusal rather than shortening the finding or changing its basis to
bypass the check. Never capture an entire task, prompt or transcript automatically.
Choose only a durable statement worth retaining; prose surrounding a decision
may make a complete bounded source passage unsuitable for capture. Source integrity
does not prove durability, relevance or semantic independence between blocks.
Generic captures and independently inspected source observations retain their
existing contracts but do not receive native authored-intent coverage credit.

The batch returns indexed receipts or errors for separate operations. It is not
an all-or-nothing transaction. Inspect every outcome and retry only findings
known not to have been admitted; do not repeat the whole batch after a partial
result or an uncertain acknowledgement. Local admission is not server persistence.
Bounds and separate support do not prove semantic coherence or verification.

## Native finalization checkpoint

A supported Codex turn may resume once before finishing when it has neither
captured a finding nor reported a disposition. Record only newly established,
properly supported findings. If none exist, call `cairn_session` with
`action: "capture_disposition"` and `disposition: "no_durable_finding"`.
The native runtime supplies turn identity; never author or borrow an identifier.
Do not fabricate a finding to satisfy the checkpoint, replay an unconfirmed write,
or describe an empty disposition as useful capture. An acknowledged capture means
local durable command admission; it does not prove server persistence or complete
coverage. An unavailable checkpoint allows the coding turn to finish.

## Not worth recording

- Routine tool calls. Supported hooks capture bounded structured activity; do not duplicate it as durable knowledge.
- Restatements of what the code plainly says.
- Anything you have not actually established.
- Anything that will be false after the next commit.

## Rules that are absolute

- Never invent an evidence observation identifier. Cite only observations that exist.
- Never record secrets, credentials, tokens, raw prompts, or unbounded command output.
- Record the thing you learned, not the transcript of learning it.

## Give a durable fact a subject

A **topic key** names what the fact is about; a **value key** names what it asserts. Together
they must state the whole claim, because that pair is what Cairn compares.

```text
topic_key  infrastructure.production_database
value_key  postgresql
```

Specific enough matters in both directions. `database` is too coarse — a cache and a queue
would land in the same subject and be reported as disagreeing. `infrastructure.production_database.primary.postgresql.16.2`
is too fine — nothing else ever lands there, and the fact never meets the claim it
contradicts.

Single-record create retains its free-form fallback for keys that do not normalize.
The `capture` action requires nonblank content and valid topic/value keys for every
finding; invalid keys reject the entire array before any writes.

## Attach evidence rather than asserting importance

An `importance` hint does not establish truth, verification, or authorization.
Do not rely on it to make an unsupported claim authoritative.

Evidence may be a file, a configuration key, a Git ref, or a command outcome.
Include a bounded reference so a later session can re-check the finding.
An `importance: high` hint cannot substitute for that support.

The current explicit `cairn_remember` command cannot attach local observation IDs to
server-owned memory. Leave `evidence_observation_ids` empty; nonempty IDs are refused.

For a project record that should return in ordinary recall, add `capture_attestation`:

```json
{
  "basis": "user_report",
  "support_summary": "The user explicitly chose this database for production."
}
```

Use `user_report` only for a user-supplied fact, decision, failure, or constraint. Use
`inspected_source` only after reading a named source revision, and include both
`source_reference` and `source_revision`. When the claim depends on another project memory,
include `dependency_memory_id` naming an eligible record with no dependency of its own
(one hop maximum). Cairn records that dependency's current revision itself and withholds
the claim after the dependency changes or conflicts.

Keep the support summary and references bounded and authored. Never paste prompts,
transcripts, credentials, diffs, or command output. The authenticated actor is added by the
server, so do not send an actor, eligibility status, or verification authority.

A capture attestation is accountability: it records who reported the support and on what
basis. It is not objective proof and it does not create `VerificationAuthority::Attested`.
An arbitrary revision string, memory type, token, or the word "attested" does not establish
verification. Source changes Cairn can observe, such as a dependency revision or a recorded
verification drift, make the record ineligible for reuse. Cairn cannot independently detect
an arbitrary remote source change until fresh evidence or inspection reports it.

## When Cairn names a corroborating member

Writing a memory can come back with `corroborating_member`: the same subject, the same value,
different words. Cairn will not merge them, because only you can read both and say whether
they are one claim.

- Same claim, said differently → `reinforce` the existing memory. One claim, now recorded as
  confirmed by two sessions.
- Genuinely different claims that happen to agree on the value → leave both. The subject
  reports them as corroborating, which is what they are.

Doing nothing is the third option and it is the worst one: two memories, neither aware of the
other, both surfacing forever.

## Record what happened to a pattern

A reusable pattern from another project is a suggestion, never an answer. Check it against
this repository. If applying it establishes a durable procedure or failed approach, record
that finding through `cairn_remember` with the appropriate project, branch, or session scope
and existing evidence. The current MCP interface has no `record_outcome` action.

## Native completeness review

At a native Codex finalization checkpoint, call `cairn_session` with
`action: "capture_review"`. A classifier sees authored source blocks, not findings,
and selects `required_passages` with inclusive `first_block`/`last_block` bounds.
Keep linked cross-paragraph context in one complete finding. Rust computes
coverage from admissions carrying trusted proof for this current task digest;
separate fragments cannot jointly cover one selected passage. Implementation
inspection, stale, unproven or cropped findings cannot cover authored intent.
If missing, capture the complete selected durable passage through `user_report`
and review again. Do not automatically archive the whole local task.

The classifier can still omit requirements or misjudge cross-block dependencies.
Exact source binding prevents qualifier loss at admission; it is not semantic
qualification or proof that every requirement was identified.

Unknown coverage is not success. Missing/truncated/redacted input or an
unconfigured comparator permits the coding session to finish after one bounded
intervention; report the limitation. A queued finding or an agent-authored
`no_durable_finding` assertion alone does not establish completeness or server
persistence. Generic clients cannot claim a native turn review.
