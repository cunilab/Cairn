-- Explicit accountability for project-memory reuse.
--
-- Existing rows are intentionally not backfilled: absence means archival and
-- ineligible, which avoids inventing historical support or an actor.
CREATE TABLE project_memory_attestations (
    memory_id             UUID PRIMARY KEY REFERENCES memories(id) ON DELETE CASCADE,
    actor_user_id         UUID NOT NULL REFERENCES users(id),
    basis                 TEXT NOT NULL CHECK (basis IN ('user_report', 'inspected_source')),
    support_summary       TEXT NOT NULL CHECK (
        octet_length(btrim(support_summary)) > 0
        AND octet_length(support_summary) <= 512
    ),
    source_reference      TEXT CHECK (
        source_reference IS NULL OR (
            octet_length(btrim(source_reference)) > 0
            AND octet_length(source_reference) <= 256
        )
    ),
    source_revision       TEXT CHECK (
        source_revision IS NULL OR (
            octet_length(btrim(source_revision)) > 0
            AND octet_length(source_revision) <= 128
        )
    ),
    dependency_memory_id  UUID REFERENCES memories(id),
    dependency_updated_at TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    invalidated_at        TIMESTAMPTZ,
    invalidation_reason   TEXT CHECK (invalidation_reason IN (
        'superseded', 'conflicted', 'source_changed', 'forgotten'
    )),
    CHECK (basis <> 'inspected_source'
           OR (source_reference IS NOT NULL AND source_revision IS NOT NULL)),
    CHECK ((dependency_memory_id IS NULL) = (dependency_updated_at IS NULL)),
    CHECK (dependency_memory_id IS NULL OR dependency_memory_id <> memory_id),
    CHECK ((invalidated_at IS NULL) = (invalidation_reason IS NULL))
);

CREATE INDEX project_memory_attestations_active
    ON project_memory_attestations (memory_id) WHERE invalidated_at IS NULL;
