use super::*;

fn finding() -> ReviewFinding {
    serde_json::from_value(json!({"id":"F1","severity":"high","blocking":false,"file":"src/code.rs","line":4,"end_line":5,"title":"title","rationale":"rationale"})).unwrap()
}
fn result() -> Value {
    json!({"schema":"rrx.confirm.v1","labels":[{"id":"F1","label":"confirmed","evidence":{"file":"src/code.rs","line":4,"quote":"abcdefgh"},"reason":"checked"}]})
}
fn snapshot() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/code.rs"),
        "one\ntwo\nthree\nabcdefgh\nsecond line\nsix\nseven\neight\nnine abcdefgh\n",
    )
    .unwrap();
    dir
}
fn decoded(v: &Value, dir: &Path) -> Result<Value> {
    decode(&serde_json::to_vec(v).unwrap(), &[finding()], dir)
}

/// Blocking means either the explicit flag or reaching the immutable severity threshold.
#[test]
fn b8_blocking() {
    let mut f = finding();
    assert!(rounds::blocking(&f, ReviewSeverity::High));
    assert!(!rounds::blocking(&f, ReviewSeverity::Critical));
    f.severity = ReviewSeverity::Low;
    f.blocking = true;
    assert!(rounds::blocking(&f, ReviewSeverity::Critical));
}

/// Every failing filter condition is recorded and excluded while nonblocking data stays informational.
#[test]
fn b8_filter_conditions() {
    let dir = snapshot();
    let base = finding();
    let changed = BTreeSet::from([base.file.clone()]);
    let scopes = vec!["src/**".into()];
    let run = |f: ReviewFinding, changes: &BTreeSet<String>, scopes: &[String]| {
        filter(
            &ReviewResult {
                schema: "rrx.review.v1".into(),
                verdict: super::super::ReviewVerdict::RequestChanges,
                summary: String::new(),
                findings: vec![f],
            },
            ReviewSeverity::High,
            dir.path(),
            &changed,
            changes,
            scopes,
        )
        .unwrap()
    };
    assert_eq!(run(base.clone(), &changed, &scopes).blocking.len(), 1);
    for path in [
        "/src/code.rs",
        "src/../src/code.rs",
        "src/./code.rs",
        "src//code.rs",
        "src/code.rs/",
    ] {
        let mut f = base.clone();
        f.file = path.into();
        let got = run(f, &changed, &scopes);
        assert!(got.blocking.is_empty());
        assert_eq!(got.invalid[0].reason, InvalidReason::Path);
    }
    let mut f = base.clone();
    f.file = "src/missing.rs".into();
    assert_eq!(
        run(f, &changed, &scopes).invalid[0].reason,
        InvalidReason::MissingFile
    );
    for (line, end) in [(10, 10), (4, 10), (5, 4)] {
        let mut f = base.clone();
        f.line = line;
        f.end_line = Some(end);
        assert_eq!(
            run(f, &changed, &scopes).invalid[0].reason,
            InvalidReason::LineRange
        );
    }
    assert_eq!(
        run(base.clone(), &BTreeSet::new(), &scopes).invalid[0].reason,
        InvalidReason::UnchangedFile
    );
    assert_eq!(
        run(base.clone(), &changed, &["src/code".into()]).invalid[0].reason,
        InvalidReason::OutsideScope
    );
    let mut non = base;
    non.severity = ReviewSeverity::Low;
    non.file = "bad/../path".into();
    let got = run(non, &changed, &scopes);
    assert!(got.blocking.is_empty() && got.invalid.is_empty());
    assert_eq!(got.non_blocking, ["F1"]);
}

/// Filesystem aliases absent from the exact reviewed tree are invalid before file access.
#[test]
fn b8_filter_exact_tree_paths() {
    let dir = snapshot();
    let base = finding();
    let reviewed = BTreeSet::from([base.file.clone(), "src/caf\u{e9}.rs".into()]);
    for file in ["src/CODE.rs", "src/cafe\u{301}.rs"] {
        // Materialize an alias too, so this protects membership on case-sensitive hosts.
        std::fs::write(
            dir.path().join(file),
            std::fs::read(dir.path().join(&base.file)).unwrap(),
        )
        .unwrap();
        let filtered = filter(
            &ReviewResult {
                schema: "rrx.review.v1".into(),
                verdict: super::super::ReviewVerdict::RequestChanges,
                summary: String::new(),
                findings: vec![ReviewFinding {
                    file: file.into(),
                    ..base.clone()
                }],
            },
            ReviewSeverity::High,
            dir.path(),
            &reviewed,
            &BTreeSet::from([file.into()]),
            &["src/**".into()],
        )
        .unwrap();
        assert!(filtered.blocking.is_empty());
        assert_eq!(filtered.invalid[0].id, base.id);
        assert_eq!(filtered.invalid[0].reason, InvalidReason::MissingFile);
    }
}

/// Extra, missing and duplicate labels cannot become a partial confirmation.
#[test]
fn b8_exact_label_set() {
    let dir = snapshot();
    decoded(&result(), dir.path()).unwrap();
    let mut extra = result();
    let mut label = extra["labels"][0].clone();
    label["id"] = json!("F2");
    extra["labels"].as_array_mut().unwrap().push(label);
    let mut duplicate = result();
    let label = duplicate["labels"][0].clone();
    duplicate["labels"].as_array_mut().unwrap().push(label);
    let mut missing = result();
    missing["labels"] = json!([]);
    for v in [extra, duplicate, missing] {
        assert_eq!(
            decoded(&v, dir.path()).unwrap_err().to_string(),
            "confirm_label_ids"
        );
    }
}

/// Unknown fields, duplicate keys and invalid schema values are rejected before evidence checks.
#[test]
fn b8_decoder_schema() {
    let dir = snapshot();
    for (pointer, value) in [
        ("/schema", json!("other")),
        ("/labels/0/label", json!("invalid")),
        ("/labels/0/evidence/line", json!(0)),
        ("/labels/0/evidence/quote", json!("")),
        ("/labels/0/evidence/quote", json!("x".repeat(513))),
        ("/labels/0/reason", json!("x".repeat(2001))),
    ] {
        let mut v = result();
        *v.pointer_mut(pointer).unwrap() = value;
        assert_eq!(
            decoded(&v, dir.path()).unwrap_err().to_string(),
            "confirm_schema"
        );
    }
    for pointer in ["", "/labels/0", "/labels/0/evidence"] {
        let mut v = result();
        v.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("extra".into(), json!(true));
        assert_eq!(
            decoded(&v, dir.path()).unwrap_err().to_string(),
            "confirm_schema"
        );
    }
    let bytes = serde_json::to_string(&result()).unwrap().replacen(
        "\"schema\":",
        "\"schema\":\"rrx.confirm.v1\",\"schema\":",
        1,
    );
    assert_eq!(
        decode(bytes.as_bytes(), &[finding()], dir.path())
            .unwrap_err()
            .to_string(),
        "confirm_schema"
    );
    assert_eq!(
        decode(&vec![b' '; CONFIRM_BYTES + 1], &[finding()], dir.path())
            .unwrap_err()
            .to_string(),
        "confirm_schema"
    );
}

/// Failed confirmed evidence drops to unclear instead of rejecting the label set.
#[test]
fn b8_evidence_checks() {
    let dir = snapshot();
    assert_eq!(
        decoded(&result(), dir.path()).unwrap()["confirmation"]["labels"][0]["label"],
        "confirmed"
    );
    for (pointer, value) in [
        ("/labels/0/evidence/file", json!("src/other.rs")),
        ("/labels/0/evidence/line", json!(10)),
        ("/labels/0/evidence/line", json!(9)),
        ("/labels/0/evidence/line", json!(5)),
        ("/labels/0/evidence/quote", json!("abcdefg ")),
        ("/labels/0/evidence/quote", json!("absent quote")),
    ] {
        let mut v = result();
        *v.pointer_mut(pointer).unwrap() = value;
        assert_eq!(
            decoded(&v, dir.path()).unwrap()["confirmation"]["labels"][0]["label"],
            "unclear",
            "{pointer}"
        );
    }
    let mut v = result();
    v["labels"][0]["evidence"]["quote"] = json!("abcdefgh\nsecond line");
    assert_eq!(
        decoded(&v, dir.path()).unwrap()["confirmation"]["labels"][0]["label"],
        "confirmed"
    );
    v["labels"][0]["evidence"]["line"] = json!(5);
    assert_eq!(
        decoded(&v, dir.path()).unwrap()["confirmation"]["labels"][0]["label"],
        "unclear"
    );
    let mut later = finding();
    later.line = 8;
    later.end_line = Some(8);
    assert!(!evidence_matches(
        &Evidence {
            file: later.file.clone(),
            line: 4,
            quote: "abcdefgh".into()
        },
        &later,
        &std::fs::read_to_string(dir.path().join("src/code.rs")).unwrap()
    ));
    assert!(evidence_matches(
        &Evidence {
            file: later.file.clone(),
            line: 2,
            quote: "abcde\nabcde".into()
        },
        &ReviewFinding {
            line: 2,
            end_line: Some(2),
            ..later
        },
        "abcde\nabcde\nabcde\n"
    ));
}

/// Raw and escaped secrets are refused before malformed confirmation JSON is decoded or stored.
#[test]
fn b8_confirm_secret_scan() {
    let dir = snapshot();
    for bytes in [
        b"EXAMPLE_TOKEN=fixture".as_slice(),
        br#"{"reason":"EXAMPLE\u005fTOKEN=fixture","reason":0}"#,
    ] {
        assert!(
            decode(bytes, &[finding()], dir.path())
                .unwrap_err()
                .downcast_ref::<super::super::SecretHit>()
                .is_some()
        );
    }
    let mut v = result();
    v["labels"][0]["reason"] = json!("EXAMPLE_TOKEN=fixture");
    assert!(
        decoded(&v, dir.path())
            .unwrap_err()
            .downcast_ref::<super::super::SecretHit>()
            .is_some()
    );
}

/// Confirmation prompts frame findings and diffs together and scan them before persistence.
#[test]
fn b8_confirm_prompt() {
    let nonce = "abcdef0123456789abcdef0123456789";
    let prompt = String::from_utf8(prompt(nonce, &[finding()], b"diff data").unwrap()).unwrap();
    assert!(prompt.contains(&format!("BEGIN_UNTRUSTED_{nonce}\nData: ")));
    assert!(prompt.ends_with(&format!("Diff:\ndiff data\nEND_UNTRUSTED_{nonce}\n")));
    assert_eq!(
        super::prompt(nonce, &[finding()], nonce.as_bytes())
            .unwrap_err()
            .to_string(),
        "review_nonce_collision"
    );
    assert!(
        super::prompt(nonce, &[finding()], b"EXAMPLE_TOKEN=fixture")
            .unwrap_err()
            .downcast_ref::<super::super::SecretHit>()
            .is_some()
    );
}

/// Confirmation uses precisely the fixed read-only Codex template with an output schema.
#[test]
fn b8_confirm_argv() {
    let cli = crate::config::PipelineCli {
        executable: "/codex".into(),
        version: "pin".into(),
        model: "model".into(),
        effort: Some(crate::config::ReasoningEffort::High),
        codex_home: Some("/home".into()),
    };
    let args = super::super::step_arguments(
        super::super::PipelineRole::Confirm,
        &cli,
        Path::new("/snapshot"),
        Path::new("/attempt"),
        CONFIRM_SCHEMA,
    )
    .unwrap();
    assert_eq!(
        args,
        [
            "exec",
            "--sandbox",
            "read-only",
            "-c",
            "projects./snapshot.trust_level=\"untrusted\"",
            "-m",
            "model",
            "-c",
            "model_reasoning_effort=high",
            "-C",
            "/snapshot",
            "--output-schema",
            "/attempt/confirm.schema.json",
            "-o",
            "/attempt/last.json",
            "--json",
            "--color",
            "never",
            "-"
        ]
    );
}

fn retained() -> (tempfile::TempDir, PipelineStep) {
    let dir = tempfile::tempdir().unwrap();
    let mut step: PipelineStep=serde_json::from_value(json!({"run_id":"data","seq":1,"attempt":1,"kind":"confirm","provider":"codex","owner_epoch":1,"round":0,"state":"exited","version":1,"cookie":null,"pid":null,"birth":null,"step_base":"a".repeat(40),"input_sha256":"b".repeat(64),"result_commit":null,"result_tree":null,"body":{"death_confirmed":true,"provider_outcome":"success","exit":{}}})).unwrap();
    for (name, key, bytes) in [
        ("last.json", "last", serde_json::to_vec(&result()).unwrap()),
        (
            "stdout.jsonl",
            "stdout",
            b"{\"type\":\"turn.started\"}\n{\"type\":\"turn.completed\"}\n".to_vec(),
        ),
        ("stderr.log", "stderr", b"diagnostic".to_vec()),
    ] {
        std::fs::write(dir.path().join(name), &bytes).unwrap();
        std::fs::set_permissions(
            dir.path().join(name),
            std::fs::Permissions::from_mode(0o400),
        )
        .unwrap();
        step.body["exit"][key] = json!({"bytes":bytes.len(),"sha256":hash(&bytes),"complete":true});
    }
    (dir, step)
}

/// Every confirmation artifact requires a complete matching receipt and a sealed regular file.
#[test]
fn b8_output_receipts() {
    let (dir, base) = retained();
    let root = dir.path().canonicalize().unwrap();
    assert_eq!(
        output(&base, &root).unwrap(),
        serde_json::to_vec(&result()).unwrap()
    );
    for key in ["last", "stdout", "stderr"] {
        for (field, value) in [
            ("bytes", json!(999)),
            ("sha256", json!("0".repeat(64))),
            ("complete", json!(false)),
        ] {
            let mut step = base.clone();
            step.body["exit"][key][field] = value;
            assert_eq!(
                output(&step, &root).unwrap_err().to_string(),
                "confirm_output_changed"
            );
        }
    }
    for name in ["last.json", "stdout.jsonl", "stderr.log"] {
        std::fs::set_permissions(root.join(name), std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            output(&base, &root).unwrap_err().to_string(),
            "confirm_output_changed"
        );
        std::fs::set_permissions(root.join(name), std::fs::Permissions::from_mode(0o400)).unwrap();
    }
    let mut step = base;
    step.body["death_confirmed"] = json!(false);
    assert_eq!(
        output(&step, &root).unwrap_err().to_string(),
        "confirm_output_changed"
    );
}

/// Raw artifact bytes and nested stored strings are scanned before labels can be adopted.
#[test]
fn b8_output_secrets() {
    for name in ["last.json", "stdout.jsonl", "stderr.log", "body"] {
        let (dir, mut step) = retained();
        let root = dir.path().canonicalize().unwrap();
        if name == "body" {
            step.body["nested"] = json!([{"extra":"EXAMPLE_TOKEN=fixture"}]);
        } else {
            let bytes = b"EXAMPLE\\u005fTOKEN=fixture";
            std::fs::set_permissions(root.join(name), std::fs::Permissions::from_mode(0o600))
                .unwrap();
            std::fs::write(root.join(name), bytes).unwrap();
            std::fs::set_permissions(root.join(name), std::fs::Permissions::from_mode(0o400))
                .unwrap();
            let key = match name {
                "last.json" => "last",
                "stdout.jsonl" => "stdout",
                _ => "stderr",
            };
            step.body["exit"][key] =
                json!({"bytes":bytes.len(),"sha256":hash(bytes),"complete":true});
        }
        assert!(
            output(&step, &root)
                .unwrap_err()
                .downcast_ref::<super::super::SecretHit>()
                .is_some(),
            "{name}"
        );
    }
}
