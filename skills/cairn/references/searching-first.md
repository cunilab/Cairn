# Searching before investigating

The most expensive thing an agent does is rediscover something the project already knew.

## Before you investigate

Search Cairn memory for the subject, the file, the error text, or the component name. A prior
session may have recorded the answer, the failure, or the reason the obvious approach does not
work here.

Ordinary `cairn_search` uses `purpose: "reuse"` and returns only project records with a
current reuse attestation. Keep that default for working context.

Use `purpose: "inspect"` only when deliberately auditing, verifying, superseding, or
correcting stored records. Inspection preserves the archive and reports why a record is
ineligible. Do not switch to inspection merely because ordinary recall returned nothing, and
do not treat an inspected record as a working claim.

## What a good query looks like

- Prefer the words the project uses over the words you would use.
- Search for the error text verbatim before searching for your interpretation of it.
- Search for the file or module name when you are about to change it.
- Two or three narrow queries beat one broad one.

## When search returns nothing

No matching memory was returned. That does not prove nobody investigated the subject.
Do not fall back to archival inspection unless the task actually calls for auditing or
correcting stored memory. Check the query and current source, then proceed. Record a new durable finding if you
establish one — see [recording-knowledge](recording-knowledge.md).
