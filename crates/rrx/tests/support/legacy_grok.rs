//! The shared Grok fixture's legacy rows in the integration crate (the lib
//! crate provides the same API over `crate::runtime`): a Git Project
//! registered through `ProjectRegistry::add`, then `legacy::goals`.
#[path = "legacy.rs"]
mod legacy;
use rrx::{
    adapter::SharedStore,
    config::Config,
    domain::Task,
    project::{AddProject, ProjectRegistry},
    state::Store,
};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub use legacy::Planned;

/// Keeps the fixture directory alive: `<root>/` is the Project root and
/// `<state>` the migrated legacy database.
pub struct Holder {
    dir: tempfile::TempDir,
}
impl Holder {
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

fn git(root: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "SETUP: git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Legacy rows for `tasks` on a Git Project at `<dir>/<root>` (branch `main`,
/// one empty commit, then `seed`), migrated into `<dir>/<state>`.
pub fn blocking(
    root: &'static str,
    state: &'static str,
    seed: impl FnOnce(&Path) + Send + 'static,
    tasks: Vec<Planned>,
) -> (Holder, SharedStore, Vec<Task>) {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let project_root = base.join(root);
    std::fs::create_dir(&project_root).unwrap();
    git(&project_root, &["init", "-b", "main"]);
    git(
        &project_root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-m",
            "fixture",
        ],
    );
    seed(&project_root);
    let db = base.join(state);
    let project = {
        let mut store = Store::open(&db).unwrap();
        ProjectRegistry::new(&mut store)
            .add(&project_root, AddProject::default(), &Config::default())
            .unwrap()
    };
    let (_, tasks) = legacy::goals(&db, vec![(project.id, "native", tasks)])
        .into_iter()
        .next()
        .unwrap();
    let store = Store::open(&db).unwrap();
    (Holder { dir }, Arc::new(Mutex::new(store)), tasks)
}
