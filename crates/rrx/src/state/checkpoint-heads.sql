CREATE TABLE IF NOT EXISTS checkpoint_heads (
    project_id TEXT NOT NULL,
    goal_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    record_id TEXT UNIQUE NOT NULL,
    PRIMARY KEY(project_id, goal_id, task_id),
    FOREIGN KEY(task_id, goal_id, project_id) REFERENCES tasks(id, goal_id, project_id),
    FOREIGN KEY(record_id, project_id) REFERENCES records(id, project_id)
);
