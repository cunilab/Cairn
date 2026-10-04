# Cairn architecture

This describes the current alpha.9 system. [Product](product.md) owns intended behavior; code and migrations own exact fields and wire contracts.

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

An admitted payload is immutable and has stable operation identity. The daemon claims it and acknowledges it only after server acceptance. A lost acknowledgement causes retry with the same identity; the server returns its recorded receipt, producing one canonical effect over at-least-once transport. Capture and explicit commands retain separate spool rules. A policy change must resolve a prior operation's receipt state before replacing or suppressing that identity.

Retrieval considers current session, branch, then project applicability, subject to account and project authorization. Evidence, authority, verification, conflict, supersession, and pinning affect selection; recency alone does not decide truth. Decay changes ranking and is explainable. Returned context is bounded and can be absent without implying an outage.

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
| Capture / context deadline | 250 ms / 1,500 ms | Deadline expiry returns an honest fallback. |
| Outage cache | 200 sessions, 64 KiB each, 300 s TTL | Account-bound and finite; restart loses it. |
| Event spool | 50,000 rows or 256 MiB of payloads | Oldest ordinary capture can be dropped and counted; protected boundary rows are retained, then admission is refused. This is not a total disk limit. |
| Delivery claim / HTTP request | 60 s / 20 s | Expired claims are recoverable; server receipts make retries idempotent. |
| Retrieval traces | 90 days, sweeps of 500 | Trace expiry does not delete knowledge or evidence. |
| Logical export / import request | 32 MiB serialized JSON / 33 MiB HTTP body | Oversized transfers are refused. This is not a database-size limit. |
| Local snapshot I/O | 64 KiB chunks | Copy and hash avoid loading the whole SQLite file, but need free disk for backup. |

Accepted knowledge, evidence, backups, and PostgreSQL volume growth have no automatic retention policy in alpha.9. Operators must size storage and test physical backup and restore. [Validation](validation.md) states the evidence needed before any wider operating claim.
