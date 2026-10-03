CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    root TEXT UNIQUE NOT NULL,
    version INTEGER NOT NULL CHECK(version > 0),
    body TEXT NOT NULL CHECK(json_valid(body))
);
CREATE TABLE goals (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    version INTEGER NOT NULL CHECK(version > 0),
    body TEXT NOT NULL CHECK(json_valid(body)),
    UNIQUE(id, project_id)
);
CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    goal_id TEXT NOT NULL,
    issue INTEGER,
    version INTEGER NOT NULL CHECK(version > 0),
    body TEXT NOT NULL CHECK(json_valid(body)),
    UNIQUE(id, goal_id, project_id),
    FOREIGN KEY(goal_id, project_id) REFERENCES goals(id, project_id)
);
CREATE INDEX tasks_by_project_issue ON tasks(project_id, issue);
CREATE INDEX tasks_by_goal ON tasks(goal_id);
CREATE TABLE records (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id),
    goal_id TEXT,
    task_id TEXT,
    version INTEGER NOT NULL CHECK(version > 0),
    body TEXT NOT NULL CHECK(json_valid(body)),
    UNIQUE(id, project_id),
    CHECK(task_id IS NULL OR goal_id IS NOT NULL),
    FOREIGN KEY(goal_id, project_id) REFERENCES goals(id, project_id),
    FOREIGN KEY(task_id, goal_id, project_id) REFERENCES tasks(id, goal_id, project_id)
);
CREATE INDEX records_by_scope ON records(project_id, goal_id, task_id, kind);
CREATE TABLE context_versions (
    project_id TEXT NOT NULL REFERENCES projects(id),
    goal_id TEXT NOT NULL,
    task_id TEXT,
    owner TEXT NOT NULL,
    version INTEGER NOT NULL CHECK(version > 0),
    body TEXT NOT NULL CHECK(json_valid(body)),
    PRIMARY KEY(project_id, owner, version),
    FOREIGN KEY(goal_id, project_id) REFERENCES goals(id, project_id),
    FOREIGN KEY(task_id, goal_id, project_id) REFERENCES tasks(id, goal_id, project_id)
);
CREATE TABLE usage (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL REFERENCES projects(id),
    goal_id TEXT,
    task_id TEXT,
    session_id TEXT NOT NULL,
    body TEXT NOT NULL CHECK(json_valid(body)),
    CHECK(task_id IS NULL OR goal_id IS NOT NULL),
    FOREIGN KEY(goal_id, project_id) REFERENCES goals(id, project_id),
    FOREIGN KEY(task_id, goal_id, project_id) REFERENCES tasks(id, goal_id, project_id),
    FOREIGN KEY(session_id, project_id) REFERENCES records(id, project_id)
);
CREATE INDEX usage_by_scope ON usage(project_id, goal_id, task_id);
CREATE TABLE audit (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL REFERENCES projects(id),
    goal_id TEXT,
    task_id TEXT,
    kind TEXT NOT NULL,
    at INTEGER NOT NULL,
    data TEXT NOT NULL CHECK(json_valid(data)),
    CHECK(task_id IS NULL OR goal_id IS NOT NULL),
    FOREIGN KEY(goal_id, project_id) REFERENCES goals(id, project_id),
    FOREIGN KEY(task_id, goal_id, project_id) REFERENCES tasks(id, goal_id, project_id)
);
CREATE INDEX audit_by_scope ON audit(project_id, goal_id, task_id, sequence);
CREATE TRIGGER audit_no_update BEFORE UPDATE ON audit BEGIN
    SELECT RAISE(ABORT, 'audit is append-only');
END;
CREATE TRIGGER audit_no_delete BEFORE DELETE ON audit BEGIN
    SELECT RAISE(ABORT, 'audit is append-only');
END;
