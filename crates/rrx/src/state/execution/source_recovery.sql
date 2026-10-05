CREATE TABLE IF NOT EXISTS source_recoveries (
    task_id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL, goal_id TEXT NOT NULL,
    id TEXT UNIQUE NOT NULL, owner_epoch INTEGER NOT NULL CHECK(owner_epoch>0),
    version INTEGER NOT NULL CHECK(version>0),
    state TEXT NOT NULL CHECK(state IN ('preparing','installed','invalid')),
    body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=131072),
    FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id)
);
CREATE TRIGGER IF NOT EXISTS source_recovery_identity BEFORE UPDATE ON source_recoveries
WHEN OLD.task_id<>NEW.task_id OR OLD.project_id<>NEW.project_id OR OLD.goal_id<>NEW.goal_id
 OR NEW.version<>OLD.version+1
 OR (OLD.id=NEW.id AND OLD.owner_epoch<>NEW.owner_epoch)
 OR (OLD.id<>NEW.id AND (OLD.state='preparing' OR NEW.state<>'preparing'))
BEGIN SELECT RAISE(ABORT,'invalid source recovery transition'); END;
CREATE TRIGGER IF NOT EXISTS source_recovery_no_replace BEFORE INSERT ON source_recoveries
WHEN EXISTS(SELECT 1 FROM source_recoveries WHERE task_id=NEW.task_id OR id=NEW.id)
BEGIN SELECT RAISE(ABORT,'source recovery cannot be replaced'); END;
CREATE TRIGGER IF NOT EXISTS source_recovery_no_delete BEFORE DELETE ON source_recoveries
BEGIN SELECT RAISE(ABORT,'source recovery history is retained'); END;
