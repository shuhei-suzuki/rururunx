use super::*;

fn fixture() -> (tempfile::TempDir, std::sync::Arc<RuntimeOwner>) {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    (dir, owner)
}

/// Refused observations cannot be replaced even with recursive triggers disabled.
#[test]
fn b3_portable_no_replace() {
    let (_dir, owner) = fixture();
    let store = owner.store.lock().unwrap();
    let c = &store.connection;
    c.execute("INSERT INTO pipeline_qualifications(owner_epoch,subject,state,body) VALUES(?1,'platform','refused','{}')", [owner.epoch()]).unwrap();
    c.pragma_update(None, "recursive_triggers", false).unwrap();
    let error = c.execute("INSERT OR REPLACE INTO pipeline_qualifications SELECT * FROM pipeline_qualifications LIMIT 1", []).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pipeline row cannot be replaced"),
        "{error}"
    );
    assert_eq!(
        store.pipeline_qualifications(&owner).unwrap()[0].state,
        "refused"
    );
}

/// Explicit nonpositive rowids cannot enter observation history.
#[test]
fn b3_portable_nonpositive_rowid() {
    let (_dir, owner) = fixture();
    let store = owner.store.lock().unwrap();
    for rowid in [0, -1, -2] {
        let error = store.connection.execute("INSERT INTO pipeline_qualifications(rowid,owner_epoch,subject,state,body) VALUES(?1,?2,'platform','refused','{}')", params![rowid, owner.epoch()]).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("pipeline rowid must be positive"),
            "{error}"
        );
    }
    assert!(store.pipeline_qualifications(&owner).unwrap().is_empty());
}

/// Implicit observation inserts persist positive rowids despite the BEFORE sentinel.
#[test]
fn b3_portable_implicit_rowid() {
    let (_dir, owner) = fixture();
    let store = owner.store.lock().unwrap();
    let c = &store.connection;
    c.execute_batch(
        "CREATE TEMP TABLE rowid_observation(value INTEGER);
        CREATE TEMP TRIGGER observe_rowid BEFORE INSERT ON main.pipeline_qualifications
        BEGIN INSERT INTO rowid_observation VALUES(NEW.rowid); END;",
    )
    .unwrap();
    c.execute("INSERT INTO pipeline_qualifications(owner_epoch,subject,state,body) VALUES(?1,'platform','refused','{}')", [owner.epoch()]).unwrap();
    assert_eq!(
        c.query_row("SELECT value FROM rowid_observation", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        -1
    );
    assert!(
        c.query_row("SELECT rowid FROM pipeline_qualifications", [], |r| r
            .get::<_, i64>(0))
            .unwrap()
            > 0
    );
}

/// An old epoch cannot append observations after a native owner restart.
#[test]
fn b3_portable_epoch_trigger() {
    let (dir, owner) = fixture();
    let old = owner.epoch();
    drop(owner);
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    let store = owner.store.lock().unwrap();
    let error = store.connection.execute("INSERT INTO pipeline_qualifications(owner_epoch,subject,state,body) VALUES(?1,'platform','refused','{}')", [old]).unwrap_err();
    assert!(
        error.to_string().contains("pipeline record owner retired"),
        "{error}"
    );
    store.connection.execute("INSERT INTO pipeline_qualifications(owner_epoch,subject,state,body) VALUES(?1,'platform','refused','{}')", [owner.epoch()]).unwrap();
}
