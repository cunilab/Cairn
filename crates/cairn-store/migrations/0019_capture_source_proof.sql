-- Source proof stays local with its finding; legacy snapshots cannot prove authored intent.
ALTER TABLE local_capture_findings ADD COLUMN source_task_digest TEXT;
UPDATE local_task_records
SET coverage_status = 'unknown',
    coverage_task_digest = NULL,
    coverage_findings_digest = NULL,
    coverage_revision = NULL;
