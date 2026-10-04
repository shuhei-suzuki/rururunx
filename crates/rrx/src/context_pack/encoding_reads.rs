//! Owned synthetic codec fixtures. These prove artifact readers, not native
//! process containment, prepared-input admission or settlement authority.
use super::*;
use crate::{
    git::WorktreeManager,
    state::Store,
    workflow::{BudgetClass, ContextBudget, Phase, WorkflowSources},
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

struct Fixture {
    _temp: tempfile::TempDir,
    db: PathBuf,
    store: SharedStore,
    project: Project,
    goal: Goal,
    task: Task,
}
fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().into()
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("repo");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        for (key, value) in [
            ("user.name", "Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", ".git/hooks"),
        ] {
            git(&root, &["config", key, value]);
        }
        std::fs::write(root.join(".gitignore"), "worktree/\n").unwrap();
        std::fs::write(root.join("RULES.md"), "Keep exact Project scope.\n").unwrap();
        std::fs::write(root.join("lib.rs"), "pub fn run() {}\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-m", "owned fixture"]);
        let db = temp.path().join("state.db");
        let mut store = Store::open(&db).unwrap();
        let mut project = Project::new(
            "fixture".into(),
            root.clone(),
            crate::git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        project.rule_refs = vec![root.join("RULES.md")];
        store.put_project(&mut project).unwrap();
        let mut goal = Goal::new(project.id, "Preserve mandatory facts".into(), vec![]);
        goal.state = GoalState::Running;
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(
            project.id,
            goal.id,
            "Codec fixture".into(),
            "fixture".into(),
        );
        store.put_task(&mut task).unwrap();
        WorktreeManager::create(&mut store, task.id).unwrap();
        task = store.task(task.id).unwrap().unwrap();
        Self {
            _temp: temp,
            db,
            store: Arc::new(Mutex::new(store)),
            project,
            goal,
            task,
        }
    }
    fn packs(&self) -> ContextPacks {
        ContextPacks::new(self.store.clone())
    }
    async fn task_context(&self) -> (ContextVersion, PackRef) {
        let packs = self.packs();
        let draft = packs
            .draft_task(&self.task.scope(), TaskInputs::default())
            .await
            .unwrap();
        let r = packs.publish_task(&draft).await.unwrap();
        let c = self
            .store
            .lock()
            .unwrap()
            .context(&r.scope, Some(r.version))
            .unwrap()
            .unwrap();
        packs.task_pack(&r).unwrap();
        (c, r)
    }
    // Corrupt ONLY an owned negative fixture; never a publication/native proof.
    fn corrupt(&self, context: &ContextVersion) -> PackRef {
        let body = serde_json::to_string(context).unwrap();
        assert!(
            body.len() < 8 * MAX_BYTES,
            "row cap must not mask reader limit"
        );
        let owner = context
            .scope
            .task_id
            .map(|id| format!("task:{id}"))
            .unwrap_or_else(|| format!("goal:{}", context.scope.goal_id.unwrap()));
        rusqlite::Connection::open(&self.db).unwrap().execute(
            "UPDATE context_versions SET body=?1 WHERE project_id=?2 AND owner=?3 AND version=?4",
            rusqlite::params![body, context.scope.project_id.to_string(), owner, context.version],
        ).unwrap();
        reference(context).unwrap()
    }
}
fn nested(depth: usize) -> Value {
    (0..depth).fold(Value::Null, |child, _| json!([child]))
}
fn reset() {
    encoding::take_read_stages();
}
fn refused(error: anyhow::Error, message: &str) {
    assert_eq!(error.root_cause().to_string(), message);
    assert_eq!(
        encoding::take_read_stages(),
        0,
        "guard must precede digest/clone/decode"
    );
}
const BYTES: &str = "mandatory pack/checkpoint exceeds 1 MiB; narrow explicitly";
const DEPTH: &str = "artifact JSON nesting exceeds 120 containers";

#[tokio::test]
async fn typed_task_and_provenance_readers_guard_before_digest_and_decode() {
    let f = Fixture::new();
    let (control, _) = f.task_context().await;
    for (field, value, message) in [
        ("decisions", json!(["x".repeat(MAX_BYTES)]), BYTES),
        ("task", nested(120), DEPTH),
    ] {
        let mut c = control.clone();
        c.data[field] = value;
        let r = f.corrupt(&c);
        for intent in [ContextRead::TypedTask, ContextRead::TaskProvenance] {
            reset();
            refused(f.packs().load_context(&r, intent).unwrap_err(), message);
        }
        reset();
        refused(f.packs().task_pack(&r).unwrap_err(), message);
    }
    let mut c = control;
    c.data["format"] = json!("unknown.format");
    c.data["decisions"] = json!(["x".repeat(MAX_BYTES)]);
    let r = f.corrupt(&c);
    reset();
    refused(f.packs().task_pack(&r).unwrap_err(), BYTES);
    let opaque = f
        .packs()
        .load_context(&r, ContextRead::TaskProvenance)
        .unwrap();
    assert_eq!(opaque.data, c.data);
}

#[tokio::test]
async fn typed_goal_reader_guards_before_digest_and_decode() {
    let f = Fixture::new();
    let r = f
        .packs()
        .publish_goal(&f.goal.scope(), vec![], Default::default())
        .await
        .unwrap();
    let control = f
        .store
        .lock()
        .unwrap()
        .context(&r.scope, Some(r.version))
        .unwrap()
        .unwrap();
    f.packs().goal_pack(&r).unwrap();
    for (field, value, message) in [
        (
            "cross_task_decisions",
            json!(["x".repeat(MAX_BYTES)]),
            BYTES,
        ),
        ("goal", nested(120), DEPTH),
    ] {
        let mut c = control.clone();
        c.data[field] = value;
        let r = f.corrupt(&c);
        reset();
        refused(f.packs().goal_pack(&r).unwrap_err(), message);
    }
    let mut c = control;
    c.data["format"] = json!("unknown.format");
    c.data["cross_task_decisions"] = json!(["x".repeat(MAX_BYTES)]);
    let r = f.corrupt(&c);
    reset();
    refused(f.packs().goal_pack(&r).unwrap_err(), BYTES);
}

#[tokio::test]
async fn opaque_goal_format_in_task_scope_keeps_its_provenance_path() {
    let f = Fixture::new();
    let (mut c, _) = f.task_context().await;
    c.data = json!({"format":GOAL_FORMAT,"opaque":"x".repeat(MAX_BYTES)});
    let r = f.corrupt(&c); // Negative corruption stands for historical opaque shape.
    reset();
    f.packs()
        .load_context(&r, ContextRead::TaskProvenance)
        .unwrap();
    assert_eq!(encoding::take_read_stages(), encoding::CONTEXT_DIGEST);
    let t = f.store.lock().unwrap().task(f.task.id).unwrap().unwrap();
    f.packs().validate_task_reference(&t, &r).await.unwrap();
}

#[tokio::test]
async fn phase_reader_and_capture_guard_before_artifact_clone() {
    let f = Fixture::new();
    let sources = workflow::WorkflowPackSources::new(f.packs());
    let budget = ContextBudget {
        class: BudgetClass::Normal,
        discretionary_tokens: 128,
    };
    let source = sources
        .capture(
            f.project.clone(),
            f.task.clone(),
            Phase::Implement,
            budget.clone(),
        )
        .await
        .unwrap();
    let artifact = sources.pack_artifact(&source).unwrap().unwrap();
    workflow::validate_capture(&artifact, &source, Phase::Implement, &budget).unwrap();
    let control = ContextVersion {
        scope: source.scope.clone(),
        version: 1,
        revision: source.revision.clone(),
        source_hashes: source.source_versions.clone(),
        data: json!({"task_pack":artifact,"phase":Phase::Implement,"budget":budget,
            "payload":source.payload,"source_payload_offset":0}),
    };
    workflow::context_artifact(&control).unwrap();
    let mut c = control.clone();
    c.data["task_pack"]["source_versions"]["negative:padding"] = json!("x".repeat(MAX_BYTES));
    c.source_hashes
        .insert("negative:padding".into(), "x".repeat(MAX_BYTES));
    let mut altered_source = source.clone();
    altered_source.source_versions = c.source_hashes.clone();
    assert!(serde_json::to_vec(&c).unwrap().len() < 8 * MAX_BYTES);
    reset();
    refused(workflow::context_artifact(&c).unwrap_err(), BYTES);
    reset();
    refused(
        workflow::validate_capture(
            &c.data["task_pack"],
            &altered_source,
            Phase::Implement,
            &budget,
        )
        .unwrap_err(),
        BYTES,
    );
    c = control;
    c.data["task_pack"]["pack"]["task"] = nested(119); // artifact root + pack +119 =121
    reset();
    refused(workflow::context_artifact(&c).unwrap_err(), DEPTH);
    reset();
    refused(
        workflow::validate_capture(&c.data["task_pack"], &source, Phase::Implement, &budget)
            .unwrap_err(),
        DEPTH,
    );
}

#[test]
fn maximum_depth_general_artifact_round_trips_through_actual_store_codec() {
    let f = Fixture::new();
    let data = nested(120);
    bounded(&data).unwrap();
    let context = ContextVersion {
        scope: f.goal.scope(),
        version: 1,
        revision: "fixture".into(),
        source_hashes: Default::default(),
        data,
    };
    f.store.lock().unwrap().put_context(&context).unwrap();
    let loaded = f
        .store
        .lock()
        .unwrap()
        .context(&context.scope, Some(1))
        .unwrap()
        .unwrap();
    assert_eq!(loaded.data, context.data);
    // Nonlaunchable factual Record has the same +2 JSON embedding as
    // ContextVersion.data.task_pack, without inventing a phase publication.
    let mut record = Record::new(
        f.goal.scope(),
        RecordKind::Verification,
        json!({"task_pack":nested(120)}),
    );
    bounded(&record.data["task_pack"]).unwrap();
    f.store.lock().unwrap().put_record(&mut record).unwrap();
    let loaded = f.store.lock().unwrap().record(record.id).unwrap().unwrap();
    assert_eq!(loaded.data, record.data);
}
