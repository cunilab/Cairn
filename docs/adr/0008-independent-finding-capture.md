# 0008 — Capture independently supported findings through existing writes

**Status:** Implemented; structural checks passed; autonomous capture screen failed; M2 remains open.
**Recorded:** 2026-10-10.

## Context and decision

The user authorized the implementation recommended by
[0007](0007-agentmemory-workflow-reassessment.md). Historical failures originated
in agent-authored paragraphs combining user intent with implementation details.
Another description change alone would leave the same single-record incentive.

Add `cairn_remember(action: "capture", findings: [...])` for one to eight project
findings. Each item carries its own type, scope, content, topic/value keys and
required capture attestation. Optional scope keys retain the existing server and
session defaults. Keep each complete finding, including necessary conditions,
exceptions and attribution, together; do not equate a finding with a sentence.
Limit each content field to 2,048 UTF-8 bytes and validate the entire array before
sending any write. Refuse unknown or misplaced fields rather than silently
dropping their meaning. Require nonblank content and canonical, bounded topic/value
keys through the existing strict normalizers.

Each item uses the existing `MemoryCapture` request, durable command queue and
server admission path. Do not introduce another memory store, database migration
or an author-supplied atomicity flag. Existing single create and supersede remain
available; personal knowledge and governed team promotion keep their existing
interfaces. No batch item can choose another authenticated actor.

Return indexed receipts and bounded errors. Admission is sequential, not a
transaction spanning the batch. An IPC failure may have lost an acknowledgement;
report uncertain admission and never instruct a whole-batch replay. Each new batch
item makes one IPC request attempt without automatic replay after a lost reply.
Existing
admitted commands retain their own identities through delivery retries. Repeating
the MCP tool invocation is not a batch idempotency guarantee.

Update the generated agent contract and installed Skill together: agents capture
durable findings during ordinary work and issue short task-bound context queries
without asking the human to maintain memory. Native unqueried context remains
continuity-only. Raw task prompts and transcripts remain prohibited.

## Retrieval boundary and alternatives

Separate authored items give the existing retrieval path independently attributable
units to select and budget. Bounds and provenance do not prove semantic coherence
or relevance. The existing selector remains in place; this change neither enables
a model-free bypass nor qualifies a provider. Legacy mixed records remain eligible
for adverse evaluation. Existing context assembly already supports several records;
do not replace it with a one-sentence/top-one pipeline.

A new batch wire protocol and database transaction were rejected for this bounded
change: existing per-record admission and explicit receipts suffice. Automatic
sentence splitting would reproduce the observed dependency loss. An author flag
that bypasses semantic selection would assert a guarantee the write contract does
not establish. Another provider trial does not repair capture granularity.

## Verification requirements

Verify all-item preflight, item bounds, independent provenance, preserved multi-
sentence content, prohibited identity/unknown fields, and per-item outcomes.
Verify the generated instruction bounds and Skill revision. Exercise the MCP
adapter and restore the real context call with matching installed binaries.

Semantic acceptance requires fresh ordinary agent captures, unseen legacy/mixed
and dependency cases, and the unchanged full M1/M2 evidence. Unit tests or batch
receipt counts cannot close those gates. Preserve the original judgments and
Claude's explicitly skipped status.

## Implementation evidence

The MCP preflight tests pass (24 tests), including UTF-8 byte boundaries, blank
field rejection, canonical keys, separate support and preserved conditions. The
generated contract rendering tests pass (10 tests); the declared Skill revision
is `0a10aaefc831`. These establish structural behavior, not semantic usefulness.

Independent review identified automatic IPC retries as a duplicate-write risk
after a lost acknowledgment. The new capture action uses a single-attempt send
path; legacy callers retain their existing behavior. A lost-reply regression test
checks that only one request reaches the socket.

The context failure recorded in ADR 0007 was traced to installed alpha.8 client
and daemon binaries against the newer section-based server response. An isolated
copy of the local database upgraded with the alpha.9 daemon restored the old
client's context call. This diagnosis requires a matching live binary upgrade and
actual context observation before the integration repair is reported complete.

The actual MCP/fake-socket adapter passed on the built candidate, including
independent scopes/support, all-item preflight, partial receipts and one request
after a lost reply. The matching alpha.9 pair is now installed locally. The real
context tool renders without the previous missing-briefing parse error, but the
server's durable knowledge remains unavailable. A cold restart took longer than
the client deadline and caused retry starts before one daemon settled.

A frozen isolated F1 diagnostic at `b526e64c` observed zero memory writes and zero
persisted treatment memories despite delivered capture guidance. Both sessions
issued task queries. The automatic-capture prerequisite failed, so qualification
stopped without prompt iteration, unseen screening or full milestone advancement.
See [development evidence](../../evals/m1-m2/finding-capture-development-metadata.json).
The remaining decision is how ordinary work reliably produces durable findings;
adding a batch surface and guidance alone has not demonstrated that journey.
