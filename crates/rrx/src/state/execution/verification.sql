CREATE TABLE IF NOT EXISTS verification_profiles (
    project_id TEXT PRIMARY KEY NOT NULL REFERENCES projects(id),
    digest TEXT NOT NULL UNIQUE CHECK(length(digest)=64),
    body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=1048576),
    UNIQUE(project_id,digest)
);
CREATE TRIGGER IF NOT EXISTS verification_profile_no_update BEFORE UPDATE ON verification_profiles
BEGIN SELECT RAISE(ABORT,'verification profile activation is immutable'); END;
CREATE TRIGGER IF NOT EXISTS verification_profile_no_delete BEFORE DELETE ON verification_profiles
BEGIN SELECT RAISE(ABORT,'verification profile is retained'); END;
CREATE TABLE IF NOT EXISTS workflow_verification_contracts (
    workflow_id TEXT PRIMARY KEY NOT NULL REFERENCES records(id),
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL, task_id TEXT NOT NULL UNIQUE,
    owner_epoch INTEGER NOT NULL CHECK(owner_epoch>=0),
    profile_digest TEXT,
    CHECK(owner_epoch>0 OR profile_digest IS NULL),
    FOREIGN KEY(project_id,profile_digest) REFERENCES verification_profiles(project_id,digest),
    FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id)
);
CREATE TRIGGER IF NOT EXISTS verification_contract_no_update BEFORE UPDATE ON workflow_verification_contracts
BEGIN SELECT RAISE(ABORT,'managed verification contract is immutable'); END;
CREATE TRIGGER IF NOT EXISTS verification_contract_no_delete BEFORE DELETE ON workflow_verification_contracts
BEGIN SELECT RAISE(ABORT,'managed verification contract is retained'); END;
CREATE TABLE IF NOT EXISTS verification_runs (
    unit_id TEXT PRIMARY KEY NOT NULL,
    workflow_id TEXT NOT NULL REFERENCES workflow_verification_contracts(workflow_id),
    project_id TEXT NOT NULL,goal_id TEXT NOT NULL,task_id TEXT NOT NULL,
    generation INTEGER NOT NULL CHECK(generation>0),owner_epoch INTEGER NOT NULL CHECK(owner_epoch>0),
    profile_digest TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('admitted','terminal','accepted','unknown')),
    version INTEGER NOT NULL CHECK(version>0),
    claim_digest TEXT NOT NULL CHECK(length(claim_digest)=64),
    claim TEXT NOT NULL CHECK(json_valid(claim) AND length(CAST(claim AS BLOB))<=262144),
    snapshot_digest TEXT,
    body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=262144),
    FOREIGN KEY(unit_id,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id),
    FOREIGN KEY(project_id,profile_digest) REFERENCES verification_profiles(project_id,digest)
);
CREATE TRIGGER IF NOT EXISTS verification_run_identity BEFORE UPDATE ON verification_runs
WHEN OLD.unit_id<>NEW.unit_id OR OLD.workflow_id<>NEW.workflow_id OR OLD.project_id<>NEW.project_id
 OR OLD.goal_id<>NEW.goal_id OR OLD.task_id<>NEW.task_id OR OLD.generation<>NEW.generation
 OR OLD.owner_epoch<>NEW.owner_epoch OR OLD.profile_digest<>NEW.profile_digest
 OR OLD.claim<>NEW.claim OR OLD.claim_digest<>NEW.claim_digest OR NEW.version<>OLD.version+1
 OR NOT ((OLD.state='admitted' AND NEW.state IN ('terminal','unknown'))
     OR (OLD.state='terminal' AND NEW.state='accepted' AND OLD.body=NEW.body AND OLD.snapshot_digest IS NEW.snapshot_digest))
BEGIN SELECT RAISE(ABORT,'verification run provenance is immutable'); END;
CREATE TRIGGER IF NOT EXISTS verification_run_no_delete BEFORE DELETE ON verification_runs
BEGIN SELECT RAISE(ABORT,'verification run is retained'); END;
CREATE TABLE IF NOT EXISTS verification_commands (
    unit_id TEXT NOT NULL REFERENCES verification_runs(unit_id),
    ordinal INTEGER NOT NULL CHECK(ordinal>=0 AND ordinal<32),
    operation_id TEXT NOT NULL UNIQUE REFERENCES managed_effects(id),
    state TEXT NOT NULL CHECK(state IN ('pending','terminal')),
    body TEXT CHECK(body IS NULL OR (json_valid(body) AND length(CAST(body AS BLOB))<=65536)),
    PRIMARY KEY(unit_id,ordinal),
    CHECK((state='pending' AND body IS NULL) OR (state='terminal' AND body IS NOT NULL))
);
CREATE TRIGGER IF NOT EXISTS verification_command_identity BEFORE UPDATE ON verification_commands
WHEN OLD.unit_id<>NEW.unit_id OR OLD.ordinal<>NEW.ordinal OR OLD.operation_id<>NEW.operation_id
 OR OLD.state<>'pending' OR NEW.state<>'terminal'
BEGIN SELECT RAISE(ABORT,'verification command receipt is immutable'); END;
CREATE TRIGGER IF NOT EXISTS verification_command_no_delete BEFORE DELETE ON verification_commands
BEGIN SELECT RAISE(ABORT,'verification command is retained'); END;
