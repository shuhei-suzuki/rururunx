use super::{control::*, goal::*, *};
use crate::{
    config::{AgentConfig, WorkflowClass},
    domain::RiskClass,
};
use uuid::Uuid;

fn config() -> Config {
    let mut c = Config::default();
    c.agents.insert(
        "worker".into(),
        AgentConfig {
            provider: Some("claude".into()),
            command: vec!["/bin/true".into()],
            ..Default::default()
        },
    );
    c
}
fn definition() -> GoalDefinition {
    GoalDefinition {
        title: "t".into(),
        objective: "o".into(),
        criteria: vec![CriterionDefinition {
            id: "c".into(),
            description: "d".into(),
            evaluator: CriterionEvaluator::RequiredTasksVerified,
        }],
        constraints: vec![],
        non_goals: vec![],
        source_refs: vec![],
    }
}
pub(super) fn plan() -> GoalPlan {
    GoalPlan {
        definition: definition(),
        tasks: vec![TaskDefinition {
            key: "one".into(),
            title: "work".into(),
            acceptance_criteria: vec!["verified result".into()],
            executor: "worker".into(),
            reviewers: vec![],
            workflow: WorkflowClass::Quick,
            risk: RiskClass::R2,
        }],
        dependencies: vec![],
    }
}
#[test]
fn canonical_goal_definition_has_framed_order_and_variant_identity() {
    let original = definition();
    let first = original.canonical_digest().unwrap();
    let reversed:GoalDefinition=serde_json::from_str(r#"{"source_refs":[],"non_goals":[],"constraints":[],"criteria":[{"evaluator":{"kind":"required_tasks_verified"},"description":"d","id":"c"}],"objective":"o","title":"t"}"#).unwrap();
    assert_eq!(first, reversed.canonical_digest().unwrap());
    let mut a = original.clone();
    let mut b = original.clone();
    a.constraints = vec!["x".into(), "yz".into()];
    b.constraints = vec!["xy".into(), "z".into()];
    assert_ne!(a.canonical_digest().unwrap(), b.canonical_digest().unwrap());
    b.constraints = vec!["yz".into(), "x".into()];
    assert_ne!(a.canonical_digest().unwrap(), b.canonical_digest().unwrap());
    a.criteria[0].evaluator = CriterionEvaluator::Human {
        goal_pack_input: false,
    };
    b = a.clone();
    b.criteria[0].evaluator = CriterionEvaluator::Human {
        goal_pack_input: true,
    };
    assert_ne!(a.canonical_digest().unwrap(), b.canonical_digest().unwrap());
}
#[test]
fn accepted_definition_never_uses_unverified_boolean_or_duplicate_criterion() {
    let mut d = definition();
    d.criteria.push(d.criteria[0].clone());
    assert!(d.canonical_digest().is_err());
    d.criteria.pop();
    d.criteria[0].evaluator = CriterionEvaluator::Unverified;
    assert!(d.canonical_digest().is_err());
    let mut raw = serde_json::to_value(definition()).unwrap();
    raw["criteria"][0]["satisfied"] = true.into();
    assert!(serde_json::from_value::<GoalDefinition>(raw).is_err());
}
#[test]
fn definition_exact_encoded_byte_budget_and_utf8_text_boundary() {
    let mut d = definition();
    d.constraints = vec!["x".repeat(MAX_TEXT_BYTES); 63];
    // Independent count of the declared framing: domain/title/objective, criterion count,
    // ID/description/variant, and three list counts. Each remaining text has its length prefix.
    let fixed = 8
        + "rrx.goal.definition.v1".len()
        + 9
        + 9
        + 8
        + 9
        + 9
        + 1
        + 8
        + 63 * (8 + MAX_TEXT_BYTES)
        + 8
        + 8
        + 8;
    let remaining = MAX_DEFINITION_BYTES - fixed;
    assert!(remaining < MAX_TEXT_BYTES);
    d.constraints.push("x".repeat(remaining));
    assert!(d.canonical_digest().is_ok(), "inclusive exact1MiB must fit");
    d.constraints.last_mut().unwrap().push('x');
    assert!(d.canonical_digest().is_err(), "encoded1MiB+1 must refuse");
    let mut d = definition();
    d.title = "界".repeat(MAX_TEXT_BYTES / 3) + "a";
    assert_eq!(d.title.len(), MAX_TEXT_BYTES);
    assert!(d.canonical_digest().is_ok());
    d.title.push('x');
    assert!(d.canonical_digest().is_err());
}
#[test]
fn actual_plan_validator_reuses_hard_dag_and_preserves_risk_floor() {
    let configured = config();
    let valid = plan().validate(&configured).unwrap();
    assert_eq!(valid.plan().tasks[0].workflow, WorkflowClass::Standard);
    let mut p = plan();
    p.tasks.push(p.tasks[0].clone());
    assert!(p.validate(&configured).is_err());
    let mut p = plan();
    let mut second = p.tasks[0].clone();
    second.key = "two".into();
    p.tasks.push(second);
    p.dependencies = vec![
        PlanDependency {
            prerequisite: "one".into(),
            dependent: "two".into(),
            hard: true,
        },
        PlanDependency {
            prerequisite: "two".into(),
            dependent: "one".into(),
            hard: true,
        },
    ];
    assert!(p.clone().validate(&configured).is_err());
    p.dependencies[1].hard = false;
    assert!(p.clone().validate(&configured).is_ok());
    p.dependencies[1].dependent = "foreign".into();
    assert!(p.validate(&configured).is_err());
    let mut p = plan();
    p.tasks[0].executor = "foreign".into();
    assert!(p.validate(&configured).is_err());
}
#[tokio::test]
async fn actual_local_ingress_checks_epoch_and_refuses_unknown_project() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    let runtime = Runtime::new(owner.clone(), config()).unwrap();
    let identity = runtime.control_identity();
    assert_eq!(
        identity.state,
        dir.path().join("state.db").canonicalize().unwrap()
    );
    assert_eq!(identity.instance, owner.instance_id());
    assert_eq!(identity.epoch, owner.epoch());
    identity.validate().unwrap();
    assert_eq!(runtime.control_identity(), identity);
    let (server, _client) = tokio::net::UnixStream::pair().unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        action: ControlAction::CreateGoal {
            project: crate::domain::ProjectId::new(),
            expected_project: 1,
            plan: plan(),
        },
    };
    let mut stale = request.clone();
    stale.epoch += 1;
    assert!(runtime.handle_control(&server, stale).await.is_err());
    assert!(runtime.handle_control(&server, request).await.is_err());
    assert!(owner.store().lock().unwrap().projects().unwrap().is_empty());
}

#[tokio::test]
async fn actual_service_resolves_names_uuid_and_cwd_without_source_grant() {
    use crate::domain::{Project, ProjectState};
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    let root = dir.path().join("source");
    std::fs::create_dir_all(root.join("sub")).unwrap();
    let root = root.canonicalize().unwrap();
    let mut project = Project::new(
        "route".into(),
        root.clone(),
        "routing-metadata-fixture".into(),
        "main".into(),
    );
    owner
        .store()
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    let runtime = Runtime::new(owner.clone(), config()).unwrap();
    let (server, _client) = tokio::net::UnixStream::pair().unwrap();
    for selector in [Some("route".into()), Some(project.id.to_string()), None] {
        let request = ControlRequest {
            request_id: Uuid::new_v4(),
            instance: owner.instance_id().into(),
            epoch: owner.epoch(),
            action: ControlAction::ResolveProject {
                selector,
                cwd: root.join("sub"),
            },
        };
        match runtime.handle_control(&server, request).await.unwrap() {
            ControlResponse::ProjectResolved {
                project: id,
                version,
                display_name,
                state,
            } => {
                assert_eq!(id, project.id);
                assert_eq!(version, project.version);
                assert_eq!(display_name, "route");
                assert_eq!(state, ProjectState::Registered);
            }
            _ => panic!("actual service routing result missing"),
        }
    }
    std::fs::create_dir_all(project.worktree_root.join("unregistered")).unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        action: ControlAction::ResolveProject {
            selector: None,
            cwd: project.worktree_root.join("unregistered"),
        },
    };
    assert!(
        runtime.handle_control(&server, request).await.is_err(),
        "unregistered worktree namespace must not route as source"
    );
    assert!(
        owner
            .store()
            .lock()
            .unwrap()
            .goals(project.id)
            .unwrap()
            .is_empty(),
        "routing must not create a Goal/Source/Unit grant"
    );
}

#[tokio::test]
async fn actual_routing_refuses_ambiguity_stale_snapshot_and_body_index_corruption() {
    use crate::domain::Project;
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("state.db");
    let owner = RuntimeOwner::open(&state).unwrap();
    let mut projects = Vec::new();
    for index in 0..2 {
        let root = dir.path().join(format!("source{index}"));
        std::fs::create_dir(&root).unwrap();
        let mut project = Project::new(
            "duplicate".into(),
            root.canonicalize().unwrap(),
            format!("routing-metadata-fixture-{index}"),
            "main".into(),
        );
        owner
            .store()
            .lock()
            .unwrap()
            .put_project(&mut project)
            .unwrap();
        projects.push(project);
    }
    let runtime = Runtime::new(owner.clone(), config()).unwrap();
    let (server, _client) = tokio::net::UnixStream::pair().unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        action: ControlAction::ResolveProject {
            selector: Some("duplicate".into()),
            cwd: projects[0].root.clone(),
        },
    };
    assert!(runtime.handle_control(&server, request).await.is_err());
    let id = projects[0].id.to_string();
    {
        let store = owner.store();
        let mut store = store.lock().unwrap();
        let snapshot = store
            .runtime_project_routes(
                Some(&id),
                &projects[0].root,
                owner.instance_id(),
                owner.epoch(),
            )
            .unwrap();
        projects[0].name = "changed".into();
        store.put_project(&mut projects[0]).unwrap();
        assert!(
            store
                .recheck_runtime_project_routes(
                    Some(&id),
                    &projects[0].root,
                    owner.instance_id(),
                    owner.epoch(),
                    &snapshot
                )
                .is_err()
        );
        assert!(
            store
                .runtime_project_routes(
                    Some(&id),
                    &projects[0].root,
                    owner.instance_id(),
                    owner.epoch() + 1
                )
                .is_err()
        );
    }
    let connection = crate::state::current_test_writer(&state).unwrap();
    connection
        .execute(
            "UPDATE projects SET body=json_set(body,'$.version',version+1) WHERE id=?1",
            [&id],
        )
        .unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        action: ControlAction::ResolveProject {
            selector: Some(id),
            cwd: projects[0].root.clone(),
        },
    };
    assert!(
        runtime.handle_control(&server, request).await.is_err(),
        "routing body/index mismatch cannot be returned as current metadata"
    );
}

// Actual accepted Unix peer ingress; this fixture never constructs Human/Driver authority.
pub(crate) struct ControlFixture {
    pub(super) _dir: tempfile::TempDir,
    pub(super) owner: Arc<RuntimeOwner>,
    pub(super) runtime: Arc<Runtime>,
    pub(super) project: crate::domain::Project,
    pub(super) socket: tokio::net::UnixStream,
    _peer: tokio::net::UnixStream,
}
impl ControlFixture {
    pub(crate) fn store(&self) -> crate::adapter::SharedStore {
        self.owner.store()
    }
    pub(crate) fn state_path(&self) -> std::path::PathBuf {
        self._dir.path().join("state.db")
    }
    fn new() -> Self {
        Self::configured(|_| config())
    }
    pub(super) fn configured(configure: impl FnOnce(&std::path::Path) -> Config) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let root = dir.path().join("source");
        std::fs::create_dir(&root).unwrap();
        let mut project = crate::domain::Project::new(
            "control-fixture".into(),
            root.canonicalize().unwrap(),
            "account-free-no-Git".into(),
            "main".into(),
        );
        owner
            .store()
            .lock()
            .unwrap()
            .put_project(&mut project)
            .unwrap();
        let runtime = Arc::new(Runtime::new(owner.clone(), configure(dir.path())).unwrap());
        let (socket, peer) = tokio::net::UnixStream::pair().unwrap();
        Self {
            _dir: dir,
            owner,
            runtime,
            project,
            socket,
            _peer: peer,
        }
    }
    pub(super) fn register_real_git_project(&mut self) {
        self.register_real_git_project_named("real-git-source");
    }
    pub(super) fn register_real_git_project_named(&mut self, name: &str) {
        let real_root = self._dir.path().join(name);
        std::fs::create_dir(&real_root).unwrap();
        self.project = crate::domain::Project::new(
            "real-source".into(),
            real_root.canonicalize().unwrap(),
            "not-yet-registered".into(),
            "main".into(),
        );
        for args in [
            vec!["init", "-b", "main"],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=.git/hooks",
                "commit",
                "--allow-empty",
                "-m",
                "fixture",
            ],
        ] {
            let output = std::process::Command::new("git")
                .current_dir(&self.project.root)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "isolated Git fixture must be valid: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let valid_before = crate::git::observed_git_outputs();
        self.project.repository_identity =
            crate::git::repository_identity(&self.project.root, "main").unwrap();
        assert!(
            crate::git::observed_git_outputs() > valid_before,
            "valid native repository identity must reach actual helper outputs"
        );
        self.owner
            .store()
            .lock()
            .unwrap()
            .put_project(&mut self.project)
            .unwrap();
    }
    /// A real restart over the same state file (FM §8.2): shutdown, release
    /// every Runtime and owner custody (each awaited, never inferred), then
    /// `RuntimeOwner::open` (a new epoch), a new Runtime with the same config,
    /// and a new Unix-peer pair. The temporary directory is kept.
    pub(super) async fn restart(self) -> Self {
        let config = self.runtime.config.clone();
        let _ = self.runtime.shutdown().await;
        let Self {
            _dir,
            owner,
            runtime,
            project,
            socket,
            _peer,
        } = self;
        drop((socket, _peer));
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        // The installed graph holds the owner, so the owner release below
        // also awaits the graph's custody.
        let released = Arc::downgrade(&runtime);
        drop(runtime);
        while released.upgrade().is_some() {
            assert!(
                tokio::time::Instant::now() < deadline,
                "SETUP: Runtime custody not released"
            );
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let old_epoch = owner.epoch();
        let released = Arc::downgrade(&owner);
        drop(owner);
        while released.upgrade().is_some() {
            assert!(
                tokio::time::Instant::now() < deadline,
                "SETUP: owner custody not released"
            );
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let owner = RuntimeOwner::open(&_dir.path().join("state.db")).unwrap();
        assert!(owner.epoch() > old_epoch, "SETUP: restart has a new epoch");
        let runtime = Arc::new(Runtime::new(owner.clone(), config).unwrap());
        let project = owner
            .store()
            .lock()
            .unwrap()
            .project(project.id)
            .unwrap()
            .unwrap();
        let (socket, peer) = tokio::net::UnixStream::pair().unwrap();
        Self {
            _dir,
            owner,
            runtime,
            project,
            socket,
            _peer: peer,
        }
    }
    pub(super) fn request(&self, action: ControlAction) -> ControlRequest {
        ControlRequest {
            request_id: Uuid::new_v4(),
            instance: self.owner.instance_id().into(),
            epoch: self.owner.epoch(),
            action,
        }
    }
    pub(super) async fn create(&self, plan: GoalPlan) -> crate::domain::GoalId {
        match self
            .runtime
            .handle_control(
                &self.socket,
                self.request(ControlAction::CreateGoal {
                    project: self.project.id,
                    expected_project: self.project.version,
                    plan,
                }),
            )
            .await
            .unwrap()
        {
            ControlResponse::GoalAccepted { goal, .. } => goal,
            _ => panic!("actual accepted Goal required"),
        }
    }
}

/// Sibling controls reuse actual retained Runtime/Unix ingress, not generic Goal
/// writes or a Native issuer. The complete initial plan owns the returned Task.
pub(crate) async fn accepted_goal_fixture() -> (ControlFixture, crate::domain::Task) {
    let fixture = ControlFixture::new();
    let goal_id = fixture.create(plan()).await;
    let task = {
        let shared = fixture.store();
        let store = shared.lock().unwrap();
        let goal = store.goal(goal_id).unwrap().unwrap();
        assert_eq!(goal.project_id, fixture.project.id);
        assert_eq!(goal.dag.nodes.len(), 1);
        let task = store.task(goal.dag.nodes[0]).unwrap().unwrap();
        assert_eq!(task.project_id, goal.project_id);
        assert_eq!(task.goal_id, goal.id);
        assert!(goal.version > 0 && task.version > 0);
        task
    };
    (fixture, task)
}
/// One planned Task of a legacy fixture: its plan key, executor and class.
pub(crate) struct LegacyTask {
    pub(crate) key: &'static str,
    pub(crate) executor: &'static str,
    pub(crate) workflow: WorkflowClass,
    pub(crate) risk: RiskClass,
    /// Planned reviewer agents (configured for the ingress like executors).
    pub(crate) reviewers: &'static [&'static str],
}
impl LegacyTask {
    /// `Task::new`'s historical class (Standard, R1) for a named executor.
    pub(crate) fn standard(key: &'static str, executor: &'static str) -> Self {
        Self {
            key,
            executor,
            workflow: WorkflowClass::Standard,
            risk: RiskClass::R1,
            reviewers: &[],
        }
    }
}

/// FM §8.1 L harness: genuinely legacy rows. The Goal and its Tasks come from
/// accepted Unix-peer ingress; only the historical nongrant tables are copied
/// into the historical schema at `user_version = 2`, and `RuntimeOwner::open`
/// runs the ordered migration (precedent `66f170b`). No authority, scheduler,
/// Driver or execution row is copied, and no guard or writer is changed.
/// The fixture holds no owner: `legacy_fixture` returns the only one, so a
/// test can drop it and reopen the same state.
pub(crate) struct LegacyFixture {
    dir: tempfile::TempDir,
    pub(crate) tasks: Vec<crate::domain::Task>,
}
impl LegacyFixture {
    /// The fixture directory: `repo/` is the Project root, `state.db` the
    /// migrated legacy database.
    pub(crate) fn path(&self) -> &std::path::Path {
        self.dir.path()
    }
    pub(crate) fn task(&self) -> crate::domain::Task {
        self.tasks[0].clone()
    }
}

/// Historical nongrant tables, the only migration input.
const LEGACY_COPIED: [&str; 7] = [
    "projects",
    "goals",
    "tasks",
    "records",
    "context_versions",
    "usage",
    "audit",
];
/// Authority, scheduler, Driver and execution tables that must be empty after
/// the migration (FM-L1). `runtime_epoch` is exempt: the migration itself
/// creates its singleton.
pub(crate) fn legacy_required_empty(connection: &rusqlite::Connection) -> Vec<String> {
    let mut statement = connection
        .prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND (name IN ('goal_authority','scheduler_projects','scheduler_goals','scheduler_tasks','task_drivers','execution_units','session_units','task_execution','source_recoveries') OR name LIKE 'managed\\_%' ESCAPE '\\' OR name LIKE 'native\\_%' ESCAPE '\\') ORDER BY name",
        )
        .unwrap();
    statement
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// Copy the nongrant rows of `source` into the historical schema at `target`
/// and label it `user_version = 2`. The caller opens it to migrate.
pub(crate) fn write_historical_copy(source: &std::path::Path, target: &std::path::Path) {
    let connection = rusqlite::Connection::open(target).unwrap();
    // Build the historical SQL layout, rather than relabelling a current
    // database whose execution tables and writer guards already exist.
    connection
        .execute_batch(include_str!("../state/schema.sql"))
        .unwrap();
    connection
        .execute(
            "ATTACH DATABASE ?1 AS current_fixture",
            [source.to_str().unwrap()],
        )
        .unwrap();
    for table in LEGACY_COPIED {
        connection
            .execute_batch(&format!(
                "INSERT INTO {table} SELECT * FROM current_fixture.{table}"
            ))
            .unwrap();
    }
    connection
        .execute_batch("DETACH DATABASE current_fixture")
        .unwrap();
    connection
        .pragma_update(None, "application_id", crate::state::APPLICATION_ID)
        .unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
}

/// Where a legacy fixture puts its Project root and state database, both
/// directly under the fixture directory.
#[derive(Clone, Copy)]
pub(crate) struct LegacyLayout {
    pub(crate) root: &'static str,
    pub(crate) state: &'static str,
}
impl Default for LegacyLayout {
    fn default() -> Self {
        Self {
            root: "repo",
            state: "state.db",
        }
    }
}

/// FM §8.1 migration step, shared by every L fixture. `owner`/`runtime` are
/// the ingress custody on `<dir>/state.db`; they are released first. The
/// nongrant rows are copied into the historical schema, the ingress files are
/// removed, and `RuntimeOwner::open(<dir>/<state>)` migrates. Post-conditions:
/// the copy source never executed, and every authority/execution table is
/// empty after the migration (FM-L1; `runtime_epoch` exempt).
pub(crate) async fn migrate_legacy(
    dir: &std::path::Path,
    owner: Arc<RuntimeOwner>,
    runtime: Arc<Runtime>,
    state: &str,
) -> Arc<RuntimeOwner> {
    let accepted = dir.join("state.db");
    {
        let c = rusqlite::Connection::open(&accepted).unwrap();
        for table in ["execution_units", "session_units", "task_drivers"] {
            let n: i64 = c
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 0, "SETUP: the copy source executed ({table})");
        }
    }
    let weak = Arc::downgrade(&runtime);
    drop(runtime);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    while weak.upgrade().is_some() || Arc::strong_count(&owner) > 1 {
        assert!(
            tokio::time::Instant::now() < deadline,
            "SETUP: the ingress owner custody did not drop"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    drop(owner);
    let legacy = dir.join("legacy-v2.db");
    write_historical_copy(&accepted, &legacy);
    for leftover in ["state.db", "state.db-wal", "state.db-shm"] {
        let _ = std::fs::remove_file(dir.join(leftover));
    }
    std::fs::remove_dir_all(dir.join("state.db.execution")).unwrap();
    let target = dir.join(state);
    std::fs::rename(&legacy, &target).unwrap();
    let owner = RuntimeOwner::open(&target).unwrap();
    let c = rusqlite::Connection::open(&target).unwrap();
    for table in legacy_required_empty(&c) {
        let n: i64 = c
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "SETUP: legacy migration restored {table}");
    }
    owner
}

/// FM §8.1. `seed` runs on the initialized `repo/` (branch `main`, one empty
/// commit) before the Goal is created; `tasks` become the accepted plan.
pub(crate) async fn legacy_fixture(
    seed: impl FnOnce(&std::path::Path),
    tasks: Vec<LegacyTask>,
) -> (LegacyFixture, Arc<RuntimeOwner>) {
    legacy_fixture_in(LegacyLayout::default(), seed, tasks).await
}

/// `legacy_fixture` for a synchronous caller: the same ingress and migration
/// on a dedicated thread and runtime.
pub(crate) fn legacy_fixture_blocking(
    layout: LegacyLayout,
    seed: impl FnOnce(&std::path::Path) + Send + 'static,
    tasks: Vec<LegacyTask>,
) -> (LegacyFixture, Arc<RuntimeOwner>) {
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(legacy_fixture_in(layout, seed, tasks))
    })
    .join()
    .unwrap()
}

/// Legacy rows behind a plain `Store` (no Runtime owner), for ledger tests
/// that drive `Store` directly; the default layout.
pub(crate) fn legacy_store(tasks: Vec<LegacyTask>) -> (LegacyFixture, crate::state::Store) {
    let (legacy, owner) = legacy_fixture_blocking(LegacyLayout::default(), |_| {}, tasks);
    assert_eq!(Arc::strong_count(&owner), 1, "SETUP: owner still shared");
    drop(owner);
    let store = crate::state::Store::open(&legacy.path().join("state.db")).unwrap();
    (legacy, store)
}

pub(crate) async fn legacy_fixture_in(
    layout: LegacyLayout,
    seed: impl FnOnce(&std::path::Path),
    tasks: Vec<LegacyTask>,
) -> (LegacyFixture, Arc<RuntimeOwner>) {
    let mut config = Config::default();
    let agents = tasks
        .iter()
        .flat_map(|t| std::iter::once(t.executor).chain(t.reviewers.iter().copied()));
    for agent in agents {
        // Ingress-only config: accepted plans admit claude|codex providers.
        // The copied legacy row carries only the executor name, exactly as
        // `Task::new(.., executor)` wrote it (e.g. "grok").
        let provider = match agent {
            "codex" => "codex",
            _ => "claude",
        };
        config.agents.insert(
            agent.into(),
            AgentConfig {
                provider: Some(provider.into()),
                command: vec!["/bin/true".into()],
                ..Default::default()
            },
        );
    }
    // A Quick plan needs minimum Quick (Quick = R0 + minimum Quick).
    config.minimum_workflow = tasks.iter().map(|t| t.workflow).min().unwrap_or_default();
    let mut f = ControlFixture::configured(|_| config);
    f.register_real_git_project_named(layout.root);
    seed(&f.project.root);
    let mut plan = plan();
    plan.tasks = tasks
        .iter()
        .map(|t| TaskDefinition {
            key: t.key.into(),
            title: t.key.into(),
            acceptance_criteria: vec!["verified result".into()],
            executor: t.executor.into(),
            reviewers: t.reviewers.iter().map(|r| (*r).into()).collect(),
            workflow: t.workflow,
            risk: t.risk,
        })
        .collect();
    let goal_id = f.create(plan).await;
    let ControlFixture {
        _dir: dir,
        owner,
        runtime,
        socket,
        _peer,
        ..
    } = f;
    drop((socket, _peer));
    let owner = migrate_legacy(dir.path(), owner, runtime, layout.state).await;
    let tasks = {
        let store = owner.store.lock().unwrap();
        let goal = store.goal(goal_id).unwrap().unwrap();
        let tasks = goal
            .dag
            .nodes
            .iter()
            .map(|id| store.task(*id).unwrap().unwrap())
            .collect::<Vec<_>>();
        assert!(
            tasks.iter().all(|t| t.worktree.is_none()),
            "SETUP: no LegacyUnreconciled Unit source"
        );
        tasks
    };
    (LegacyFixture { dir, tasks }, owner)
}

#[tokio::test]
async fn actual_accepted_goal_transaction_idempotency_and_generic_writers() {
    let f = ControlFixture::new();
    let request = f.request(ControlAction::CreateGoal {
        project: f.project.id,
        expected_project: f.project.version,
        plan: plan(),
    });
    let response = f
        .runtime
        .handle_control(&f.socket, request.clone())
        .await
        .unwrap();
    let ControlResponse::GoalAccepted {
        goal,
        version,
        task_count,
    } = response
    else {
        panic!("accepted result")
    };
    assert_eq!((version, task_count), (1, 1));
    let mut original = f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap();
    let mut task = f
        .owner
        .store()
        .lock()
        .unwrap()
        .task(original.dag.nodes[0])
        .unwrap()
        .unwrap();
    let events = f
        .owner
        .store()
        .lock()
        .unwrap()
        .events(&original.scope(), 0, 100)
        .unwrap()
        .len();
    assert!(
        matches!(f.runtime.handle_control(&f.socket,request.clone()).await.unwrap(),ControlResponse::GoalAccepted{goal:id,..} if id==goal)
    );
    let mut reused = request.clone();
    if let ControlAction::CreateGoal { plan, .. } = &mut reused.action {
        plan.definition.title = "changed".into();
    }
    assert!(f.runtime.handle_control(&f.socket, reused).await.is_err());
    {
        let shared = f.owner.store();
        let mut store = shared.lock().unwrap();
        assert_eq!(store.goals(f.project.id).unwrap().len(), 1);
        store.put_goal(&mut original).unwrap();
        store.put_task(&mut task).unwrap();
        assert_eq!(
            store.events(&original.scope(), 0, 100).unwrap().len(),
            events
        );
        original.objective = "arbitrary JSON authority".into();
        assert!(store.put_goal(&mut original).is_err());
        task.title = "arbitrary managed Task write".into();
        assert!(store.put_task(&mut task).is_err());
        let mut new = crate::domain::Goal::new(f.project.id, "untrusted proposal".into(), vec![]);
        assert!(store.put_goal(&mut new).is_err());
        assert_eq!(store.goals(f.project.id).unwrap().len(), 1);
    }
}
#[tokio::test]
async fn actual_accepted_goal_pages_scope_complete_inventory_and_native_hold() {
    let f = ControlFixture::new();
    let mut many = plan();
    for i in 1..129 {
        let mut t = many.tasks[0].clone();
        t.key = format!("task-{i}");
        many.tasks.push(t);
    }
    let goal = f.create(many).await;
    let response = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::GoalTasks {
                view: None,
                project: f.project.id,
                goal,
                after: None,
                maximum: 128,
            }),
        )
        .await
        .unwrap();
    assert!(serde_json::to_vec(&response).unwrap().len() <= 65536);
    let ControlResponse::GoalTaskPage {
        tasks,
        next: Some(next),
        ..
    } = response
    else {
        panic!("first bounded page")
    };
    assert_eq!(tasks.len(), 128);
    assert!(
        tasks
            .windows(2)
            .all(|p| p[0].scope.task_id < p[1].scope.task_id)
    );
    let last = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::GoalTasks {
                view: None,
                project: f.project.id,
                goal,
                after: Some(next),
                maximum: 128,
            }),
        )
        .await
        .unwrap();
    assert!(matches!(last,ControlResponse::GoalTaskPage{tasks,next:None,..} if tasks.len()==1));
    for (after, maximum) in [
        (Some(crate::domain::TaskId::new()), 1),
        (None, 0),
        (None, 129),
    ] {
        assert!(
            f.runtime
                .handle_control(
                    &f.socket,
                    f.request(ControlAction::GoalTasks {
                        view: None,
                        project: f.project.id,
                        goal,
                        after,
                        maximum
                    })
                )
                .await
                .is_err()
        );
    }
    let task = tasks[0].scope.task_id.unwrap();
    let sources =
        crate::execution::workflow_source::ManagedWorkflowSources::new(f.owner.clone(), config())
            .unwrap();
    let error = sources.prepare(task, "claude").await.unwrap_err();
    // The accepted controller graph must not escape to registered Git without a real Driver.
    assert!(
        format!("{error:#}").contains("Query returned no rows")
            || format!("{error:#}").contains("Driver"),
        "{error:#}"
    );
    let connection = crate::state::current_test_writer(f.owner.state_path()).unwrap();
    for table in [
        "execution_units",
        "managed_effects",
        "task_drivers",
        "native_invocations",
    ] {
        let count: u64 = connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "no actual {table} producer may be invented");
    }
    assert!(matches!(
        f.runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::GoalStatus {
                    view: None,
                    project: f.project.id,
                    goal
                })
            )
            .await
            .unwrap(),
        ControlResponse::GoalFacts {
            dispatch_available: false,
            attention: UnavailableReason::NativeBindingUnavailable,
            ..
        }
    ));
}
#[tokio::test]
async fn actual_goal_lifecycle_cas_policy_fence_and_never_prepared_resume() {
    let f = ControlFixture::new();
    let goal = f.create(plan()).await;
    let paused = f.request(ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal,
        expected_goal: 1,
        target: GoalControl::Pause,
        reason: "explicit lifecycle hold".into(),
    });
    assert!(matches!(
        f.runtime
            .handle_control(&f.socket, paused.clone())
            .await
            .unwrap(),
        ControlResponse::GoalLifecycleChanged {
            version: 2,
            state: crate::domain::GoalState::Paused,
            ..
        }
    ));
    assert!(matches!(
        f.runtime.handle_control(&f.socket, paused).await.unwrap(),
        ControlResponse::GoalLifecycleChanged { version: 2, .. }
    ));
    let stale = f.request(ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal,
        expected_goal: 1,
        target: GoalControl::Resume,
        reason: "stale".into(),
    });
    assert!(f.runtime.handle_control(&f.socket, stale).await.is_err());
    let resumed = f.request(ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal,
        expected_goal: 2,
        target: GoalControl::Resume,
        reason: "no Unit has ever been prepared".into(),
    });
    assert!(matches!(
        f.runtime.handle_control(&f.socket, resumed).await.unwrap(),
        ControlResponse::GoalLifecycleChanged {
            version: 3,
            state: crate::domain::GoalState::Running,
            ..
        }
    ));
    let cancel = f.request(ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal,
        expected_goal: 3,
        target: GoalControl::Cancel,
        reason: "trusted cancel".into(),
    });
    assert!(matches!(
        f.runtime.handle_control(&f.socket, cancel).await.unwrap(),
        ControlResponse::GoalLifecycleChanged {
            version: 4,
            state: crate::domain::GoalState::Cancelled,
            ..
        }
    ));
    assert!(
        f.runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::SetGoalLifecycle {
                    project: f.project.id,
                    goal,
                    expected_goal: 4,
                    target: GoalControl::Resume,
                    reason: "terminal cannot reactivate".into()
                })
            )
            .await
            .is_err()
    );
}
#[tokio::test]
async fn actual_control_loop_persists_named_hold_without_authority_bumps() {
    let f = ControlFixture::new();
    let goal = f.create(plan()).await;
    let before = f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap();
    let task = f
        .owner
        .store()
        .lock()
        .unwrap()
        .task(before.dag.nodes[0])
        .unwrap()
        .unwrap();
    f.runtime.start().await.unwrap();
    assert!(f.runtime.start().await.is_err());
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let connection = crate::state::current_test_writer(f.owner.state_path()).unwrap();
            let attention: Option<String> = connection
                .query_row(
                    "SELECT attention FROM scheduler_tasks WHERE task_id=?1",
                    [task.id.to_string()],
                    |r| r.get(0),
                )
                .unwrap();
            if attention.is_some() {
                assert!(attention.unwrap().contains("native_binding_unavailable"));
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&task).unwrap(),
        serde_json::to_value(
            f.owner
                .store()
                .lock()
                .unwrap()
                .task(task.id)
                .unwrap()
                .unwrap()
        )
        .unwrap()
    );
    assert!(matches!(
        f.runtime
            .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
            .await
            .unwrap(),
        ControlResponse::RuntimeMetadata {
            operational: false,
            service_running: true,
            ..
        }
    ));
    f.runtime.shutdown().await.unwrap();
    assert!(matches!(
        f.runtime
            .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
            .await
            .unwrap(),
        ControlResponse::RuntimeMetadata {
            operational: false,
            service_running: false,
            ..
        }
    ));
    assert!(
        f.runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::CreateGoal {
                    project: f.project.id,
                    expected_project: f.project.version,
                    plan: plan()
                })
            )
            .await
            .is_err()
    );
}

#[tokio::test]
async fn actual_indexed_context_history_cannot_resume_by_malformed_body_scope() {
    let f = ControlFixture::new();
    for body in [
        serde_json::json!({}),
        serde_json::json!({"scope":{"goal_id":crate::domain::GoalId::new()}}),
        serde_json::json!({"valid_scope_placeholder":true}),
    ] {
        let goal = f.create(plan()).await;
        f.runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::SetGoalLifecycle {
                    project: f.project.id,
                    goal,
                    expected_goal: 1,
                    target: GoalControl::Pause,
                    reason: "history hold".into(),
                }),
            )
            .await
            .unwrap();
        let original = f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap();
        let events = f
            .owner
            .store()
            .lock()
            .unwrap()
            .events(&original.scope(), 0, 100)
            .unwrap()
            .len();
        let connection = crate::state::current_test_writer(f.owner.state_path()).unwrap();
        let body = if body.get("valid_scope_placeholder").is_some() {
            serde_json::json!({"scope":original.scope(),"version":1})
        } else {
            body
        };
        connection.execute("INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) VALUES(?1,?2,NULL,?3,1,?4)",rusqlite::params![f.project.id.to_string(),goal.to_string(),goal.to_string(),body.to_string()]).unwrap();
        let request = f.request(ControlAction::SetGoalLifecycle {
            project: f.project.id,
            goal,
            expected_goal: 2,
            target: GoalControl::Resume,
            reason: "indexed history must remain held".into(),
        });
        let request_id = request.request_id;
        assert!(matches!(
            f.runtime.handle_control(&f.socket, request).await.unwrap(),
            ControlResponse::Unavailable {
                reason: UnavailableReason::FreshBootstrapRecoveryUnavailable,
                ..
            }
        ));
        assert_eq!(
            serde_json::to_value(&original).unwrap(),
            serde_json::to_value(f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap())
                .unwrap()
        );
        assert_eq!(
            f.owner
                .store()
                .lock()
                .unwrap()
                .events(&original.scope(), 0, 100)
                .unwrap()
                .len(),
            events
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM runtime_control_acks WHERE request_id=?1",
                    [request_id.to_string()],
                    |r| r.get::<_, u64>(0)
                )
                .unwrap(),
            0
        );
    }
}
#[tokio::test]
async fn actual_goal_acceptance_and_stop_share_publication_linearization() {
    use std::{future::Future, task::Poll};
    let f = ControlFixture::new();
    let (reached_rx_sender, reached) = tokio::sync::oneshot::channel();
    let (release, release_rx) = tokio::sync::oneshot::channel();
    *f.runtime.goal_admission_pause.lock().unwrap() = Some(GoalAdmissionPause {
        reached: reached_rx_sender,
        release: release_rx,
    });
    let runtime = f.runtime.clone();
    let request = f.request(ControlAction::CreateGoal {
        project: f.project.id,
        expected_project: f.project.version,
        plan: plan(),
    });
    let (server, _peer) = tokio::net::UnixStream::pair().unwrap();
    let creating = tokio::spawn(async move { runtime.handle_control(&server, request).await });
    reached.await.unwrap();
    let stopping = f
        .runtime
        .handle_control(&f.socket, f.request(ControlAction::RuntimeStop));
    tokio::pin!(stopping);
    std::future::poll_fn(|cx| {
        assert!(
            matches!(stopping.as_mut().poll(cx), Poll::Pending),
            "Stop acknowledged before its admitted Goal publication finished"
        );
        Poll::Ready(())
    })
    .await;
    release.send(()).unwrap();
    assert!(matches!(
        creating.await.unwrap().unwrap(),
        ControlResponse::GoalAccepted { .. }
    ));
    assert!(matches!(
        stopping.await.unwrap(),
        ControlResponse::RuntimeStopped { .. }
    ));
    assert_eq!(
        f.owner
            .store()
            .lock()
            .unwrap()
            .goals(f.project.id)
            .unwrap()
            .len(),
        1
    );
    assert!(
        f.runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::CreateGoal {
                    project: f.project.id,
                    expected_project: f.project.version,
                    plan: plan()
                })
            )
            .await
            .is_err()
    );
    assert_eq!(
        f.owner
            .store()
            .lock()
            .unwrap()
            .goals(f.project.id)
            .unwrap()
            .len(),
        1
    );
}
#[tokio::test]
async fn actual_accepted_worktree_routes_refuse_before_valid_native_git_helpers() {
    let mut f = ControlFixture::new();
    f.register_real_git_project();
    let goal = f.create(plan()).await;
    let g = f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap();
    let t = f
        .owner
        .store()
        .lock()
        .unwrap()
        .task(g.dag.nodes[0])
        .unwrap()
        .unwrap();
    let outputs = crate::git::observed_git_outputs();
    let before = f
        .owner
        .store()
        .lock()
        .unwrap()
        .events(&t.scope(), 0, 100)
        .unwrap()
        .len();
    {
        let shared = f.owner.store();
        let mut store = shared.lock().unwrap();
        let error = crate::git::WorktreeManager::create(&mut store, t.id).unwrap_err();
        assert!(format!("{error:#}").contains("managed Driver/binding"));
        assert!(crate::git::WorktreeManager::status(&store, t.id).is_err());
        assert!(crate::git::WorktreeManager::ensure_mutation_allowed(&store, t.id).is_err());
        assert!(
            crate::git::WorktreeManager::lock_review(
                &mut store,
                t.id,
                &"a".repeat(40),
                "held review"
            )
            .is_err()
        );
        assert!(crate::git::WorktreeManager::cleanup(&mut store, t.id).is_err());
        assert_eq!(
            crate::git::observed_git_outputs(),
            outputs,
            "accepted public worktree route reached native Git helper"
        );
        assert_eq!(
            serde_json::to_value(&t).unwrap(),
            serde_json::to_value(store.task(t.id).unwrap().unwrap()).unwrap()
        );
        assert_eq!(store.events(&t.scope(), 0, 100).unwrap().len(), before);
        assert!(
            store
                .records(&t.scope(), crate::domain::RecordKind::WorktreeLock)
                .unwrap()
                .is_empty()
        );
        assert!(store.execution_units(Some(&t.scope())).unwrap().is_empty());
    }
    assert!(!f.project.worktree_root.exists());
}

#[tokio::test]
async fn actual_control_loop_error_retires_join_result_before_repeated_shutdown() {
    let f = ControlFixture::new();
    f.runtime.start().await.unwrap();
    // Actual epoch revocation; neither this row nor the loop constructs a Driver.
    f.owner
        .store()
        .lock()
        .unwrap()
        .begin_execution_epoch()
        .unwrap();
    f.runtime.wake.notify_one();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let finished = f
                .runtime
                .supervisor
                .lock()
                .await
                .as_ref()
                .is_some_and(|h| h.is_finished());
            if finished {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let error = f.runtime.shutdown().await.unwrap_err();
    assert!(format!("{error:#}").contains("owner retired"));
    assert!(
        f.runtime.supervisor.lock().await.is_none(),
        "completed Err must not leave an already-polled handle"
    );
    assert!(!f.runtime.service_running());
    f.runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn actual_worktree_snapshot_identity_precedes_legacy_scope_and_git() {
    use crate::domain::{Goal, GoalState, Task, TaskId};
    let mut f = ControlFixture::new();
    f.register_real_git_project();
    let accepted = f.create(plan()).await;
    let accepted_goal = f
        .owner
        .store()
        .lock()
        .unwrap()
        .goal(accepted)
        .unwrap()
        .unwrap();
    let accepted_task = f
        .owner
        .store()
        .lock()
        .unwrap()
        .task(accepted_goal.dag.nodes[0])
        .unwrap()
        .unwrap();
    let connection = crate::state::current_test_writer(f.owner.state_path()).unwrap();
    // Historical unaccepted fixture content, not a new production ingress or
    // a managed owner. Public Goal creation remains refused. Only the public
    // legacy worktree route is exercised positively; no Driver is installed.
    let mut legacy_goal = Goal::new(f.project.id, "historical legacy helper".into(), vec![]);
    legacy_goal.version = 1;
    legacy_goal.state = GoalState::Running;
    connection
        .execute(
            "INSERT INTO goals(id,project_id,version,body) VALUES(?1,?2,?3,?4)",
            rusqlite::params![
                legacy_goal.id.to_string(),
                f.project.id.to_string(),
                legacy_goal.version,
                serde_json::to_string(&legacy_goal).unwrap()
            ],
        )
        .unwrap();
    let mut legacy = Task::new(
        f.project.id,
        legacy_goal.id,
        "legacy negative".into(),
        "worker".into(),
    );
    let mut positive = Task::new(
        f.project.id,
        legacy_goal.id,
        "canonical legacy positive".into(),
        "worker".into(),
    );
    {
        let shared = f.owner.store();
        let mut store = shared.lock().unwrap();
        store.put_task(&mut legacy).unwrap();
        store.put_task(&mut positive).unwrap();
        let before = crate::git::observed_git_outputs();
        let status = crate::git::WorktreeManager::create(&mut store, positive.id).unwrap();
        assert!(status.worktree.is_dir());
        assert!(
            crate::git::observed_git_outputs() > before,
            "canonical legacy route must reach actual Git helpers"
        );
    }
    let mut wrong_id = legacy.clone();
    wrong_id.id = TaskId::new();
    let mut wrong_project = legacy.clone();
    wrong_project.project_id = crate::domain::ProjectId::new();
    let mut wrong_goal = legacy.clone();
    wrong_goal.goal_id = accepted;
    let mut wrong_version = legacy.clone();
    wrong_version.version += 1;
    let mut oversized = legacy.clone();
    oversized.title = "x".repeat(1024 * 1024);
    let bodies = [
        serde_json::to_string(&accepted_task).unwrap(),
        serde_json::to_string(&wrong_id).unwrap(),
        serde_json::to_string(&wrong_project).unwrap(),
        serde_json::to_string(&wrong_goal).unwrap(),
        serde_json::to_string(&wrong_version).unwrap(),
        "{}".into(),
        serde_json::to_string(&oversized).unwrap(),
    ];
    for body in bodies {
        connection
            .execute(
                "UPDATE tasks SET body=?1 WHERE id=?2",
                rusqlite::params![body, legacy.id.to_string()],
            )
            .unwrap();
        let before = raw_control_fixture_rows(&connection);
        let outputs = crate::git::observed_git_outputs();
        {
            let shared = f.owner.store();
            let mut store = shared.lock().unwrap();
            assert!(crate::git::WorktreeManager::create(&mut store, legacy.id).is_err());
            assert!(crate::git::WorktreeManager::status(&store, legacy.id).is_err());
            assert!(
                crate::git::WorktreeManager::ensure_mutation_allowed(&store, legacy.id).is_err()
            );
            assert!(
                crate::git::WorktreeManager::lock_review(
                    &mut store,
                    legacy.id,
                    &"a".repeat(40),
                    "corrupt Task"
                )
                .is_err()
            );
            assert!(crate::git::WorktreeManager::cleanup(&mut store, legacy.id).is_err());
        }
        assert_eq!(
            crate::git::observed_git_outputs(),
            outputs,
            "malformed Task body/index reached actual Git helpers"
        );
        assert_eq!(
            raw_control_fixture_rows(&connection),
            before,
            "malformed Task helper wrote authority or audit"
        );
        assert!(
            !f.project
                .worktree_root
                .join(format!("task-{}", legacy.id))
                .exists()
        );
        assert!(
            !f.project
                .worktree_root
                .join(format!("task-{}", accepted_task.id))
                .exists()
        );
    }
    let before = crate::git::observed_git_outputs();
    let shared = f.owner.store();
    let mut store = shared.lock().unwrap();
    assert!(crate::git::WorktreeManager::create(&mut store, TaskId::new()).is_err());
    assert_eq!(crate::git::observed_git_outputs(), before);
}

// Entire small isolated fixture DB, including native/resource/audit tables.
// Comparing rows (not only the main file) includes any WAL-visible writes.
fn raw_control_fixture_rows(connection: &rusqlite::Connection) -> Vec<(String, Vec<Vec<String>>)> {
    let mut names = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap();
    names
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(|name| {
            let name = name.unwrap();
            let mut rows = connection
                .prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))
                .unwrap();
            let count = rows.column_count();
            let mut contents = rows
                .query_map([], |row| {
                    (0..count)
                        .map(|i| row.get_ref(i).map(|v| format!("{v:?}")))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .map(|r| r.unwrap())
                .collect::<Vec<_>>();
            contents.sort();
            (name, contents)
        })
        .collect()
}

#[tokio::test]
async fn public_source_preparation_refuses_accepted_goal_before_unit_or_helper_effects() {
    let mut f = ControlFixture::new();
    f.register_real_git_project();
    let goal = f.create(plan()).await;
    let task = {
        let shared = f.owner.store();
        let store = shared.lock().unwrap();
        let goal = store.goal(goal).unwrap().unwrap();
        store.task(goal.dag.nodes[0]).unwrap().unwrap()
    };
    let before = {
        let shared = f.owner.store();
        let store = shared.lock().unwrap();
        store.events(&task.scope(), 0, 100).unwrap().len()
    };
    let outputs = crate::git::observed_git_outputs();
    let manager = crate::execution::attempts::AttemptManager::new(f.owner.clone());
    let error = manager
        .prepare(task.id, "codex", "implement", None)
        .await
        .unwrap_err();
    assert!(format!("{error:#}").contains("managed Driver/binding"));
    let sources =
        crate::execution::workflow_source::ManagedWorkflowSources::new(f.owner.clone(), config())
            .unwrap();
    let error = sources.prepare(task.id, "codex").await.unwrap_err();
    assert!(format!("{error:#}").contains("managed Driver/binding"));
    let shared = f.owner.store();
    let store = shared.lock().unwrap();
    assert_eq!(
        serde_json::to_value(&task).unwrap(),
        serde_json::to_value(store.task(task.id).unwrap().unwrap()).unwrap()
    );
    assert_eq!(store.events(&task.scope(), 0, 100).unwrap().len(), before);
    assert!(
        store
            .execution_units(Some(&task.scope()))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        crate::git::observed_git_outputs(),
        outputs,
        "public preparation reached actual Git helper"
    );
    assert!(!f.project.worktree_root.exists());
}
