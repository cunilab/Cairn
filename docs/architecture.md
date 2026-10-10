# Cairn architecture

This describes the current source, whose package version remains alpha.9. Candidate changes are not proof of published behavior; see the [M1/M2 evidence](../evals/m1-m2/results.md). [Product](product.md) owns intended behavior; code and migrations own exact fields and wire contracts.

```text
agent hooks and MCP  →  cairn / cairnd  →  cairn-server  →  PostgreSQL
                           SQLite edge        ↑
                                         web control plane
```

## Boundaries and flow

`cairn setup` binds an authorized repository and installs supported agent adapters. Hooks capture safe structured lifecycle activity and MCP offers context, search, remember, session, and handoff tools. They are adapters, not separate sources of truth. The local `cairnd` daemon checks the safe event shape, correlates caller and session identity, admits work to typed durable spools, sends it to the server, and caches only bounded returned context.

SQLite stores binding and integration metadata, distinct capture and command spools, retry receipts, hook correlation, and migration metadata. The outage context cache is account-bound and in memory, so a daemon restart clears it. The edge does not store canonical searchable knowledge or a replica of server governance.

`cairn-server` validates the same privacy and authorization boundaries, records accepted events, manages sessions and handoffs, consolidates supported knowledge, and retrieves authorized context. PostgreSQL owns projects, membership, account and team knowledge, evidence, relations, verification, correction history, retrieval state, and idempotency receipts. The web application lets people provision access, inspect memory and sessions, govern shared knowledge, and see health. The supplied Compose stack places web and API behind one public proxy; database, web, and API services are private to the Compose network.

## Delivery and retrieval

Explicit project capture supports one to eight separately supported findings in
`cairn_remember(action: "capture")`. All items are checked before admission;
each then uses the existing single-record durable write path and receives its
own receipt or error. Batch admission is not transactional, and an uncertain
acknowledgement must not cause a whole-batch replay. Separate records preserve
independent source support; they do not establish semantic coherence or bypass
the recall selector. [ADR 0008](adr/0008-independent-finding-capture.md) records
the boundary and required evidence.

An admitted payload is immutable and has stable operation identity. The daemon claims it and acknowledges it only after server acceptance. A lost acknowledgement causes retry with the same identity; the server returns its recorded receipt, producing one canonical effect over at-least-once transport. Capture and explicit commands retain separate spool rules. A policy change must resolve a prior operation's receipt state before replacing or suppressing that identity.

Retrieval considers current session, branch, then project applicability, subject to account and project authorization. Evidence, authority, verification, conflict, supersession, and pinning affect selection; recency alone does not decide truth. Decay changes ranking and is explainable. Returned context is bounded and can be absent without implying an outage.

Context without a task query provides continuity and authorized memory availability,
while project findings require bounded task keywords. Native hooks send no query.
Explicit `cairn_context(query)` uses lexical matching to gather candidates; ordinary
`cairn_search` working recall requires a query. Query responses carry a policy and
normalized query digest so older components cannot silently drop the selector.
Explicit query context bypasses the outage cache. An older unqueried response has
project bodies withheld and its trace is never reported as transmitted.

The candidate server then selects task-conditioned exact excerpts inside those
records before working search, detail or context delivery. Byte spans and source
and excerpt hashes bind provenance to the delivered text; context traces retain
that provenance in schema 10. Authorization, eligibility and source revision are
checked again after inference. Automatic pins and warnings contain references or
status, without memory bodies. Archive inspection retains the original records
and their eligibility disclosures.

Missing inference, provider failure or invalid extraction refuses matching
working recall without whole-record fallback. Exact quotation validation does
not establish semantic relevance or faithful omission of qualifiers; live M2
claim-quality evidence remains pending. [Inference configuration](inference.md)
documents the provider data boundary. [ADR 0004](adr/0004-extractive-semantic-selection.md)
records the selection decision; the [ADR index](adr/README.md) preserves the
other M1/M2 choices and rejected approaches.

## Degraded behavior and recovery

During a network outage, bounded local capture can continue. Saturation is visible; protected boundary events are not silently discarded. Eligible cached context is finite-age and labelled with age and identity. Fresh search reports server unavailability. An observed authorization denial invalidates matching cache immediately; a disconnected client cannot know about unseen revocation.

Legacy SQLite migration preserves the original and WAL state, makes a verified backup, starts a fresh edge store, and exports a versioned import bundle with a conservation report. Only unambiguous safe pending operations keep their identities. Unsupported, local-only, ambiguous, and removed task records remain offline with explicit disposition; migration never broadens their scope. Server logical import is bounded, resumable, and idempotent. It is a healthy-service transfer path, not PostgreSQL disaster recovery.

## Privacy and identity

The edge and server both enforce structured bounds, path restrictions, and secret screening. Free text, including `content`, topic keys, value keys, and applicability values, must be validated; no such field is inherently unable to carry a path or command. Applicability facts are distinct from a knowledge record's `topic_key` and cannot confer access. Credentials, membership, and role authorize operations; payload fields cannot substitute for the actor's identity. Refusals name a safe policy class without echoing rejected material.

## Current source limits

These alpha.9 defaults are safety bounds, not throughput, capacity, or disaster-recovery guarantees. Configurable values may differ at deployment.

| Boundary | Default or limit | Consequence |
| --- | --- | --- |
| Context | 3,000 tokens, minimum 600 | Returned context is bounded. |
| Capture / automatic context / explicit query context deadline | 250 ms / 1,500 ms / 20 s | Deadline expiry returns an honest fallback; queried bodies are never replayed from cache. |
| Inference | 10 s HTTP timeout, 4 concurrent requests, 72 records, 64 KiB response | Capacity saturation and invalid or unavailable output refuse working recall. |
| Outage cache | 200 sessions, 64 KiB each, 300 s TTL | Account-bound and finite; restart loses it. |
| Event spool | 50,000 rows or 256 MiB of payloads | Oldest ordinary capture can be dropped and counted; protected boundary rows are retained, then admission is refused. This is not a total disk limit. |
| Delivery claim / HTTP request | 60 s / 20 s | Expired claims are recoverable; server receipts make retries idempotent. |
| Retrieval traces | 90 days, sweeps of 500 | Trace expiry does not delete knowledge or evidence. |
| Logical export / import request | 32 MiB serialized JSON / 33 MiB HTTP body | Oversized transfers are refused. This is not a database-size limit. |
| Local snapshot I/O | 64 KiB chunks | Copy and hash avoid loading the whole SQLite file, but need free disk for backup. |

Accepted knowledge, evidence, backups, and PostgreSQL volume growth have no automatic retention policy in alpha.9. Operators must size storage and test physical backup and restore. [Validation](validation.md) states the evidence needed before any wider operating claim.

### Native capture finalization

The pinned Codex profile can resume once at Stop to capture supported findings or
state `no_durable_finding`. Native turn metadata is cross-checked with the session
identity; generic clients do not claim this checkpoint. Local bookkeeping is
partitioned by account, server URL digest, session and turn. Command admission and
capture credit share a SQLite transaction. A separate intervention flag, native
recursion guard and remaining hook deadline bound the interaction; errors release
finalization. No prompt or assistant text is stored in checkpoint rows.
[ADR 0009](adr/0009-bounded-native-finalization-checkpoint.md) owns its evidence and
limitations; an admitted command is not semantic or full milestone acceptance.
