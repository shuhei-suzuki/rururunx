CREATE TABLE runtime_startup_probes (
    owner_epoch INTEGER NOT NULL CHECK(owner_epoch > 0),
    installation TEXT NOT NULL,
    alias TEXT NOT NULL,
    step TEXT NOT NULL CHECK(step IN ('version','schema','handshake')),
    state TEXT NOT NULL CHECK(state IN ('pending','confirmed','unknown')),
    version INTEGER NOT NULL CHECK(version > 0),
    body TEXT NOT NULL CHECK(length(CAST(body AS BLOB)) <= 4096 AND json_valid(body)),
    PRIMARY KEY(owner_epoch,installation,alias,step)
);
