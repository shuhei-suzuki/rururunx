CREATE TABLE prepared_pack_inputs (
    project_id TEXT NOT NULL,
    goal_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    context_version INTEGER NOT NULL CHECK(context_version > 0),
    payload_sha256 TEXT NOT NULL CHECK(length(payload_sha256) = 64),
    revision TEXT NOT NULL,
    input_bytes INTEGER NOT NULL CHECK(input_bytes > 0 AND input_bytes <= 16777216),
    source_versions TEXT NOT NULL CHECK(json_valid(source_versions)),
    PRIMARY KEY(project_id, goal_id, task_id, context_version, payload_sha256),
    FOREIGN KEY(task_id, goal_id, project_id) REFERENCES tasks(id, goal_id, project_id)
);
CREATE TRIGGER prepared_pack_inputs_no_update BEFORE UPDATE ON prepared_pack_inputs BEGIN
    SELECT RAISE(ABORT, 'prepared input authority is immutable');
END;
CREATE TRIGGER prepared_pack_inputs_no_delete BEFORE DELETE ON prepared_pack_inputs BEGIN
    SELECT RAISE(ABORT, 'prepared input authority is immutable');
END;
CREATE TRIGGER prepared_pack_inputs_no_replace BEFORE INSERT ON prepared_pack_inputs
WHEN EXISTS (SELECT 1 FROM prepared_pack_inputs WHERE project_id=NEW.project_id AND goal_id=NEW.goal_id AND task_id=NEW.task_id AND context_version=NEW.context_version AND payload_sha256=NEW.payload_sha256) BEGIN
    SELECT RAISE(ABORT, 'prepared input authority is immutable');
END;
