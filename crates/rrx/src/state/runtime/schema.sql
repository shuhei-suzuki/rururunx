-- Canonical accepted definitions remain in goals.body; this row stores provenance only.
CREATE TABLE goal_authority (
 goal_id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL,
 definition_sha256 TEXT NOT NULL CHECK(length(definition_sha256)=64 AND definition_sha256 NOT GLOB '*[^0-9a-f]*'),
 accepted_epoch INTEGER NOT NULL CHECK(typeof(accepted_epoch)='integer' AND accepted_epoch>0),
 ingress_uid INTEGER NOT NULL CHECK(typeof(ingress_uid)='integer' AND ingress_uid>=0),
 policy_sha256 TEXT NOT NULL CHECK(length(policy_sha256)=64 AND policy_sha256 NOT GLOB '*[^0-9a-f]*'),
 version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version=1),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id)
);
CREATE TRIGGER goal_authority_no_update BEFORE UPDATE ON goal_authority
BEGIN SELECT RAISE(ABORT,'accepted Goal definition is immutable'); END;
CREATE TRIGGER goal_authority_no_delete BEFORE DELETE ON goal_authority
BEGIN SELECT RAISE(ABORT,'accepted Goal history is retained'); END;
CREATE TRIGGER goal_authority_no_replace BEFORE INSERT ON goal_authority
WHEN EXISTS(SELECT 1 FROM goal_authority WHERE goal_id=NEW.goal_id)
BEGIN SELECT RAISE(ABORT,'accepted Goal cannot be replaced'); END;
CREATE TRIGGER runtime_goal_no_replace BEFORE INSERT ON goals
WHEN EXISTS(SELECT 1 FROM goal_authority WHERE goal_id=NEW.id)
BEGIN SELECT RAISE(ABORT,'accepted Goal cannot be replaced'); END;
CREATE TRIGGER runtime_goal_definition BEFORE UPDATE ON goals
WHEN EXISTS(SELECT 1 FROM goal_authority WHERE goal_id=OLD.id) AND (
 OLD.id<>NEW.id OR OLD.project_id<>NEW.project_id
 OR json_extract(OLD.body,'$.title') IS NOT json_extract(NEW.body,'$.title')
 OR json_extract(OLD.body,'$.objective') IS NOT json_extract(NEW.body,'$.objective')
 OR json_extract(OLD.body,'$.completion_criteria') IS NOT json_extract(NEW.body,'$.completion_criteria')
 OR json_extract(OLD.body,'$.constraints') IS NOT json_extract(NEW.body,'$.constraints')
 OR json_extract(OLD.body,'$.non_goals') IS NOT json_extract(NEW.body,'$.non_goals')
 OR json_extract(OLD.body,'$.source_refs') IS NOT json_extract(NEW.body,'$.source_refs'))
BEGIN SELECT RAISE(ABORT,'accepted Goal definition is immutable'); END;
CREATE TRIGGER runtime_goal_no_delete BEFORE DELETE ON goals
WHEN EXISTS(SELECT 1 FROM goal_authority WHERE goal_id=OLD.id)
BEGIN SELECT RAISE(ABORT,'accepted Goal history is retained'); END;

CREATE TABLE runtime_control_acks (
 request_id TEXT PRIMARY KEY NOT NULL, instance TEXT NOT NULL,
 action_sha256 TEXT NOT NULL CHECK(length(action_sha256)=64 AND action_sha256 NOT GLOB '*[^0-9a-f]*'), ingress_uid INTEGER NOT NULL,
 accepted_epoch INTEGER NOT NULL CHECK(typeof(accepted_epoch)='integer' AND accepted_epoch>0),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536)
);
CREATE TRIGGER runtime_control_acks_no_update BEFORE UPDATE ON runtime_control_acks
BEGIN SELECT RAISE(ABORT,'control decisions are immutable'); END;
CREATE TRIGGER runtime_control_acks_no_delete BEFORE DELETE ON runtime_control_acks
BEGIN SELECT RAISE(ABORT,'control decisions are retained'); END;
CREATE TRIGGER runtime_control_acks_no_replace BEFORE INSERT ON runtime_control_acks
WHEN EXISTS(SELECT 1 FROM runtime_control_acks WHERE request_id=NEW.request_id)
BEGIN SELECT RAISE(ABORT,'control decision cannot be replaced'); END;

CREATE TABLE scheduler_clock (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1), sequence INTEGER NOT NULL CHECK(typeof(sequence)='integer' AND sequence>=0)
);
INSERT INTO scheduler_clock(singleton,sequence) VALUES(1,0);
CREATE TABLE scheduler_projects (
 project_id TEXT PRIMARY KEY REFERENCES projects(id), rotation INTEGER NOT NULL CHECK(typeof(rotation)='integer' AND rotation>=0)
);
CREATE TABLE scheduler_goals (
 goal_id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL,
 rotation INTEGER NOT NULL CHECK(typeof(rotation)='integer' AND rotation>=0),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id)
);
CREATE TABLE scheduler_tasks (
 task_id TEXT PRIMARY KEY NOT NULL, goal_id TEXT NOT NULL, project_id TEXT NOT NULL,
 queue_sequence INTEGER UNIQUE NOT NULL CHECK(typeof(queue_sequence)='integer' AND queue_sequence>0),
 attention TEXT CHECK(attention IS NULL OR length(CAST(attention AS BLOB))<=65536),
 next_due INTEGER,
 FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id)
);
CREATE TRIGGER scheduler_tasks_scope BEFORE UPDATE ON scheduler_tasks
WHEN OLD.task_id<>NEW.task_id OR OLD.goal_id<>NEW.goal_id OR OLD.project_id<>NEW.project_id OR OLD.queue_sequence<>NEW.queue_sequence
BEGIN SELECT RAISE(ABORT,'scheduler Task scope is immutable'); END;
CREATE TRIGGER scheduler_tasks_no_replace BEFORE INSERT ON scheduler_tasks
WHEN EXISTS(SELECT 1 FROM scheduler_tasks WHERE task_id=NEW.task_id)
BEGIN SELECT RAISE(ABORT,'scheduler Task cannot be replaced'); END;
CREATE TRIGGER scheduler_tasks_no_delete BEFORE DELETE ON scheduler_tasks
BEGIN SELECT RAISE(ABORT,'scheduler Task history is retained'); END;

CREATE TABLE task_drivers (
 task_id TEXT PRIMARY KEY NOT NULL, goal_id TEXT NOT NULL, project_id TEXT NOT NULL,
 id TEXT UNIQUE NOT NULL, owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>0),
 version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
 state TEXT NOT NULL CHECK(state IN ('driving','parked','invalid')),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=131072
 AND json_extract(body,'$.id') IS id
 AND json_extract(body,'$.epoch') IS owner_epoch
 AND json_extract(body,'$.version') IS version
 AND json_extract(body,'$.state') IS state
 AND json_extract(body,'$.pins.scope.task_id') IS task_id
 AND json_extract(body,'$.pins.scope.goal_id') IS goal_id
 AND json_extract(body,'$.pins.scope.project_id') IS project_id),
 FOREIGN KEY(task_id,goal_id,project_id) REFERENCES tasks(id,goal_id,project_id)
);
CREATE TRIGGER task_drivers_transition BEFORE UPDATE ON task_drivers
WHEN OLD.task_id<>NEW.task_id OR OLD.goal_id<>NEW.goal_id OR OLD.project_id<>NEW.project_id
 OR NEW.version<>OLD.version+1
 OR (OLD.id=NEW.id AND OLD.owner_epoch<>NEW.owner_epoch)
 OR (OLD.id=NEW.id AND OLD.state<>'driving' AND NEW.state='driving')
 OR (OLD.id<>NEW.id AND (OLD.state='driving' OR NEW.state<>'driving'))
BEGIN SELECT RAISE(ABORT,'invalid Driver transition'); END;
CREATE TRIGGER task_drivers_no_replace BEFORE INSERT ON task_drivers
WHEN EXISTS(SELECT 1 FROM task_drivers WHERE task_id=NEW.task_id OR id=NEW.id)
BEGIN SELECT RAISE(ABORT,'Driver cannot be replaced'); END;
CREATE TRIGGER task_drivers_no_delete BEFORE DELETE ON task_drivers
BEGIN SELECT RAISE(ABORT,'Driver history is retained'); END;

CREATE TABLE goal_observations (
 goal_id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id)
);
CREATE TRIGGER goal_observations_scope BEFORE UPDATE ON goal_observations
WHEN OLD.goal_id<>NEW.goal_id OR OLD.project_id<>NEW.project_id OR NEW.version<>OLD.version+1
BEGIN SELECT RAISE(ABORT,'invalid Goal observation'); END;
CREATE TRIGGER goal_observations_no_replace BEFORE INSERT ON goal_observations
WHEN EXISTS(SELECT 1 FROM goal_observations WHERE goal_id=NEW.goal_id)
BEGIN SELECT RAISE(ABORT,'Goal observation cannot be replaced'); END;
CREATE TRIGGER goal_observations_no_delete BEFORE DELETE ON goal_observations
BEGIN SELECT RAISE(ABORT,'Goal observations are retained'); END;

CREATE TRIGGER scheduler_clock_scope BEFORE UPDATE ON scheduler_clock
WHEN NEW.singleton<>OLD.singleton OR NEW.sequence<OLD.sequence
BEGIN SELECT RAISE(ABORT,'scheduler clock cannot move backwards'); END;
CREATE TRIGGER scheduler_clock_no_replace BEFORE INSERT ON scheduler_clock
WHEN EXISTS(SELECT 1 FROM scheduler_clock WHERE singleton=NEW.singleton)
BEGIN SELECT RAISE(ABORT,'scheduler clock cannot be replaced'); END;
CREATE TRIGGER scheduler_clock_no_delete BEFORE DELETE ON scheduler_clock
BEGIN SELECT RAISE(ABORT,'scheduler clock is retained'); END;
CREATE TRIGGER scheduler_projects_scope BEFORE UPDATE ON scheduler_projects
WHEN OLD.project_id<>NEW.project_id OR NEW.rotation<OLD.rotation
BEGIN SELECT RAISE(ABORT,'scheduler Project rotation invalid'); END;
CREATE TRIGGER scheduler_projects_no_replace BEFORE INSERT ON scheduler_projects
WHEN EXISTS(SELECT 1 FROM scheduler_projects WHERE project_id=NEW.project_id)
BEGIN SELECT RAISE(ABORT,'scheduler Project cannot be replaced'); END;
CREATE TRIGGER scheduler_projects_no_delete BEFORE DELETE ON scheduler_projects
BEGIN SELECT RAISE(ABORT,'scheduler Project is retained'); END;
CREATE TRIGGER scheduler_goals_scope BEFORE UPDATE ON scheduler_goals
WHEN OLD.goal_id<>NEW.goal_id OR OLD.project_id<>NEW.project_id OR NEW.rotation<OLD.rotation
BEGIN SELECT RAISE(ABORT,'scheduler Goal rotation invalid'); END;
CREATE TRIGGER scheduler_goals_no_replace BEFORE INSERT ON scheduler_goals
WHEN EXISTS(SELECT 1 FROM scheduler_goals WHERE goal_id=NEW.goal_id)
BEGIN SELECT RAISE(ABORT,'scheduler Goal cannot be replaced'); END;
CREATE TRIGGER scheduler_goals_no_delete BEFORE DELETE ON scheduler_goals
BEGIN SELECT RAISE(ABORT,'scheduler Goal is retained'); END;
