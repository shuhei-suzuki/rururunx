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
        let mut goal = Goal::new(
            project.id,
            "Preserve mandatory facts".into(),
            vec![CompletionCriterion {
                id: "codec".into(),
                description: "exact artifact boundaries verified".into(),
                evidence: None,
                satisfied: false,
            }],
        );
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
        let pack = packs.task_pack(&r).unwrap();
        assert_eq!(
            encoding::encoded_len(&pack).unwrap(),
            serde_json::to_vec(&pack).unwrap().len()
        );
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
        let mut connection = rusqlite::Connection::open(&self.db).unwrap();
        let trigger: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='trigger' AND name='context_no_update'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let tx = connection.transaction().unwrap();
        // Deliberate negative corruption, confined to this fixture DB. Restore
        // the actual trigger in the same transaction; no production writer is
        // allowed to mutate an immutable context.
        tx.execute_batch("DROP TRIGGER context_no_update").unwrap();
        let changed = tx.execute(
            "UPDATE context_versions SET body=?1 WHERE project_id=?2 AND owner=?3 AND version=?4",
            rusqlite::params![
                body,
                context.scope.project_id.to_string(),
                owner,
                context.version
            ],
        )
        .unwrap();
        assert_eq!(changed, 1, "negative fixture must change exactly its row");
        tx.execute_batch(&trigger).unwrap();
        tx.commit().unwrap();
        reference(context).unwrap()
    }
    fn factual_session(&self) -> SessionId {
        // Initial terminal history is factual component input only. It proves
        // neither model execution nor native cleanup/managed ownership.
        let session = Session {
            id: SessionId::new(),
            scope: self.task.scope(),
            agent: "codec-fixture".into(),
            provider: "codec-fixture".into(),
            role: SessionRole::Consultant,
            native_ref: None,
            pid: None,
            worktree: self.task.worktree.clone().unwrap(),
            state: SessionState::Exited,
            model: None,
            effort: None,
            recovery: Value::Null,
            started_at: now_ms(),
        };
        self.store.lock().unwrap().put_session(&session, 0).unwrap();
        session.id
    }
    fn corrupt_checkpoint(&self, record: &Record) -> CheckpointRef {
        let body = serde_json::to_string(record).unwrap();
        assert!(body.len() < 8 * MAX_BYTES);
        // The SQL mutation is an owned negative fixture, never an append proof.
        let changed = rusqlite::Connection::open(&self.db)
            .unwrap()
            .execute(
                "UPDATE records SET body=?1 WHERE id=?2",
                rusqlite::params![body, record.id.to_string()],
            )
            .unwrap();
        assert_eq!(changed, 1, "negative fixture must change exactly its row");
        CheckpointRef {
            scope: record.scope.clone(),
            id: record.id,
            version: record.version,
            digest: digest(&record.data).unwrap(),
        }
    }
}
fn nested(depth: usize) -> Value {
    (0..depth).fold(Value::Null, |child, _| json!([child]))
}
fn reset() {
    encoding::take_read_stages();
}
trait FixtureResult<T> {
    fn bounded_err(self) -> anyhow::Error;
}
impl<T> FixtureResult<T> for Result<T> {
    fn bounded_err(self) -> anyhow::Error {
        match self {
            Err(error) => error,
            Ok(_) => panic!("expected bounded consumer refusal"),
        }
    }
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
async fn checkpoint_reader_guards_after_identity_before_digest_and_decode() {
    let f = Fixture::new();
    let session = f.factual_session();
    let reference = f
        .packs()
        .checkpoint(
            &f.task.scope(),
            session,
            None,
            vec![
                HistoryEvent {
                    sequence: 1,
                    kind: EventKind::Transient,
                    text: "owned history".repeat(100),
                },
                HistoryEvent {
                    sequence: 2,
                    kind: EventKind::Transient,
                    text: "remaining history".into(),
                },
            ],
            HistoryPolicy {
                recent_history_bytes: 256,
            },
        )
        .await
        .unwrap();
    let checkpoint = f.packs().load_checkpoint(&reference).unwrap();
    assert_eq!(checkpoint.omitted_transient, 1);
    assert_eq!(checkpoint.recent.len(), 1);
    assert_eq!(checkpoint.recent[0].event.sequence, 2);
    assert!(checkpoint.recent_bytes <= 256);
    assert_eq!(
        encoding::encoded_len(&checkpoint).unwrap(),
        serde_json::to_vec(&checkpoint).unwrap().len()
    );
    assert_eq!(
        checkpoint.recent_bytes,
        checkpoint
            .recent
            .iter()
            .map(|e| serde_json::to_vec(e).unwrap().len())
            .sum::<usize>()
    );
    let control = f
        .store
        .lock()
        .unwrap()
        .record(reference.id)
        .unwrap()
        .unwrap();
    for (value, message) in [
        (json!({"padding":"x".repeat(MAX_BYTES)}), BYTES),
        (nested(120), DEPTH),
    ] {
        let mut record = control.clone();
        record.data["mandatory_goal"] = value;
        let r = f.corrupt_checkpoint(&record);
        reset();
        refused(f.packs().load_checkpoint(&r).bounded_err(), message);
        let mut stale = r.clone();
        stale.digest = "wrong".into();
        reset();
        refused(f.packs().load_checkpoint(&stale).bounded_err(), message);
        let mut foreign = r;
        foreign.scope.task_id = Some(TaskId::new());
        reset();
        refused(
            f.packs().load_checkpoint(&foreign).bounded_err(),
            "stale/foreign checkpoint reference",
        );
        let mut version = f.corrupt_checkpoint(&record);
        version.version = 2;
        reset();
        refused(
            f.packs().load_checkpoint(&version).bounded_err(),
            "stale/foreign checkpoint reference",
        );
        record.kind = RecordKind::Verification;
        let wrong_kind = f.corrupt_checkpoint(&record);
        reset();
        refused(
            f.packs().load_checkpoint(&wrong_kind).bounded_err(),
            "stale/foreign checkpoint reference",
        );
    }
    let r = f.corrupt_checkpoint(&control);
    let mut stale = r.clone();
    stale.digest = "wrong".into();
    reset();
    assert_eq!(
        f.packs()
            .load_checkpoint(&stale)
            .bounded_err()
            .root_cause()
            .to_string(),
        "stale/foreign checkpoint reference"
    );
    assert_eq!(encoding::take_read_stages(), encoding::CHECKPOINT_DIGEST);
    f.packs().load_checkpoint(&r).unwrap();
}

#[tokio::test]
async fn checkpoint_admission_rejects_extra_artifact_envelope_before_append() {
    let f = Fixture::new();
    let (mut context, _) = f.task_context().await;
    context.data["historical_consultation"] = nested(119); // Task root +119 =120
    let r = f.corrupt(&context);
    f.packs().task_pack(&r).unwrap();
    let session = f.factual_session();
    reset();
    let error = f
        .packs()
        .checkpoint(
            &f.task.scope(),
            session,
            None,
            vec![HistoryEvent {
                sequence: 1,
                kind: EventKind::Decision,
                text: "preserve exact facts".into(),
            }],
            HistoryPolicy {
                recent_history_bytes: 4096,
            },
        )
        .await
        .bounded_err();
    assert_eq!(error.root_cause().to_string(), DEPTH);
    assert_eq!(
        encoding::take_read_stages() & encoding::CHECKPOINT_APPEND,
        0
    );
    assert!(
        f.store
            .lock()
            .unwrap()
            .pack_checkpoint_head(&f.task.scope())
            .unwrap()
            .is_none()
    );
}

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
            refused(f.packs().load_context(&r, intent).bounded_err(), message);
        }
        reset();
        refused(f.packs().task_pack(&r).bounded_err(), message);
    }
    let mut c = control;
    c.data["format"] = json!("unknown.format");
    let fitting = f.corrupt(&c);
    assert_eq!(
        f.packs()
            .task_pack(&fitting)
            .bounded_err()
            .root_cause()
            .to_string(),
        "invalid Task pack envelope"
    );
    c.data["decisions"] = json!(["x".repeat(MAX_BYTES)]);
    let r = f.corrupt(&c);
    reset();
    refused(f.packs().task_pack(&r).bounded_err(), BYTES);
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
    reset();
    let pack = f.packs().goal_pack(&r).unwrap();
    assert_eq!(
        encoding::take_read_stages(),
        encoding::CONTEXT_DIGEST | encoding::TYPED_DECODE
    );
    assert_eq!(
        encoding::encoded_len(&pack).unwrap(),
        serde_json::to_vec(&pack).unwrap().len()
    );
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
        refused(f.packs().goal_pack(&r).bounded_err(), message);
        reset();
        refused(f.packs().validate_goal(&r).await.bounded_err(), message);
    }
    let mut c = control;
    c.data["format"] = json!("unknown.format");
    let fitting = f.corrupt(&c);
    assert_eq!(
        f.packs()
            .goal_pack(&fitting)
            .bounded_err()
            .root_cause()
            .to_string(),
        "invalid Goal pack envelope"
    );
    c.data["cross_task_decisions"] = json!(["x".repeat(MAX_BYTES)]);
    let r = f.corrupt(&c);
    reset();
    refused(f.packs().goal_pack(&r).bounded_err(), BYTES);
}

#[tokio::test]
async fn opaque_goal_format_in_task_scope_keeps_its_provenance_path() {
    let f = Fixture::new();
    let (mut c, _) = f.task_context().await;
    for format in [GOAL_FORMAT, "unknown.format"] {
        c.data = json!({"format":format,"opaque":"x".repeat(MAX_BYTES)});
        let r = f.corrupt(&c); // Negative corruption stands for historical opaque shape.
        reset();
        f.packs()
            .load_context(&r, ContextRead::TaskProvenance)
            .unwrap();
        assert_eq!(encoding::take_read_stages(), encoding::CONTEXT_DIGEST);
        let t = f.store.lock().unwrap().task(f.task.id).unwrap().unwrap();
        f.packs().validate_task_reference(&t, &r).await.unwrap();
        let goal = f
            .packs()
            .publish_goal(&f.goal.scope(), vec![], Default::default())
            .await
            .unwrap();
        let pack = f.packs().goal_pack(&goal).unwrap();
        assert_eq!(pack.tasks.len(), 1);
        assert!(!pack.tasks[0].typed_context);
        assert_eq!(pack.tasks[0].context, Some(r));
        f.packs().validate_goal(&goal).await.unwrap();
    }
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
    reset();
    workflow::validate_capture(&artifact, &source, Phase::Implement, &budget).unwrap();
    assert_eq!(encoding::take_read_stages(), encoding::TYPED_DECODE);
    let control = ContextVersion {
        scope: source.scope.clone(),
        version: 1,
        revision: source.revision.clone(),
        source_hashes: source.source_versions.clone(),
        data: json!({"task_pack":artifact,"phase":Phase::Implement,"budget":budget,
            "payload":source.payload,"source_payload_offset":0}),
    };
    reset();
    let typed_artifact = workflow::context_artifact(&control).unwrap();
    assert_eq!(encoding::take_read_stages(), encoding::TYPED_DECODE);
    assert_eq!(
        encoding::encoded_len(&typed_artifact).unwrap(),
        serde_json::to_vec(&typed_artifact).unwrap().len()
    );
    f.task_context().await; // Genuine publication before owned negative corruption.
    let phase_reference = f.corrupt(&control);
    reset();
    assert_eq!(
        f.packs()
            .validate_task_map(&phase_reference)
            .await
            .bounded_err()
            .root_cause()
            .to_string(),
        "workflow phase pack is prepared only by its Engine"
    );
    assert_eq!(encoding::take_read_stages(), encoding::CONTEXT_DIGEST);
    // Engine prepends mandatory rules outside the captured source artifact.
    // The delivered frame fits its own cap while the complete row exceeds the
    // artifact cap. This synthetic prefix is codec input, not launch authority.
    let mut wide = control.clone();
    let prefix = "r".repeat(MAX_BYTES - source.payload.len());
    wide.data["payload"] = json!(format!("{prefix}{}", source.payload));
    wide.data["source_payload_offset"] = json!(prefix.len());
    assert_eq!(wide.data["payload"].as_str().unwrap().len(), MAX_BYTES);
    assert!(serde_json::to_vec(&wide.data).unwrap().len() > MAX_BYTES);
    bounded(&wide.data["task_pack"]).unwrap();
    let wide_ref = f.corrupt(&wide);
    reset();
    f.packs().task_pack(&wide_ref).unwrap();
    assert_eq!(
        encoding::take_read_stages(),
        encoding::CONTEXT_DIGEST | encoding::TYPED_DECODE
    );
    reset();
    f.packs()
        .load_context(&wide_ref, ContextRead::TaskProvenance)
        .unwrap();
    assert_eq!(encoding::take_read_stages(), encoding::CONTEXT_DIGEST);
    let task = f.store.lock().unwrap().task(f.task.id).unwrap().unwrap();
    f.packs()
        .validate_task_reference(&task, &wide_ref)
        .await
        .unwrap();
    let goal_ref = f
        .packs()
        .publish_goal(&f.goal.scope(), vec![], Default::default())
        .await
        .unwrap();
    let goal_pack = f.packs().goal_pack(&goal_ref).unwrap();
    assert!(goal_pack.tasks[0].typed_context);
    assert_eq!(goal_pack.tasks[0].context, Some(wide_ref));
    f.packs().validate_goal(&goal_ref).await.unwrap();
    let mut c = control.clone();
    c.data["task_pack"]["source_versions"]["negative:padding"] = json!("x".repeat(MAX_BYTES));
    c.source_hashes
        .insert("negative:padding".into(), "x".repeat(MAX_BYTES));
    let mut altered_source = source.clone();
    altered_source.source_versions = c.source_hashes.clone();
    assert!(serde_json::to_vec(&c).unwrap().len() < 8 * MAX_BYTES);
    reset();
    refused(workflow::context_artifact(&c).bounded_err(), BYTES);
    let phase_reference = f.corrupt(&c);
    reset();
    refused(
        f.packs()
            .load_context(&phase_reference, ContextRead::TaskProvenance)
            .bounded_err(),
        BYTES,
    );
    reset();
    refused(
        f.packs()
            .validate_task_map(&phase_reference)
            .await
            .bounded_err(),
        BYTES,
    );
    reset();
    refused(
        workflow::validate_capture(
            &c.data["task_pack"],
            &altered_source,
            Phase::Implement,
            &budget,
        )
        .bounded_err(),
        BYTES,
    );
    c = control;
    let old_header = format!(
        "{}\n",
        serde_json::to_string(&json!({
            "kind":"phase_context_pack", "phase":Phase::Implement, "budget":budget,
            "body":c.data["task_pack"]["pack"],
        }))
        .unwrap()
    );
    c.data["task_pack"]["pack"]["task"] = nested(119); // artifact1 + pack1 +119 =121
    let mut changed: workflow::PhasePackArtifact =
        serde_json::from_value(c.data["task_pack"].clone()).unwrap();
    let header = format!(
        "{}\n",
        serde_json::to_string(&json!({
            "kind":"phase_context_pack", "phase":Phase::Implement, "budget":budget,
            "body":changed.pack,
        }))
        .unwrap()
    );
    let mut depth_source = source.clone();
    depth_source.payload = format!(
        "{header}{}",
        source.payload.strip_prefix(&old_header).unwrap()
    );
    changed.mandatory_bytes = changed
        .mandatory_bytes
        .checked_sub(old_header.len())
        .unwrap()
        .checked_add(header.len())
        .unwrap();
    changed.estimated_bytes = depth_source.payload.len();
    changed.estimated_tokens = depth_source.payload.len();
    changed.payload_digest = format!(
        "sha256:{:x}",
        Sha256::digest(depth_source.payload.as_bytes())
    );
    c.data["task_pack"] = serde_json::to_value(changed).unwrap();
    c.data["payload"] = json!(depth_source.payload);
    assert!(depth_source.payload.len() < MAX_BYTES);
    reset();
    refused(workflow::context_artifact(&c).bounded_err(), DEPTH);
    reset();
    refused(
        workflow::validate_capture(
            &c.data["task_pack"],
            &depth_source,
            Phase::Implement,
            &budget,
        )
        .bounded_err(),
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

// Separate checkpoint identity component; these are artifact-consumer controls,
// outside the encoder's 16-test qualification and all native authority gates.
fn append_body(f: &Fixture, session: SessionId, cp: &Checkpoint) -> (Record, Result<()>) {
    let mut record = Record::new(
        f.task.scope(),
        RecordKind::Checkpoint,
        serde_json::to_value(cp).unwrap(),
    );
    let mut store = f.store.lock().unwrap();
    let expected = [
        store.project(f.project.id).unwrap().unwrap().version,
        store.goal(f.goal.id).unwrap().unwrap().version,
        store.task(f.task.id).unwrap().unwrap().version,
    ];
    let version = store.session(session).unwrap().unwrap().1;
    let result = store.append_pack_checkpoint(
        &f.task.scope(),
        expected,
        session,
        version,
        cp.previous.as_ref(),
        &mut record,
    );
    (record, result)
}
fn reject_body(
    f: &Fixture,
    session: SessionId,
    cp: Checkpoint,
    previous: Option<&CheckpointRef>,
    message: &str,
) {
    let scope = f.task.scope();
    let mut store = f.store.lock().unwrap();
    let expected = [
        store.project(f.project.id).unwrap().unwrap().version,
        store.goal(f.goal.id).unwrap().unwrap().version,
        store.task(f.task.id).unwrap().unwrap().version,
    ];
    let version = store.session(session).unwrap().unwrap().1;
    let head = store.pack_checkpoint_head(&scope).unwrap();
    let events = store.events(&scope, 0, 100).unwrap().len();
    // The body is deliberately inconsistent; outer envelope and captured
    // arguments remain valid so the actual private consumer is the cause.
    let mut record = Record::new(
        scope.clone(),
        RecordKind::Checkpoint,
        serde_json::to_value(&cp).unwrap(),
    );
    let result =
        store.append_pack_checkpoint(&scope, expected, session, version, previous, &mut record);
    let Err(error) = result else {
        panic!("inconsistent checkpoint body accepted: {message}")
    };
    if message == "owned injected checkpoint audit failure" {
        assert_eq!(error.to_string(), message);
    } else {
        assert_eq!(error.root_cause().to_string(), message);
    }
    assert_eq!(record.version, 0);
    assert!(store.record(record.id).unwrap().is_none());
    assert_eq!(store.pack_checkpoint_head(&scope).unwrap(), head);
    assert_eq!(store.events(&scope, 0, 100).unwrap().len(), events);
}

#[tokio::test]
async fn checkpoint_append_binds_inner_identity_and_exact_mandatory_prefix() {
    let f = Fixture::new();
    let session = f.factual_session();
    let other_session = f.factual_session();
    let map = f
        .packs()
        .source()
        .index(&f.task.scope(), vec![])
        .await
        .unwrap();
    let events = vec![
        HistoryEvent {
            sequence: 1,
            kind: EventKind::Decision,
            text: "first exact decision".into(),
        },
        HistoryEvent {
            sequence: 2,
            kind: EventKind::Constraint,
            text: "second exact constraint".into(),
        },
    ];
    // Actual source observation and persisted factual Session, no raw seeded
    // positive authority. This body uses the same typed first-chain recipe as
    // the service. The resulting row must pass the actual service reader.
    let first = Checkpoint {
        format: CHECKPOINT.into(),
        scope: f.task.scope(),
        session,
        role: SessionRole::Consultant,
        previous: None,
        chain_version: 1,
        first_sequence: 1,
        last_sequence: 2,
        input_digest: digest(&events).unwrap(),
        authority: RepositoryRef::of(&map, vec![]).unwrap(),
        mandatory_goal: projection(&f.goal).unwrap(),
        mandatory_task: projection(&f.task).unwrap(),
        mandatory_rules: rules(&map),
        retained: events
            .into_iter()
            .map(|event| RetainedEvent { session, event })
            .collect(),
        recent: vec![],
        omitted_transient: 0,
        omitted_first_sequence: None,
        omitted_last_sequence: None,
        omitted_digest: None,
        recent_bytes: 0,
        recent_history_limit_bytes: Some(0),
        measured_tokens: None,
    };
    let mut fabricated = first.clone();
    fabricated.previous = Some(CheckpointRef {
        scope: f.task.scope(),
        id: RecordId::new(),
        version: 1,
        digest: "sha256:unknown".into(),
    });
    reject_body(
        &f,
        session,
        fabricated,
        None,
        "checkpoint body predecessor differs from append predecessor",
    );
    let (row, result) = append_body(&f, session, &first);
    result.unwrap();
    let reference = f
        .store
        .lock()
        .unwrap()
        .pack_checkpoint_head(&f.task.scope())
        .unwrap()
        .unwrap();
    assert_eq!(reference.id, row.id);
    let loaded = f.packs().load_checkpoint(&reference).unwrap();
    assert_eq!(loaded.retained, first.retained);
    let mut next = loaded;
    next.previous = Some(reference.clone());
    next.chain_version = 2;
    next.first_sequence = 3;
    next.last_sequence = 3;
    let event = HistoryEvent {
        sequence: 3,
        kind: EventKind::Decision,
        text: "third exact decision".into(),
    };
    next.input_digest = digest(&vec![event.clone()]).unwrap();
    next.retained.push(RetainedEvent { session, event });
    let mut cases = Vec::new();
    let mut cp = next.clone();
    cp.format = "unknown.format".into();
    cases.push((cp, "checkpoint body format differs from append contract"));
    let mut cp = next.clone();
    cp.scope.task_id = Some(TaskId::new());
    cases.push((cp, "checkpoint body scope differs from append scope"));
    for id in [SessionId::new(), other_session] {
        let mut cp = next.clone();
        cp.session = id;
        cases.push((cp, "checkpoint body Session differs from append Session"));
    }
    let mut cp = next.clone();
    cp.role = SessionRole::Reviewer;
    cases.push((cp, "checkpoint body role differs from append Session"));
    let mut cp = next.clone();
    cp.authority.worktree = f._temp.path().join("foreign-worktree");
    cases.push((cp, "checkpoint body worktree differs from append Session"));
    let mut cp = next.clone();
    cp.previous = None;
    cases.push((
        cp,
        "checkpoint body predecessor differs from append predecessor",
    ));
    let mut cp = next.clone();
    cp.previous.as_mut().unwrap().digest = "sha256:stale".into();
    cases.push((
        cp,
        "checkpoint body predecessor differs from append predecessor",
    ));
    let mut cp = next.clone();
    cp.retained.remove(1);
    cases.push((cp, "checkpoint body rewrites retained mandatory history"));
    let mut cp = next.clone();
    cp.retained.swap(0, 1);
    cases.push((cp, "checkpoint body rewrites retained mandatory history"));
    let mut cp = next.clone();
    cp.retained[1].event.text = "rewritten mandatory constraint".into();
    cases.push((cp, "checkpoint body rewrites retained mandatory history"));
    let mut cp = next.clone();
    cp.retained[1].event.kind = EventKind::Decision;
    cases.push((cp, "checkpoint body rewrites retained mandatory history"));
    let mut cp = next.clone();
    cp.retained[1].session = other_session;
    cases.push((cp, "checkpoint body rewrites retained mandatory history"));
    for (cp, message) in cases {
        reject_body(&f, session, cp, Some(&reference), message);
    }
    let connection = rusqlite::Connection::open(&f.db).unwrap();
    connection.execute_batch("CREATE TRIGGER fail_checkpoint_identity_audit BEFORE INSERT ON audit WHEN NEW.kind='checkpoint.saved' BEGIN SELECT RAISE(ABORT,'owned injected checkpoint audit failure'); END;").unwrap();
    reject_body(
        &f,
        session,
        next.clone(),
        Some(&reference),
        "owned injected checkpoint audit failure",
    );
    connection
        .execute_batch("DROP TRIGGER fail_checkpoint_identity_audit")
        .unwrap();
    let (_, result) = append_body(&f, session, &next);
    result.unwrap();
    let head = f
        .store
        .lock()
        .unwrap()
        .pack_checkpoint_head(&f.task.scope())
        .unwrap()
        .unwrap();
    assert_eq!(
        f.packs().load_checkpoint(&head).unwrap().retained,
        next.retained
    );
    // Finally exercise the sole service producer against that direct consumer's
    // valid predecessor, preserving both earlier mandatory events unchanged.
    let final_ref = f
        .packs()
        .checkpoint(
            &f.task.scope(),
            session,
            Some(head),
            vec![HistoryEvent {
                sequence: 4,
                kind: EventKind::Verification,
                text: "service producer preserved exact prefix".into(),
            }],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    assert!(
        f.packs()
            .load_checkpoint(&final_ref)
            .unwrap()
            .retained
            .starts_with(&next.retained)
    );
}
