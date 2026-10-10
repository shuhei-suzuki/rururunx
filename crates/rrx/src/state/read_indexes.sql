-- Schema 11 (#81 S4 D6-R3 R6): read-surface seek index only. Historical
-- layout files stay byte-identical; this file is installed fresh, at the
-- ordered step to 11, and in the current-layout reference.
CREATE INDEX goals_by_project ON goals(project_id, id);
