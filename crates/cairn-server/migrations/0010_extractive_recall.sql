-- Bind a delivered project-memory excerpt to the exact source bytes selected.
-- Existing trace items stay valid historical rows with no excerpt provenance.
ALTER TABLE retrieval_trace_items
    ADD COLUMN selection_provenance JSONB;

ALTER TABLE retrieval_trace_items
    ADD CONSTRAINT retrieval_trace_items_selection_provenance_shape CHECK (
        selection_provenance IS NULL OR (
            jsonb_typeof(selection_provenance) = 'object'
            AND selection_provenance ?& ARRAY['kind', 'source_content_sha256', 'selected_content_sha256', 'spans']
            AND jsonb_typeof(selection_provenance->'kind') = 'string'
            AND selection_provenance->>'kind' = 'extractive_excerpt'
            AND jsonb_typeof(selection_provenance->'source_content_sha256') = 'string'
            AND jsonb_typeof(selection_provenance->'selected_content_sha256') = 'string'
            AND jsonb_typeof(selection_provenance->'spans') = 'array'
        )
    );
