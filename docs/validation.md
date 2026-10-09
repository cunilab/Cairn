# Validation and release evidence

This document owns what must be demonstrated before Cairn claims a behavior works. [Product](product.md) owns the required outcome; [architecture](architecture.md) owns current technical boundaries. Passing a test count or citing an old requirement is not the same as proving a current user journey.

## Test tiers

| Tier | Boundary | Typical location |
| --- | --- | --- |
| 1 — pure | In-memory logic with no filesystem, socket, database, clock, or subprocess | `#[cfg(test)]` beside code |
| 2 — component | Real temporary SQLite, repository, or filesystem in-process | `#[cfg(test)]` beside code |
| 3 — journey | Real binaries, browser, agent hook/MCP, and user-visible outcome | `tests/tests/`, `web/e2e/`, release scripts |
| 4 — hostile | Crash, concurrency, corruption, unavailable network, saturation, or platform transport | focused integration and release tests |

Choose the smallest tier that exercises the boundary, then add a real journey when the claim crosses binaries or services. A test should say what a user can observe. Historical `FR-*` citations in older test names/comments retain their original meaning in [the immutable alpha.7 tree](https://github.com/cunilab/Cairn/tree/v0.1.0-alpha.7); they do not replace the current [PRD-01–09](product.md#product-requirements) contract.

## Local checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace --all-targets
cd web && npm ci && npm run typecheck && npm run api-contract:check && npm run build
```

PostgreSQL tests use `CAIRN_TEST_DATABASE_URL` pointing to a disposable database. A required server lane sets `CAIRN_REQUIRE_DATABASE_TESTS=1`; missing configuration must fail rather than silently count as a pass. Optional local runs can report `NOT RUN`. Record the database and platform versions, build/source identity, exact command and result. Web and installed-artifact checks need their own gates; source tests alone do not establish them.

For a release-server journey, select the built artifact explicitly so worker tests do not
pick up an older binary from another build directory:

```bash
cargo build --workspace --release --locked
CAIRN_SERVER_BIN="$PWD/target/release/cairn-server" \
CAIRN_REQUIRE_DATABASE_TESTS=1 cargo test --workspace --all-targets
```

## Required product evidence

| Claim | Positive and adverse evidence |
| --- | --- |
| First use and return | Fresh database and home: browser project/member/token → matching repository → `cairn setup` → accepted event/knowledge → later relevant recall. Also invalid remote, nonmember, wrong origin, reload, and missing credential. |
| Agent integration | Real setup and rerun for supported agents, missing-resource repair, user-edit conflict preservation, trust and version gating, actual installed hook/MCP behavior, and explicit generic-MCP limits. |
| Privacy and identity | Safe structured capture and explicit memory, refused unsafe payloads without echoed content, exact caller/project/account attribution, concurrent callers, revoked token, personal isolation, and team authorization. |
| Delivery and retrieval | Offline queue, retry after lost acknowledgement, one canonical effect, saturation and boundary events, cache age/expiry/denial, empty and conflicting results, bounded rendering, and actual transmitted context. |
| Knowledge usefulness | Frozen paired later tasks with/without Cairn across repositories: useful fact, procedure, failed approach, conflict, benign absence, and stale or unsupported information. Report useful, irrelevant, and harmful recall, not only record counts. |
| Recovery | Crash/restart, legacy SQLite with WAL, import idempotence and rejection, source disposition accounting, concurrent export snapshot, physical PostgreSQL backup/restore, and actionable health. Logical import/export is not disaster recovery. |
| Platform and capacity | Installed native archives on advertised platforms, published image digests, clean Compose deployment, browser smoke, measured hook deadlines and resource envelope under stated workload. Unrun platforms and unmeasured capacity remain limitations. |

For each release candidate, record `PASS`, `FAIL`, or `NOT RUN` with a reason; source SHA, archive checksums, image digests, OS/architecture, PostgreSQL version, commands, results, and artifact links. Do not include credentials or raw agent material. Required failures and missing evidence block that claim. Candidate source, release assets, OCI metadata, attestation subjects, published tags, and deployment defaults must agree on the repository and version. Publishing immutable artifacts is a separate action from preparing evidence.

The [alpha.9 evidence report](../.github/release-evidence/v0.1.0-alpha.9.md) records candidate tests and limits; the [published release](https://github.com/cunilab/Cairn/releases/tag/v0.1.0-alpha.9) and [publish workflow](https://github.com/cunilab/Cairn/actions/workflows/publish-release.yml) establish subsequent publication. Its source limits in [architecture](architecture.md#current-source-limits) are not measured support guarantees. The alpha.9 report explicitly lacks native Intel macOS execution evidence; future support claims need fresh installed-artifact proof.

The [M1/M2 candidate report](../evals/m1-m2/results.md) is separate evidence for the new
source candidate. It records local journey passes and the frozen 30-pair run, with full
M1 deployment/platform evidence and M2 semantic gates still open. Historical release
evidence does not close those candidate gates.

## Bounded retrieval development experiments

[ADR 0006](adr/0006-local-embedding-experiment.md) authorizes a local embedding
screen before production integration. Freeze source units, independent retrieval
calibration, development queries, selection/cutoff rules, runtime/model identities
and evidence hashes before execution. Compare keyword and embedding ranking on
identical automatic units. Calibrate similarity cutoffs only on the separate
calibration examples; do not treat scores as probabilities or tune them using
development judgments.

The independent calibrated judge inventories actual delivered claims, including
irrelevant claims in the same unit, and checks source attribution, negation,
conditions and eligible useful recall. Record abstentions, misses, corruption,
latency and denominators. Require the existing 90% claim quality, 80% eligible
useful-recall targets and no qualifier/authority corruption to advance. Preserve
failures and stop a failed approach before integration. Empty denominators do
not pass. This synthetic experiment cannot substitute for the frozen paired
workload, privacy/adverse evidence or fresh installed deployment journey.

## Proposed future gates

These are planning thresholds inherited from the previous roadmap, not results or current support guarantees. Freeze the workload and scoring rubric before using them as release gates; record denominators and revise targets openly if product evidence warrants it.

- **Useful recall:** 30 paired tasks across at least three repositories and Claude Code/Codex, five pairs each for fixes, decisions/procedures, repeated failures, stale/conflicting advice, privacy/refusal, and benign no-memory work. Proposed targets: useful recall of at least 80% of eligible labelled claims, at least 90% of delivered claims relevant and supported, no privacy leak or high-impact harmful advice, and at least 20% median fewer repeated investigation steps where the control performed investigation. Task completion must be no worse than control. Report tokens and latency, plus misses and adverse cases.
- **Small-team envelope:** reference host of 4 vCPU, 8 GiB RAM, SSD, and LAN round-trip time at most 50 ms; 10 accounts, 10 active sessions, 20 projects, 100,000 canonical memories, and 1,000,000 accepted events. Proposed ordinary authenticated API p95 at most 500 ms and usable main page p95 at most 2 s under stated workload. Sustained 10 events/s for 30 minutes should drain within 10 minutes after arrivals stop, with no unexplained loss or duplicate effect.
- **Transfer and recovery:** proposed logical bundle up to 256 MiB in either direction within 10 minutes and at most 1 GiB additional process memory on the reference host. A separately rehearsed physical backup/restore aims for RPO at most 24 h and RTO at most 60 min. Current alpha.9 limits remain lower and authoritative until implementation and evidence change.
- **Release candidate:** exercise the advertised OS/architecture, agent, browser, archive, and image combinations as installed artifacts. A proposed 14-day pilot across three deployments, including solo and small-team use, precedes stable 0.1. An unresolved critical/high security or data-integrity defect blocks the relevant claim.
