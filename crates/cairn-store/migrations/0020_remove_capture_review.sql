-- Preserve bounded local task source proof while removing model-review state.
DROP TABLE local_capture_findings;
ALTER TABLE local_task_records DROP COLUMN coverage_status;
ALTER TABLE local_task_records DROP COLUMN coverage_task_digest;
ALTER TABLE local_task_records DROP COLUMN coverage_findings_digest;
ALTER TABLE local_task_records DROP COLUMN coverage_revision;
