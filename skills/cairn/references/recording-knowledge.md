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
For these records, preserve the user finding concisely and cite a source locator when useful.
Do not append a code-path narrative, current constants, or other implementation summaries.
Those details belong in the task answer; a later session asking about the choice needs the
choice itself.

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

A key that will not normalize does not lose you the memory: it is stored free-form, exactly
as it would have been before, and Cairn tells you it did that.

## Attach evidence rather than asserting importance

An `importance` hint does not establish truth, verification, or authorization.
Do not rely on it to make an unsupported claim authoritative.

Evidence may be a file, a configuration key, a Git ref, or a command outcome.
Include a bounded reference so a later session can re-check the finding.
An `importance: high` hint cannot substitute for that support.

The current explicit `cairn_remember` command cannot attach local observation IDs to
server-owned memory. Leave `evidence_observation_ids` empty; nonempty IDs are refused.
Cite a bounded source reference or observed outcome in the finding instead. This is an
agent attestation, not server verification or an attached evidence record.

An attestation — you reporting that you checked — is recorded and labelled as yours. It is
worth having, and it is not the same as a check Cairn ran itself.

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
