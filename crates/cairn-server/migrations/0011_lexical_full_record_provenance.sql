-- Task-scoped retrieval now delivers deterministic lexical full records.
-- Historical extractive rows remain valid; new rows carry exactly one digest for
-- the complete delivered/source record and no misleading excerpt spans.
ALTER TABLE retrieval_trace_items
    DROP CONSTRAINT retrieval_trace_items_selection_provenance_shape;

ALTER TABLE retrieval_trace_items
    ADD CONSTRAINT retrieval_trace_items_selection_provenance_shape CHECK (
        selection_provenance IS NULL OR (
            jsonb_typeof(selection_provenance) = 'object'
            AND (
                (
                    selection_provenance ?& ARRAY['kind', 'source_content_sha256', 'selected_content_sha256', 'spans']
                    AND jsonb_typeof(selection_provenance->'kind') = 'string'
                    AND selection_provenance->>'kind' = 'extractive_excerpt'
                    AND jsonb_typeof(selection_provenance->'source_content_sha256') = 'string'
                    AND jsonb_typeof(selection_provenance->'selected_content_sha256') = 'string'
                    AND jsonb_typeof(selection_provenance->'spans') = 'array'
                )
                OR
                (
                    selection_provenance ?& ARRAY['kind', 'content_sha256']
                    AND (selection_provenance - ARRAY['kind', 'content_sha256']) = '{}'::jsonb
                    AND jsonb_typeof(selection_provenance->'kind') = 'string'
                    AND selection_provenance->>'kind' = 'lexical_full_record'
                    AND jsonb_typeof(selection_provenance->'content_sha256') = 'string'
                )
            )
        )
    );
