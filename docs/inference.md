# Task-scoped recall inference

This configuration belongs to the candidate excerpt-selector implementation.
Published alpha.9 does not provide it. Live semantic quality and M2 exit evidence
remain pending; [ADR 0004](adr/0004-extractive-semantic-selection.md) explains the
selection boundary and [candidate results](../evals/m1-m2/results.md) preserve
the measured failures.

## Configure the server

Provide these three settings together through the server's protected environment:

| Setting | Value |
| --- | --- |
| `CAIRN_INFERENCE_BASE_URL` | OpenAI-compatible API base, including `/v1` if the provider requires it; for example `https://provider.example/v1` |
| `CAIRN_INFERENCE_MODEL` | Exact model ID supported by that endpoint |
| `CAIRN_INFERENCE_API_KEY` | Private server credential |

The server appends `/chat/completions` to the base and requests a JSON object
through chat completions. The provider must support that response format.
HTTPS is required; loopback HTTP is permitted for local tests. URLs cannot contain
credentials, queries or fragments. Redirects are refused.

The supplied [Compose environment example](../deploy/.env.example) and
[server configuration](../deploy/docker-compose.yml) pass these settings only
to the server. In Dokploy, set them in the protected deployment environment and
pass them to the `server` service; deploy the matching candidate server image.
Keep the key out of Git, browser configuration, agent settings and shared logs.

Explicit working recall sends the task query and full authorized eligible
candidate record contents to the provider. That includes content the final
excerpts omit. Choose a provider allowed to process the project's information
under its retention and processing rules. Unqueried continuity and archive
inspection do not invoke inference.

## Failure and compatibility

All settings absent or blank leave inference unconfigured. Startup, unqueried
continuity and archive inspection remain available. A working recall request
with matching candidates reports `selector_unavailable`; no original-record
fallback is returned. A valid selection containing no adequate excerpts returns
an empty result. Partial or malformed configuration fails startup.

Working recall requires the matching server, daemon and CLI policy
`task_excerpts_v1`, plus server schema 10 for excerpt trace provenance. Older
components cannot silently return whole records. Query context uses a 20-second
client deadline; automatic context retains its configured short deadline.
The provider request is bounded separately. Provider failure and malformed or
nonextractive output produce safe actionable errors; inspect original records
only when archival review is intended.

## Validate before milestone claims

First prove extraction bounds, provenance, membership checks and failure behavior
with mechanical tests. Then run the configured provider against mixed records,
negation, attribution, interleaved qualifiers and no-match cases, using the
independent calibrated judge. Freeze that source before rerunning the unchanged
paired workload and installed-artifact deployment journey. A successful API
connection or an exact quotation alone does not establish M2 claim quality.
Record the selector's endpoint identity, model ID and policy alongside the actor,
judge and source/artifact identities. Keep credentials and raw provider inputs
private; disclose any provider revision or input-readback evidence that is unavailable.
