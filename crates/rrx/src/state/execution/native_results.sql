CREATE TABLE IF NOT EXISTS native_invocations (
    id TEXT PRIMARY KEY NOT NULL,
    unit_id TEXT UNIQUE NOT NULL, session_id TEXT UNIQUE NOT NULL REFERENCES session_units(session_id),
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL, task_id TEXT NOT NULL,
    generation INTEGER NOT NULL CHECK(generation>0), owner_epoch INTEGER NOT NULL CHECK(owner_epoch>0),
    provider TEXT NOT NULL CHECK(provider IN ('claude','codex')),
    state TEXT NOT NULL CHECK(state IN ('not_dispatched','input_pending','acknowledged','closed')),
    version INTEGER NOT NULL CHECK(version>0),
    input_operation TEXT REFERENCES managed_effects(id), native_thread TEXT, native_turn TEXT,
    body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536),
    UNIQUE(id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch),
    FOREIGN KEY(unit_id,project_id,goal_id,task_id) REFERENCES execution_units(id,project_id,goal_id,task_id)
);
CREATE TRIGGER IF NOT EXISTS native_invocation_identity BEFORE UPDATE ON native_invocations
WHEN OLD.id<>NEW.id OR OLD.unit_id<>NEW.unit_id OR OLD.session_id<>NEW.session_id
 OR OLD.project_id<>NEW.project_id OR OLD.goal_id<>NEW.goal_id OR OLD.task_id<>NEW.task_id
 OR OLD.generation<>NEW.generation OR OLD.owner_epoch<>NEW.owner_epoch OR OLD.provider<>NEW.provider
 OR NEW.version<>OLD.version+1
 OR json_remove(OLD.body,'$.version','$.state','$.input_operation','$.frame_sha256','$.native_thread','$.native_turn')
 <>json_remove(NEW.body,'$.version','$.state','$.input_operation','$.frame_sha256','$.native_thread','$.native_turn')
 OR (OLD.input_operation IS NOT NULL AND OLD.input_operation IS NOT NEW.input_operation)
 OR (OLD.native_thread IS NOT NULL AND OLD.native_thread IS NOT NEW.native_thread)
 OR (OLD.native_turn IS NOT NULL AND OLD.native_turn IS NOT NEW.native_turn)
 OR OLD.state='closed'
BEGIN SELECT RAISE(ABORT,'native invocation binding is immutable'); END;
CREATE TRIGGER IF NOT EXISTS native_invocation_no_delete BEFORE DELETE ON native_invocations
BEGIN SELECT RAISE(ABORT,'native invocation is retained'); END;
CREATE TABLE IF NOT EXISTS native_results (
    id TEXT PRIMARY KEY NOT NULL, invocation_id TEXT UNIQUE NOT NULL,
    unit_id TEXT NOT NULL, session_id TEXT NOT NULL,
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL, task_id TEXT NOT NULL,
    generation INTEGER NOT NULL CHECK(generation>0), owner_epoch INTEGER NOT NULL CHECK(owner_epoch>0),
    provider TEXT NOT NULL CHECK(provider IN ('claude','codex')),
    acquisition TEXT NOT NULL CHECK(acquisition IN ('complete','missing','partial','ambiguous','unsupported','overflow')),
    authority TEXT NOT NULL CHECK(authority IN ('owned_terminal','historical_draft')),
    version INTEGER NOT NULL CHECK(version=1),
    body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=2097152),
    FOREIGN KEY(invocation_id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch)
      REFERENCES native_invocations(id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch)
);
CREATE TRIGGER IF NOT EXISTS native_result_no_update BEFORE UPDATE ON native_results
BEGIN SELECT RAISE(ABORT,'native result receipt is immutable'); END;
CREATE TRIGGER IF NOT EXISTS native_result_no_delete BEFORE DELETE ON native_results
BEGIN SELECT RAISE(ABORT,'native result receipt is retained'); END;
