-- Binding10: exact private mutation permission is necessary, never a native proof.
CREATE TABLE workflow_native_contracts (
    workflow_id TEXT PRIMARY KEY NOT NULL REFERENCES records(id),
    project_id TEXT NOT NULL CHECK(length(project_id) BETWEEN 1 AND 4096),
    goal_id TEXT NOT NULL CHECK(length(goal_id) BETWEEN 1 AND 4096),
    task_id TEXT NOT NULL CHECK(length(task_id) BETWEEN 1 AND 4096),
    owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>=0),
    origin TEXT NOT NULL CHECK(length(origin) BETWEEN 1 AND 4096),
    profile_digest TEXT CHECK(profile_digest IS NULL OR (length(profile_digest)=64 AND profile_digest NOT GLOB '*[^0-9a-f]*')),
    contract_state TEXT NOT NULL CHECK(contract_state IN ('legacy_held','composed')),
    version INTEGER NOT NULL CHECK(version=1),
    body TEXT NOT NULL CHECK(typeof(body)='text' AND json_valid(body) AND length(CAST(body AS BLOB))<=4096),
    FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id), CHECK((contract_state='legacy_held' AND owner_epoch=0 AND profile_digest IS NULL) OR (contract_state='composed' AND owner_epoch>0 AND profile_digest IS NOT NULL))
);
CREATE TABLE managed_phase_operations (
    operation_id TEXT PRIMARY KEY NOT NULL,
    workflow_id TEXT NOT NULL REFERENCES workflow_native_contracts(workflow_id),
    project_id TEXT NOT NULL CHECK(length(project_id) BETWEEN 1 AND 4096),
    goal_id TEXT NOT NULL CHECK(length(goal_id) BETWEEN 1 AND 4096),
    task_id TEXT NOT NULL CHECK(length(task_id) BETWEEN 1 AND 4096),
    workflow_generation INTEGER NOT NULL CHECK(typeof(workflow_generation)='integer' AND workflow_generation>0),
    attempt_index INTEGER NOT NULL CHECK(typeof(attempt_index)='integer' AND attempt_index>=0),
    owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>0),
    unit_id TEXT NOT NULL CHECK(length(unit_id) BETWEEN 1 AND 4096),
    execution_generation INTEGER NOT NULL CHECK(typeof(execution_generation)='integer' AND execution_generation>0),
    allocated_session_id TEXT NOT NULL UNIQUE,
    owner_id TEXT NOT NULL UNIQUE REFERENCES managed_phase_owners(owner_id) DEFERRABLE INITIALLY DEFERRED,
    pair_id TEXT NOT NULL UNIQUE REFERENCES managed_phase_inputs(pair_id) DEFERRABLE INITIALLY DEFERRED,
    marker_project_version INTEGER NOT NULL CHECK(typeof(marker_project_version)='integer' AND marker_project_version>0),
    marker_goal_version INTEGER NOT NULL CHECK(typeof(marker_goal_version)='integer' AND marker_goal_version>0),
    marker_task_version INTEGER NOT NULL CHECK(typeof(marker_task_version)='integer' AND marker_task_version>0),
    marker_workflow_version INTEGER NOT NULL CHECK(typeof(marker_workflow_version)='integer' AND marker_workflow_version>0),
    marker_project_sha256 TEXT NOT NULL CHECK(length(marker_project_sha256)=64 AND marker_project_sha256 NOT GLOB '*[^0-9a-f]*'),
    marker_goal_sha256 TEXT NOT NULL CHECK(length(marker_goal_sha256)=64 AND marker_goal_sha256 NOT GLOB '*[^0-9a-f]*'),
    marker_task_sha256 TEXT NOT NULL CHECK(length(marker_task_sha256)=64 AND marker_task_sha256 NOT GLOB '*[^0-9a-f]*'),
    marker_workflow_sha256 TEXT NOT NULL CHECK(length(marker_workflow_sha256)=64 AND marker_workflow_sha256 NOT GLOB '*[^0-9a-f]*'),
    marker_digest TEXT NOT NULL CHECK(length(marker_digest)=64 AND marker_digest NOT GLOB '*[^0-9a-f]*'),
    origin TEXT NOT NULL CHECK(length(origin) BETWEEN 1 AND 4096),
    phase TEXT NOT NULL CHECK(length(phase) BETWEEN 1 AND 4096),
    actor TEXT NOT NULL CHECK(length(actor) BETWEEN 1 AND 4096),
    provider TEXT NOT NULL CHECK(length(provider) BETWEEN 1 AND 4096),
    alias TEXT NOT NULL CHECK(length(alias) BETWEEN 1 AND 4096),
    role TEXT NOT NULL CHECK(length(role) BETWEEN 1 AND 4096),
    context_version INTEGER NOT NULL CHECK(typeof(context_version)='integer' AND context_version>0),
    phase_open INTEGER NOT NULL CHECK(typeof(phase_open)='integer' AND phase_open IN (0,1)),
    version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
    body TEXT NOT NULL CHECK(typeof(body)='text' AND json_valid(body) AND length(CAST(body AS BLOB))<=4194304),
    FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id), FOREIGN KEY(unit_id,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id), UNIQUE(workflow_id,workflow_generation,attempt_index)
);
CREATE TABLE managed_marker_bodies (
    operation_id TEXT PRIMARY KEY NOT NULL REFERENCES managed_phase_operations(operation_id),
    workflow_id TEXT NOT NULL REFERENCES records(id),
    workflow_version INTEGER NOT NULL CHECK(typeof(workflow_version)='integer' AND workflow_version>0),
    workflow_sha256 TEXT NOT NULL CHECK(length(workflow_sha256)=64 AND workflow_sha256 NOT GLOB '*[^0-9a-f]*'),
    body TEXT NOT NULL CHECK(typeof(body)='text' AND json_valid(body) AND length(CAST(body AS BLOB))<=8388608)
);
CREATE TABLE managed_phase_owners (
    owner_id TEXT PRIMARY KEY NOT NULL,
    operation_id TEXT NOT NULL UNIQUE REFERENCES managed_phase_operations(operation_id),
    project_id TEXT NOT NULL CHECK(length(project_id) BETWEEN 1 AND 4096),
    goal_id TEXT NOT NULL CHECK(length(goal_id) BETWEEN 1 AND 4096),
    task_id TEXT NOT NULL CHECK(length(task_id) BETWEEN 1 AND 4096),
    unit_id TEXT NOT NULL CHECK(length(unit_id) BETWEEN 1 AND 4096),
    owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>0),
    execution_generation INTEGER NOT NULL CHECK(typeof(execution_generation)='integer' AND execution_generation>0),
    allocated_session_id TEXT NOT NULL UNIQUE,
    provider TEXT NOT NULL CHECK(length(provider) BETWEEN 1 AND 4096),
    alias TEXT NOT NULL CHECK(length(alias) BETWEEN 1 AND 4096),
    role TEXT NOT NULL CHECK(length(role) BETWEEN 1 AND 4096),
    worktree TEXT NOT NULL CHECK(length(worktree) BETWEEN 1 AND 4096),
    origin TEXT NOT NULL CHECK(length(origin) BETWEEN 1 AND 4096),
    native_invocation_id TEXT UNIQUE REFERENCES native_invocations(id),
    validated INTEGER NOT NULL CHECK(typeof(validated)='integer' AND validated IN (0,1)),
    version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
    body TEXT NOT NULL CHECK(typeof(body)='text' AND json_valid(body) AND length(CAST(body AS BLOB))<=2097152),
    FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id), FOREIGN KEY(unit_id,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id)
);
CREATE TABLE managed_phase_inputs (
    pair_id TEXT PRIMARY KEY NOT NULL,
    owner_id TEXT NOT NULL UNIQUE REFERENCES managed_phase_owners(owner_id),
    operation_id TEXT NOT NULL UNIQUE REFERENCES managed_phase_operations(operation_id),
    project_id TEXT NOT NULL CHECK(length(project_id) BETWEEN 1 AND 4096),
    goal_id TEXT NOT NULL CHECK(length(goal_id) BETWEEN 1 AND 4096),
    task_id TEXT NOT NULL CHECK(length(task_id) BETWEEN 1 AND 4096),
    context_version INTEGER NOT NULL CHECK(typeof(context_version)='integer' AND context_version>0),
    context_digest TEXT NOT NULL CHECK(length(context_digest)=64 AND context_digest NOT GLOB '*[^0-9a-f]*'),
    revision TEXT NOT NULL CHECK(length(revision) BETWEEN 1 AND 4096),
    payload_sha256 TEXT NOT NULL CHECK(length(payload_sha256)=64 AND payload_sha256 NOT GLOB '*[^0-9a-f]*'),
    payload_bytes INTEGER NOT NULL CHECK(typeof(payload_bytes)='integer' AND payload_bytes BETWEEN 1 AND 1048576),
    profile_digest TEXT NOT NULL CHECK(length(profile_digest)=64 AND profile_digest NOT GLOB '*[^0-9a-f]*'),
    version INTEGER NOT NULL CHECK(version=1),
    body TEXT NOT NULL CHECK(typeof(body)='text' AND json_valid(body) AND length(CAST(body AS BLOB))<=2097152),
    FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id)
);
CREATE TABLE managed_phase_admissions (
    pair_id TEXT PRIMARY KEY NOT NULL REFERENCES managed_phase_inputs(pair_id),
    operation_id TEXT NOT NULL UNIQUE REFERENCES managed_phase_operations(operation_id),
    owner_id TEXT NOT NULL REFERENCES managed_phase_owners(owner_id),
    native_invocation_id TEXT NOT NULL UNIQUE REFERENCES native_invocations(id),
    input_effect_id TEXT NOT NULL UNIQUE REFERENCES managed_effects(id),
    frame_sha256 TEXT NOT NULL CHECK(length(frame_sha256)=64 AND frame_sha256 NOT GLOB '*[^0-9a-f]*'),
    native_thread TEXT CHECK(native_thread IS NULL OR length(native_thread) BETWEEN 1 AND 4096),
    native_turn TEXT CHECK(native_turn IS NULL OR length(native_turn) BETWEEN 1 AND 4096),
    confirmed INTEGER NOT NULL CHECK(typeof(confirmed)='integer' AND confirmed IN (0,1)),
    uncertain INTEGER NOT NULL CHECK(typeof(uncertain)='integer' AND uncertain IN (0,1)),
    settled INTEGER NOT NULL CHECK(typeof(settled)='integer' AND settled IN (0,1)),
    version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
    body TEXT NOT NULL CHECK(typeof(body)='text' AND json_valid(body) AND length(CAST(body AS BLOB))<=8192)
);
CREATE TABLE managed_phase_readiness (
    operation_id TEXT PRIMARY KEY NOT NULL REFERENCES managed_phase_operations(operation_id),
    origin TEXT NOT NULL CHECK(length(origin) BETWEEN 1 AND 4096),
    owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>0),
    state TEXT NOT NULL CHECK(state IN ('allocated','preparing','parked','registered','deferred','held','closed')),
    start_ended INTEGER NOT NULL CHECK(typeof(start_ended)='integer' AND start_ended IN (0,1)),
    known_terminal INTEGER NOT NULL CHECK(typeof(known_terminal)='integer' AND known_terminal IN (0,1)),
    parking_version INTEGER CHECK(parking_version IS NULL OR (typeof(parking_version)='integer' AND parking_version>0)),
    version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
    body TEXT NOT NULL CHECK(typeof(body)='text' AND json_valid(body) AND length(CAST(body AS BLOB))<=4096)
);
CREATE TABLE scoped_session_identities (
    session_id TEXT PRIMARY KEY NOT NULL REFERENCES records(id),
    project_id TEXT NOT NULL CHECK(length(project_id) BETWEEN 1 AND 4096),
    goal_id TEXT,
    task_id TEXT,
    record_version INTEGER NOT NULL CHECK(typeof(record_version)='integer' AND record_version>0),
    provider TEXT CHECK(provider IS NULL OR length(provider) BETWEEN 1 AND 512),
    native_ref TEXT CHECK(native_ref IS NULL OR length(native_ref) BETWEEN 1 AND 512),
    malformed INTEGER NOT NULL CHECK(typeof(malformed)='integer' AND malformed IN (0,1)),
    body TEXT NOT NULL CHECK(typeof(body)='text' AND json_valid(body) AND length(CAST(body AS BLOB))<=4096),
    CHECK(task_id IS NULL OR goal_id IS NOT NULL), FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id), FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id)
);
CREATE INDEX binding_session_negative ON scoped_session_identities(project_id,goal_id,task_id,provider,native_ref,session_id);
CREATE INDEX binding_session_malformed ON scoped_session_identities(project_id,goal_id,task_id,malformed);
CREATE INDEX binding_contract_task ON workflow_native_contracts(task_id);
CREATE INDEX binding_operation_open ON managed_phase_operations(project_id,task_id,phase_open);
CREATE TRIGGER binding_workflow_native_contracts_INSERT BEFORE INSERT ON workflow_native_contracts WHEN NOT rrx_binding_permit('workflow_native_contracts','INSERT',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NEW.workflow_id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.owner_epoch,NEW.origin,NEW.profile_digest,NEW.contract_state,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_workflow_native_contracts_UPDATE BEFORE UPDATE ON workflow_native_contracts WHEN NOT rrx_binding_permit('workflow_native_contracts','UPDATE',OLD.workflow_id,OLD.project_id,OLD.goal_id,OLD.task_id,OLD.owner_epoch,OLD.origin,OLD.profile_digest,OLD.contract_state,OLD.version,OLD.body,NEW.workflow_id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.owner_epoch,NEW.origin,NEW.profile_digest,NEW.contract_state,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_workflow_native_contracts_DELETE BEFORE DELETE ON workflow_native_contracts WHEN NOT rrx_binding_permit('workflow_native_contracts','DELETE',OLD.workflow_id,OLD.project_id,OLD.goal_id,OLD.task_id,OLD.owner_epoch,OLD.origin,OLD.profile_digest,OLD.contract_state,OLD.version,OLD.body,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_workflow_native_contracts_no_replace BEFORE INSERT ON workflow_native_contracts WHEN EXISTS(SELECT 1 FROM workflow_native_contracts WHERE workflow_id=NEW.workflow_id) BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_workflow_native_contracts_no_delete BEFORE DELETE ON workflow_native_contracts BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_workflow_native_contracts_immutable BEFORE UPDATE ON workflow_native_contracts BEGIN SELECT RAISE(ABORT,'managed core is immutable'); END;
CREATE TRIGGER binding_managed_phase_operations_INSERT BEFORE INSERT ON managed_phase_operations WHEN NOT rrx_binding_permit('managed_phase_operations','INSERT',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NEW.operation_id,NEW.workflow_id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.workflow_generation,NEW.attempt_index,NEW.owner_epoch,NEW.unit_id,NEW.execution_generation,NEW.allocated_session_id,NEW.owner_id,NEW.pair_id,NEW.marker_project_version,NEW.marker_goal_version,NEW.marker_task_version,NEW.marker_workflow_version,NEW.marker_project_sha256,NEW.marker_goal_sha256,NEW.marker_task_sha256,NEW.marker_workflow_sha256,NEW.marker_digest,NEW.origin,NEW.phase,NEW.actor,NEW.provider,NEW.alias,NEW.role,NEW.context_version,NEW.phase_open,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_operations_UPDATE BEFORE UPDATE ON managed_phase_operations WHEN NOT rrx_binding_permit('managed_phase_operations','UPDATE',OLD.operation_id,OLD.workflow_id,OLD.project_id,OLD.goal_id,OLD.task_id,OLD.workflow_generation,OLD.attempt_index,OLD.owner_epoch,OLD.unit_id,OLD.execution_generation,OLD.allocated_session_id,OLD.owner_id,OLD.pair_id,OLD.marker_project_version,OLD.marker_goal_version,OLD.marker_task_version,OLD.marker_workflow_version,OLD.marker_project_sha256,OLD.marker_goal_sha256,OLD.marker_task_sha256,OLD.marker_workflow_sha256,OLD.marker_digest,OLD.origin,OLD.phase,OLD.actor,OLD.provider,OLD.alias,OLD.role,OLD.context_version,OLD.phase_open,OLD.version,OLD.body,NEW.operation_id,NEW.workflow_id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.workflow_generation,NEW.attempt_index,NEW.owner_epoch,NEW.unit_id,NEW.execution_generation,NEW.allocated_session_id,NEW.owner_id,NEW.pair_id,NEW.marker_project_version,NEW.marker_goal_version,NEW.marker_task_version,NEW.marker_workflow_version,NEW.marker_project_sha256,NEW.marker_goal_sha256,NEW.marker_task_sha256,NEW.marker_workflow_sha256,NEW.marker_digest,NEW.origin,NEW.phase,NEW.actor,NEW.provider,NEW.alias,NEW.role,NEW.context_version,NEW.phase_open,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_operations_DELETE BEFORE DELETE ON managed_phase_operations WHEN NOT rrx_binding_permit('managed_phase_operations','DELETE',OLD.operation_id,OLD.workflow_id,OLD.project_id,OLD.goal_id,OLD.task_id,OLD.workflow_generation,OLD.attempt_index,OLD.owner_epoch,OLD.unit_id,OLD.execution_generation,OLD.allocated_session_id,OLD.owner_id,OLD.pair_id,OLD.marker_project_version,OLD.marker_goal_version,OLD.marker_task_version,OLD.marker_workflow_version,OLD.marker_project_sha256,OLD.marker_goal_sha256,OLD.marker_task_sha256,OLD.marker_workflow_sha256,OLD.marker_digest,OLD.origin,OLD.phase,OLD.actor,OLD.provider,OLD.alias,OLD.role,OLD.context_version,OLD.phase_open,OLD.version,OLD.body,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_operations_no_replace BEFORE INSERT ON managed_phase_operations WHEN EXISTS(SELECT 1 FROM managed_phase_operations WHERE operation_id=NEW.operation_id) BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_operations_no_delete BEFORE DELETE ON managed_phase_operations BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_operations_core BEFORE UPDATE ON managed_phase_operations WHEN NEW.operation_id IS NOT OLD.operation_id OR NEW.workflow_id IS NOT OLD.workflow_id OR NEW.project_id IS NOT OLD.project_id OR NEW.goal_id IS NOT OLD.goal_id OR NEW.task_id IS NOT OLD.task_id OR NEW.workflow_generation IS NOT OLD.workflow_generation OR NEW.attempt_index IS NOT OLD.attempt_index OR NEW.owner_epoch IS NOT OLD.owner_epoch OR NEW.unit_id IS NOT OLD.unit_id OR NEW.execution_generation IS NOT OLD.execution_generation OR NEW.allocated_session_id IS NOT OLD.allocated_session_id OR NEW.owner_id IS NOT OLD.owner_id OR NEW.pair_id IS NOT OLD.pair_id OR NEW.marker_project_version IS NOT OLD.marker_project_version OR NEW.marker_goal_version IS NOT OLD.marker_goal_version OR NEW.marker_task_version IS NOT OLD.marker_task_version OR NEW.marker_workflow_version IS NOT OLD.marker_workflow_version OR NEW.marker_project_sha256 IS NOT OLD.marker_project_sha256 OR NEW.marker_goal_sha256 IS NOT OLD.marker_goal_sha256 OR NEW.marker_task_sha256 IS NOT OLD.marker_task_sha256 OR NEW.marker_workflow_sha256 IS NOT OLD.marker_workflow_sha256 OR NEW.marker_digest IS NOT OLD.marker_digest OR NEW.origin IS NOT OLD.origin OR NEW.phase IS NOT OLD.phase OR NEW.actor IS NOT OLD.actor OR NEW.provider IS NOT OLD.provider OR NEW.alias IS NOT OLD.alias OR NEW.role IS NOT OLD.role OR NEW.context_version IS NOT OLD.context_version OR NEW.version<>OLD.version+1 OR NEW.phase_open>OLD.phase_open BEGIN SELECT RAISE(ABORT,'managed core or fact transition differs'); END;
CREATE TRIGGER binding_managed_marker_bodies_INSERT BEFORE INSERT ON managed_marker_bodies WHEN NOT rrx_binding_permit('managed_marker_bodies','INSERT',NULL,NULL,NULL,NULL,NULL,NEW.operation_id,NEW.workflow_id,NEW.workflow_version,NEW.workflow_sha256,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_marker_bodies_UPDATE BEFORE UPDATE ON managed_marker_bodies WHEN NOT rrx_binding_permit('managed_marker_bodies','UPDATE',OLD.operation_id,OLD.workflow_id,OLD.workflow_version,OLD.workflow_sha256,OLD.body,NEW.operation_id,NEW.workflow_id,NEW.workflow_version,NEW.workflow_sha256,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_marker_bodies_DELETE BEFORE DELETE ON managed_marker_bodies WHEN NOT rrx_binding_permit('managed_marker_bodies','DELETE',OLD.operation_id,OLD.workflow_id,OLD.workflow_version,OLD.workflow_sha256,OLD.body,NULL,NULL,NULL,NULL,NULL) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_marker_bodies_no_replace BEFORE INSERT ON managed_marker_bodies WHEN EXISTS(SELECT 1 FROM managed_marker_bodies WHERE operation_id=NEW.operation_id) BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_marker_bodies_no_delete BEFORE DELETE ON managed_marker_bodies BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_marker_bodies_immutable BEFORE UPDATE ON managed_marker_bodies BEGIN SELECT RAISE(ABORT,'managed core is immutable'); END;
CREATE TRIGGER binding_managed_phase_owners_INSERT BEFORE INSERT ON managed_phase_owners WHEN NOT rrx_binding_permit('managed_phase_owners','INSERT',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NEW.owner_id,NEW.operation_id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.unit_id,NEW.owner_epoch,NEW.execution_generation,NEW.allocated_session_id,NEW.provider,NEW.alias,NEW.role,NEW.worktree,NEW.origin,NEW.native_invocation_id,NEW.validated,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_owners_UPDATE BEFORE UPDATE ON managed_phase_owners WHEN NOT rrx_binding_permit('managed_phase_owners','UPDATE',OLD.owner_id,OLD.operation_id,OLD.project_id,OLD.goal_id,OLD.task_id,OLD.unit_id,OLD.owner_epoch,OLD.execution_generation,OLD.allocated_session_id,OLD.provider,OLD.alias,OLD.role,OLD.worktree,OLD.origin,OLD.native_invocation_id,OLD.validated,OLD.version,OLD.body,NEW.owner_id,NEW.operation_id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.unit_id,NEW.owner_epoch,NEW.execution_generation,NEW.allocated_session_id,NEW.provider,NEW.alias,NEW.role,NEW.worktree,NEW.origin,NEW.native_invocation_id,NEW.validated,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_owners_DELETE BEFORE DELETE ON managed_phase_owners WHEN NOT rrx_binding_permit('managed_phase_owners','DELETE',OLD.owner_id,OLD.operation_id,OLD.project_id,OLD.goal_id,OLD.task_id,OLD.unit_id,OLD.owner_epoch,OLD.execution_generation,OLD.allocated_session_id,OLD.provider,OLD.alias,OLD.role,OLD.worktree,OLD.origin,OLD.native_invocation_id,OLD.validated,OLD.version,OLD.body,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_owners_no_replace BEFORE INSERT ON managed_phase_owners WHEN EXISTS(SELECT 1 FROM managed_phase_owners WHERE owner_id=NEW.owner_id) BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_owners_no_delete BEFORE DELETE ON managed_phase_owners BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_owners_core BEFORE UPDATE ON managed_phase_owners WHEN NEW.owner_id IS NOT OLD.owner_id OR NEW.operation_id IS NOT OLD.operation_id OR NEW.project_id IS NOT OLD.project_id OR NEW.goal_id IS NOT OLD.goal_id OR NEW.task_id IS NOT OLD.task_id OR NEW.unit_id IS NOT OLD.unit_id OR NEW.owner_epoch IS NOT OLD.owner_epoch OR NEW.execution_generation IS NOT OLD.execution_generation OR NEW.allocated_session_id IS NOT OLD.allocated_session_id OR NEW.provider IS NOT OLD.provider OR NEW.alias IS NOT OLD.alias OR NEW.role IS NOT OLD.role OR NEW.worktree IS NOT OLD.worktree OR NEW.origin IS NOT OLD.origin OR NEW.version<>OLD.version+1 OR NEW.validated<OLD.validated OR (OLD.native_invocation_id IS NOT NULL AND NEW.native_invocation_id IS NOT OLD.native_invocation_id) BEGIN SELECT RAISE(ABORT,'managed core or fact transition differs'); END;
CREATE TRIGGER binding_managed_phase_inputs_INSERT BEFORE INSERT ON managed_phase_inputs WHEN NOT rrx_binding_permit('managed_phase_inputs','INSERT',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NEW.pair_id,NEW.owner_id,NEW.operation_id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.context_version,NEW.context_digest,NEW.revision,NEW.payload_sha256,NEW.payload_bytes,NEW.profile_digest,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_inputs_UPDATE BEFORE UPDATE ON managed_phase_inputs WHEN NOT rrx_binding_permit('managed_phase_inputs','UPDATE',OLD.pair_id,OLD.owner_id,OLD.operation_id,OLD.project_id,OLD.goal_id,OLD.task_id,OLD.context_version,OLD.context_digest,OLD.revision,OLD.payload_sha256,OLD.payload_bytes,OLD.profile_digest,OLD.version,OLD.body,NEW.pair_id,NEW.owner_id,NEW.operation_id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.context_version,NEW.context_digest,NEW.revision,NEW.payload_sha256,NEW.payload_bytes,NEW.profile_digest,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_inputs_DELETE BEFORE DELETE ON managed_phase_inputs WHEN NOT rrx_binding_permit('managed_phase_inputs','DELETE',OLD.pair_id,OLD.owner_id,OLD.operation_id,OLD.project_id,OLD.goal_id,OLD.task_id,OLD.context_version,OLD.context_digest,OLD.revision,OLD.payload_sha256,OLD.payload_bytes,OLD.profile_digest,OLD.version,OLD.body,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_inputs_no_replace BEFORE INSERT ON managed_phase_inputs WHEN EXISTS(SELECT 1 FROM managed_phase_inputs WHERE pair_id=NEW.pair_id) BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_inputs_no_delete BEFORE DELETE ON managed_phase_inputs BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_inputs_immutable BEFORE UPDATE ON managed_phase_inputs BEGIN SELECT RAISE(ABORT,'managed core is immutable'); END;
CREATE TRIGGER binding_managed_phase_admissions_INSERT BEFORE INSERT ON managed_phase_admissions WHEN NOT rrx_binding_permit('managed_phase_admissions','INSERT',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NEW.pair_id,NEW.operation_id,NEW.owner_id,NEW.native_invocation_id,NEW.input_effect_id,NEW.frame_sha256,NEW.native_thread,NEW.native_turn,NEW.confirmed,NEW.uncertain,NEW.settled,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_admissions_UPDATE BEFORE UPDATE ON managed_phase_admissions WHEN NOT rrx_binding_permit('managed_phase_admissions','UPDATE',OLD.pair_id,OLD.operation_id,OLD.owner_id,OLD.native_invocation_id,OLD.input_effect_id,OLD.frame_sha256,OLD.native_thread,OLD.native_turn,OLD.confirmed,OLD.uncertain,OLD.settled,OLD.version,OLD.body,NEW.pair_id,NEW.operation_id,NEW.owner_id,NEW.native_invocation_id,NEW.input_effect_id,NEW.frame_sha256,NEW.native_thread,NEW.native_turn,NEW.confirmed,NEW.uncertain,NEW.settled,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_admissions_DELETE BEFORE DELETE ON managed_phase_admissions WHEN NOT rrx_binding_permit('managed_phase_admissions','DELETE',OLD.pair_id,OLD.operation_id,OLD.owner_id,OLD.native_invocation_id,OLD.input_effect_id,OLD.frame_sha256,OLD.native_thread,OLD.native_turn,OLD.confirmed,OLD.uncertain,OLD.settled,OLD.version,OLD.body,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_admissions_no_replace BEFORE INSERT ON managed_phase_admissions WHEN EXISTS(SELECT 1 FROM managed_phase_admissions WHERE pair_id=NEW.pair_id) BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_admissions_no_delete BEFORE DELETE ON managed_phase_admissions BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_admissions_core BEFORE UPDATE ON managed_phase_admissions WHEN NEW.pair_id IS NOT OLD.pair_id OR NEW.operation_id IS NOT OLD.operation_id OR NEW.owner_id IS NOT OLD.owner_id OR NEW.native_invocation_id IS NOT OLD.native_invocation_id OR NEW.input_effect_id IS NOT OLD.input_effect_id OR NEW.frame_sha256 IS NOT OLD.frame_sha256 OR NEW.version<>OLD.version+1 OR NEW.confirmed<OLD.confirmed OR NEW.uncertain<OLD.uncertain OR NEW.settled<OLD.settled OR (OLD.native_thread IS NOT NULL AND NEW.native_thread IS NOT OLD.native_thread) OR (OLD.native_turn IS NOT NULL AND NEW.native_turn IS NOT OLD.native_turn) BEGIN SELECT RAISE(ABORT,'managed core or fact transition differs'); END;
CREATE TRIGGER binding_managed_phase_readiness_INSERT BEFORE INSERT ON managed_phase_readiness WHEN NOT rrx_binding_permit('managed_phase_readiness','INSERT',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NEW.operation_id,NEW.origin,NEW.owner_epoch,NEW.state,NEW.start_ended,NEW.known_terminal,NEW.parking_version,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_readiness_UPDATE BEFORE UPDATE ON managed_phase_readiness WHEN NOT rrx_binding_permit('managed_phase_readiness','UPDATE',OLD.operation_id,OLD.origin,OLD.owner_epoch,OLD.state,OLD.start_ended,OLD.known_terminal,OLD.parking_version,OLD.version,OLD.body,NEW.operation_id,NEW.origin,NEW.owner_epoch,NEW.state,NEW.start_ended,NEW.known_terminal,NEW.parking_version,NEW.version,NEW.body) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_readiness_DELETE BEFORE DELETE ON managed_phase_readiness WHEN NOT rrx_binding_permit('managed_phase_readiness','DELETE',OLD.operation_id,OLD.origin,OLD.owner_epoch,OLD.state,OLD.start_ended,OLD.known_terminal,OLD.parking_version,OLD.version,OLD.body,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL) BEGIN SELECT RAISE(ABORT,'exact managed mutation permission required'); END;
CREATE TRIGGER binding_managed_phase_readiness_no_replace BEFORE INSERT ON managed_phase_readiness WHEN EXISTS(SELECT 1 FROM managed_phase_readiness WHERE operation_id=NEW.operation_id) BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_readiness_no_delete BEFORE DELETE ON managed_phase_readiness BEGIN SELECT RAISE(ABORT,'managed identity is retained'); END;
CREATE TRIGGER binding_managed_phase_readiness_core BEFORE UPDATE ON managed_phase_readiness WHEN NEW.operation_id IS NOT OLD.operation_id OR NEW.origin IS NOT OLD.origin OR NEW.owner_epoch IS NOT OLD.owner_epoch OR NEW.version<>OLD.version+1 OR NEW.start_ended<OLD.start_ended OR NEW.known_terminal<OLD.known_terminal BEGIN SELECT RAISE(ABORT,'managed core or fact transition differs'); END;
CREATE TRIGGER binding_admission_not_parked BEFORE INSERT ON managed_phase_admissions WHEN EXISTS(SELECT 1 FROM managed_phase_readiness WHERE operation_id=NEW.operation_id AND state='parked') BEGIN SELECT RAISE(ABORT,'current parking excludes consumption'); END;
CREATE TRIGGER binding_readiness_not_consumed BEFORE UPDATE ON managed_phase_readiness WHEN NEW.state='parked' AND EXISTS(SELECT 1 FROM managed_phase_admissions WHERE operation_id=NEW.operation_id) BEGIN SELECT RAISE(ABORT,'consumption excludes current parking'); END;
CREATE TRIGGER binding_readiness_insert_not_consumed BEFORE INSERT ON managed_phase_readiness WHEN NEW.state='parked' AND EXISTS(SELECT 1 FROM managed_phase_admissions WHERE operation_id=NEW.operation_id) BEGIN SELECT RAISE(ABORT,'consumption excludes current parking'); END;
CREATE TRIGGER binding_owner_allocation BEFORE INSERT ON managed_phase_owners
WHEN NOT EXISTS(SELECT 1 FROM managed_phase_operations o WHERE o.operation_id=NEW.operation_id AND o.owner_id=NEW.owner_id AND o.allocated_session_id=NEW.allocated_session_id AND o.project_id=NEW.project_id AND o.goal_id=NEW.goal_id AND o.task_id=NEW.task_id AND o.unit_id=NEW.unit_id AND o.owner_epoch=NEW.owner_epoch AND o.execution_generation=NEW.execution_generation AND o.provider=NEW.provider AND o.alias=NEW.alias AND o.role=NEW.role AND o.origin=NEW.origin)
BEGIN SELECT RAISE(ABORT,'owner allocation differs'); END;
CREATE TRIGGER binding_input_allocation BEFORE INSERT ON managed_phase_inputs
WHEN NOT EXISTS(SELECT 1 FROM managed_phase_operations o JOIN managed_phase_owners w ON w.owner_id=o.owner_id WHERE o.operation_id=NEW.operation_id AND o.pair_id=NEW.pair_id AND o.owner_id=NEW.owner_id AND o.project_id=NEW.project_id AND o.goal_id=NEW.goal_id AND o.task_id=NEW.task_id AND o.context_version=NEW.context_version)
BEGIN SELECT RAISE(ABORT,'input allocation differs'); END;
CREATE TRIGGER binding_marker_identity BEFORE INSERT ON managed_marker_bodies
WHEN NOT EXISTS(SELECT 1 FROM managed_phase_operations o WHERE o.operation_id=NEW.operation_id AND o.workflow_id=NEW.workflow_id AND o.marker_workflow_version=NEW.workflow_version AND o.marker_workflow_sha256=NEW.workflow_sha256)
BEGIN SELECT RAISE(ABORT,'original marker body identity differs'); END;
CREATE TRIGGER binding_operation_allocation BEFORE INSERT ON managed_phase_operations
WHEN NOT EXISTS(SELECT 1 FROM workflow_native_contracts c JOIN execution_units u ON u.id=NEW.unit_id WHERE c.workflow_id=NEW.workflow_id AND c.contract_state='composed' AND c.project_id=NEW.project_id AND c.goal_id=NEW.goal_id AND c.task_id=NEW.task_id AND c.owner_epoch=NEW.owner_epoch AND u.project_id=NEW.project_id AND u.goal_id=NEW.goal_id AND u.task_id=NEW.task_id AND u.owner_epoch=NEW.owner_epoch AND u.generation=NEW.execution_generation AND u.provider=NEW.provider)
BEGIN SELECT RAISE(ABORT,'operation allocation differs'); END;
CREATE TRIGGER binding_admission_allocation BEFORE INSERT ON managed_phase_admissions
WHEN NOT EXISTS(SELECT 1 FROM managed_phase_operations o JOIN managed_phase_owners w ON w.operation_id=o.operation_id JOIN managed_phase_inputs i ON i.operation_id=o.operation_id JOIN native_invocations n ON n.id=NEW.native_invocation_id JOIN managed_effects e ON e.id=NEW.input_effect_id WHERE o.operation_id=NEW.operation_id AND w.owner_id=NEW.owner_id AND i.pair_id=NEW.pair_id AND w.validated=1 AND w.native_invocation_id=n.id AND n.unit_id=o.unit_id AND n.session_id=o.allocated_session_id AND n.owner_epoch=o.owner_epoch AND n.generation=o.execution_generation AND e.unit_id=o.unit_id AND json_extract(e.body,'$.kind')='native_input' AND o.phase_open=1)
BEGIN SELECT RAISE(ABORT,'input admission allocation differs'); END;
CREATE TRIGGER binding_owner_registration BEFORE UPDATE ON managed_phase_owners
WHEN NEW.native_invocation_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM native_invocations n WHERE n.id=NEW.native_invocation_id AND n.unit_id=NEW.unit_id AND n.session_id=NEW.allocated_session_id AND n.project_id=NEW.project_id AND n.goal_id=NEW.goal_id AND n.task_id=NEW.task_id AND n.owner_epoch=NEW.owner_epoch AND n.generation=NEW.execution_generation AND n.provider=NEW.provider)
BEGIN SELECT RAISE(ABORT,'registered owner invocation differs'); END;
CREATE TRIGGER binding_contract_scope BEFORE INSERT ON workflow_native_contracts
WHEN NOT EXISTS(SELECT 1 FROM records r WHERE r.id=NEW.workflow_id AND r.kind='workflow' AND r.project_id=NEW.project_id AND r.goal_id=NEW.goal_id AND r.task_id=NEW.task_id)
BEGIN SELECT RAISE(ABORT,'Workflow contract scope differs'); END;

CREATE TRIGGER binding_readiness_origin BEFORE INSERT ON managed_phase_readiness
WHEN NOT EXISTS(SELECT 1 FROM managed_phase_operations o WHERE o.operation_id=NEW.operation_id AND o.origin=NEW.origin AND o.owner_epoch=NEW.owner_epoch)
BEGIN SELECT RAISE(ABORT,'readiness origin differs'); END;
