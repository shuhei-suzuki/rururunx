CREATE TABLE runtime_epoch (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    instance_id TEXT NOT NULL,
    epoch INTEGER NOT NULL CHECK(epoch>=0)
);
CREATE TABLE execution_units (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL, task_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('executor','reviewer','verifier','legacy')),
    generation INTEGER NOT NULL CHECK(generation>=0),
    owner_epoch INTEGER NOT NULL CHECK(owner_epoch>=0),
    version INTEGER NOT NULL CHECK(version>0),
    native_effects_open INTEGER NOT NULL CHECK(native_effects_open IN (0,1)),
    result_finalization_open INTEGER NOT NULL CHECK(result_finalization_open IN (0,1)),
    worktree TEXT UNIQUE NOT NULL,
    branch TEXT,
    body TEXT NOT NULL CHECK(json_valid(body)),
    UNIQUE(id,project_id,goal_id,task_id),
    UNIQUE(project_id,branch),
    FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id)
);
CREATE UNIQUE INDEX one_active_executor ON execution_units(task_id)
    WHERE kind='executor' AND (native_effects_open=1 OR result_finalization_open=1);
CREATE TABLE execution_context (
    unit_id TEXT PRIMARY KEY NOT NULL REFERENCES execution_units(id),
    project_version INTEGER NOT NULL CHECK(project_version>0),
    goal_version INTEGER NOT NULL CHECK(goal_version>0),
    governing_digest TEXT NOT NULL CHECK(length(governing_digest)=64)
);
CREATE TRIGGER execution_context_no_update BEFORE UPDATE ON execution_context
BEGIN SELECT RAISE(ABORT,'admission context is immutable'); END;
CREATE TRIGGER execution_context_no_delete BEFORE DELETE ON execution_context
BEGIN SELECT RAISE(ABORT,'admission context is immutable'); END;
CREATE TABLE task_execution (
    task_id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, goal_id TEXT NOT NULL,
    generation INTEGER NOT NULL CHECK(generation>=0), active_unit TEXT,
    FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id),
    FOREIGN KEY(active_unit,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id)
);
CREATE TABLE session_units (
    session_id TEXT PRIMARY KEY NOT NULL, unit_id TEXT UNIQUE NOT NULL,
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL, task_id TEXT NOT NULL,
    dispatch_state TEXT NOT NULL CHECK(dispatch_state IN ('pending','acknowledged','unknown','terminal')),
    FOREIGN KEY(session_id,project_id) REFERENCES records(id,project_id),
    FOREIGN KEY(unit_id,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id)
);
CREATE TABLE result_artifacts (
    id TEXT PRIMARY KEY NOT NULL, unit_id TEXT NOT NULL,
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL, task_id TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('staging','ready','published','invalid')),
    version INTEGER NOT NULL CHECK(version>0),
    body TEXT NOT NULL CHECK(json_valid(body)),
    UNIQUE(id,project_id,goal_id,task_id),
    FOREIGN KEY(unit_id,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id)
);
CREATE TABLE artifact_dependencies (
    artifact_id TEXT NOT NULL REFERENCES result_artifacts(id),
    name TEXT NOT NULL, digest TEXT NOT NULL, PRIMARY KEY(artifact_id,name)
);
CREATE TRIGGER artifact_published_identity BEFORE UPDATE ON result_artifacts
WHEN OLD.state IN ('published','invalid') AND NOT (
    OLD.state='published' AND NEW.state='invalid' AND NEW.version=OLD.version+1
    AND OLD.id=NEW.id AND OLD.unit_id=NEW.unit_id AND OLD.project_id=NEW.project_id
    AND OLD.goal_id=NEW.goal_id AND OLD.task_id=NEW.task_id
    AND json_remove(OLD.body,'$.state','$.version')=json_remove(NEW.body,'$.state','$.version')
) BEGIN SELECT RAISE(ABORT,'published artifact identity is immutable'); END;
CREATE TRIGGER artifact_published_no_delete BEFORE DELETE ON result_artifacts
WHEN OLD.state IN ('published','invalid')
BEGIN SELECT RAISE(ABORT,'published artifact is retained'); END;
CREATE TRIGGER artifact_dependency_no_update BEFORE UPDATE ON artifact_dependencies
BEGIN SELECT RAISE(ABORT,'artifact dependency is immutable'); END;
CREATE TRIGGER artifact_dependency_no_delete BEFORE DELETE ON artifact_dependencies
BEGIN SELECT RAISE(ABORT,'artifact dependency is immutable'); END;
CREATE TRIGGER artifact_dependency_closed BEFORE INSERT ON artifact_dependencies
WHEN (SELECT state FROM result_artifacts WHERE id=NEW.artifact_id) NOT IN ('staging','ready')
BEGIN SELECT RAISE(ABORT,'artifact dependencies closed'); END;
CREATE TABLE resource_leases (
    id TEXT PRIMARY KEY NOT NULL, unit_id TEXT NOT NULL,
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL, task_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('worktree','temp','output','ports','docker','tool_socket')),
    namespace TEXT NOT NULL, value TEXT NOT NULL,
    port_start INTEGER, port_end INTEGER,
    state TEXT NOT NULL CHECK(state IN ('reserved','creating','active','quarantined','released')),
    version INTEGER NOT NULL CHECK(version>0), body TEXT NOT NULL CHECK(json_valid(body)),
    CHECK((kind='ports' AND port_start IS NOT NULL AND port_end IS NOT NULL AND port_start BETWEEN 1024 AND 65535 AND port_end BETWEEN port_start AND 65535)
        OR (kind<>'ports' AND port_start IS NULL AND port_end IS NULL)),
    FOREIGN KEY(unit_id,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id)
);
CREATE UNIQUE INDEX resource_live_identity ON resource_leases(kind,namespace,value) WHERE state<>'released';
CREATE TRIGGER resource_port_overlap_insert BEFORE INSERT ON resource_leases
WHEN NEW.kind='ports' AND NEW.state<>'released' AND EXISTS(
    SELECT 1 FROM resource_leases WHERE kind='ports' AND namespace=NEW.namespace
    AND state<>'released' AND port_start<=NEW.port_end AND port_end>=NEW.port_start
) BEGIN SELECT RAISE(ABORT,'port range already leased'); END;
CREATE TRIGGER resource_port_overlap_update BEFORE UPDATE ON resource_leases
WHEN NEW.kind='ports' AND NEW.state<>'released' AND EXISTS(
    SELECT 1 FROM resource_leases WHERE id<>NEW.id AND kind='ports' AND namespace=NEW.namespace
    AND state<>'released' AND port_start<=NEW.port_end AND port_end>=NEW.port_start
) BEGIN SELECT RAISE(ABORT,'port range already leased'); END;
CREATE TABLE managed_effects (
    id TEXT PRIMARY KEY NOT NULL, unit_id TEXT NOT NULL,
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL, task_id TEXT NOT NULL,
    idempotency_key TEXT UNIQUE NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('pending','confirmed','unknown','resolved')),
    version INTEGER NOT NULL CHECK(version>0), body TEXT NOT NULL CHECK(json_valid(body)),
    FOREIGN KEY(unit_id,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id)
);
CREATE TABLE cleanup_jobs (
    unit_id TEXT PRIMARY KEY NOT NULL REFERENCES execution_units(id),
    next_due INTEGER NOT NULL, attempts INTEGER NOT NULL CHECK(attempts>=0),
    version INTEGER NOT NULL CHECK(version>0)
);
CREATE TABLE cleanup_observations (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT, unit_id TEXT NOT NULL REFERENCES execution_units(id),
    at INTEGER NOT NULL, body TEXT NOT NULL CHECK(json_valid(body))
);
CREATE TABLE quota_pools (
    provider TEXT NOT NULL, account_key TEXT NOT NULL,
    next_probe_at INTEGER NOT NULL DEFAULT 0, probe_unit TEXT REFERENCES execution_units(id),
    backoff INTEGER NOT NULL DEFAULT 60000 CHECK(backoff BETWEEN 60000 AND 1800000),
    last_role TEXT NOT NULL DEFAULT 'reviewer',
    PRIMARY KEY(provider,account_key)
);
CREATE TABLE quota_windows (
    provider TEXT NOT NULL, account_key TEXT NOT NULL, bucket TEXT NOT NULL,
    observed_at INTEGER NOT NULL, body TEXT NOT NULL CHECK(json_valid(body)),
    PRIMARY KEY(provider,account_key,bucket),
    FOREIGN KEY(provider,account_key) REFERENCES quota_pools(provider,account_key)
);
CREATE TABLE quota_leases (
    unit_id TEXT PRIMARY KEY NOT NULL REFERENCES execution_units(id),
    provider TEXT NOT NULL, account_key TEXT NOT NULL, role TEXT NOT NULL,
    epoch INTEGER NOT NULL CHECK(epoch>0), active INTEGER NOT NULL CHECK(active IN (0,1)),
    FOREIGN KEY(provider,account_key) REFERENCES quota_pools(provider,account_key)
);
CREATE TABLE quota_waiters (
    unit_id TEXT PRIMARY KEY NOT NULL REFERENCES execution_units(id),
    provider TEXT NOT NULL,account_key TEXT NOT NULL,
    reason TEXT NOT NULL, next_due INTEGER NOT NULL, fairness_sequence INTEGER NOT NULL,
    resume_state TEXT NOT NULL,
    FOREIGN KEY(provider,account_key) REFERENCES quota_pools(provider,account_key)
);
CREATE TRIGGER cleanup_observation_no_update BEFORE UPDATE ON cleanup_observations
BEGIN SELECT RAISE(ABORT,'cleanup observations are append-only'); END;
CREATE TRIGGER cleanup_observation_no_delete BEFORE DELETE ON cleanup_observations
BEGIN SELECT RAISE(ABORT,'cleanup observations are append-only'); END;
