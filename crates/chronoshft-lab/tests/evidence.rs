mod support;
use chronoshft_lab::{
    evidence, registry,
    runtime::{self, ArtifactOptions, Git, PinnedDefinitions},
};
use serde_json::{json, Value};
use std::fs;
use support::*;

fn reject(result: Result<(), String>, fragment: &str) {
    let error = result.expect_err("unexpected acceptance");
    assert!(error.contains(fragment), "expected {fragment}: {error}");
}
fn check_run(f: &Fixture, run: &Value) -> Result<(), String> {
    evidence::verify_run(
        f.root(),
        run,
        &mut f.resolver(),
        &f.records(),
        &ArtifactOptions::default(),
    )
}
fn check_decision(f: &Fixture, decision: &Value, current: bool) -> Result<(), String> {
    evidence::verify_decision(
        f.root(),
        decision,
        &mut f.resolver(),
        &f.records(),
        &ArtifactOptions::default(),
        current,
    )
}

#[test]
fn live_retained_evidence_and_synthetic_complete_fixture_verify() {
    let root = project();
    let report = evidence::verify_all(&root, &ArtifactOptions::default()).unwrap();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    // The original observation must survive later append-only captures.
    assert!(report.records.contains_key("RUN-exp001-local-001"));
    let f = Fixture::new();
    assert!(f.verify().is_empty(), "{:?}", f.verify());
    check_decision(&f, &f.decision(), true).unwrap();
}
#[test]
fn artifacts_detect_byte_size_and_missing_content_changes() {
    let f = Fixture::new();
    let evidence = f.read("validation/evidence-manifests/EVD-fixture-001.yaml");
    let path = f.root().join("validation/artifacts/fixture-1.txt");
    let original = fs::read(&path).unwrap();
    let mut changed = original.clone();
    changed[0] ^= 1;
    fs::write(&path, changed).unwrap();
    assert!(
        runtime::verify_artifact(f.root(), &evidence, &ArtifactOptions::default())
            .unwrap_err()
            .contains("SHA-256 mismatch")
    );
    fs::write(&path, b"short").unwrap();
    assert!(runtime::verify_artifact(f.root(), &evidence, &ArtifactOptions::default()).is_err());
    fs::remove_file(path).unwrap();
    assert!(
        runtime::verify_artifact(f.root(), &evidence, &ArtifactOptions::default())
            .unwrap_err()
            .contains("unavailable")
    );
}
#[test]
fn external_mirrors_and_repository_mirror_roots_are_explicit_and_hash_checked() {
    let f = Fixture::new();
    let mut e = f.read("validation/evidence-manifests/EVD-fixture-001.yaml");
    let source = f.root().join("validation/artifacts/fixture-1.txt");
    e["artifact"]["uri"] = json!("https://evidence.invalid/asset");
    assert!(
        runtime::verify_artifact(f.root(), &e, &ArtifactOptions::default())
            .unwrap_err()
            .contains("explicit local mirror")
    );
    let mirror = tempfile::tempdir().unwrap();
    fs::copy(&source, mirror.path().join("asset.txt")).unwrap();
    let map = mirror.path().join("map.yaml");
    fs::write(&map, "EVD-fixture-001: asset.txt\n").unwrap();
    let options = ArtifactOptions {
        mapping: runtime::read_artifact_map(&map).unwrap(),
        root: None,
    };
    runtime::verify_artifact(f.root(), &e, &options).unwrap();
    fs::write(mirror.path().join("asset.txt"), b"tampered").unwrap();
    assert!(runtime::verify_artifact(f.root(), &e, &options).is_err());
    let relative = "validation/artifacts/fixture-1.txt";
    let target = mirror.path().join(relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(&source, &target).unwrap();
    fs::remove_file(source).unwrap();
    e["artifact"]["uri"] = json!(format!("repo://{relative}"));
    runtime::verify_artifact(
        f.root(),
        &e,
        &ArtifactOptions {
            mapping: Default::default(),
            root: Some(mirror.path().to_owned()),
        },
    )
    .unwrap();
    fs::write(&map, "EVD-fixture-001: [bad]\n").unwrap();
    assert!(runtime::read_artifact_map(&map).is_err());
}
#[test]
fn historical_blobs_survive_deleted_worktree_files_and_use_pinned_schemas() {
    let f = Fixture::new();
    let mut resolver = f.resolver();
    let reference = runtime::reference("TST-001", &json!(1), &f.commit);
    fs::remove_file(f.root().join("fixture.rs")).unwrap();
    fs::remove_file(f.root().join("validation/schemas/test.schema.json")).unwrap();
    assert_eq!(resolver.record(&reference).unwrap()["revision"], 1);
    assert!(resolver
        .blob(&f.commit, "fixture.rs")
        .unwrap()
        .starts_with(b"// Synthetic"));
    assert!(resolver.commit("HEAD").is_err());
    assert!(resolver.commit(&"0".repeat(40)).is_err());
    assert!(resolver.blob(&f.commit, "../outside.txt").is_err());
    let f = Fixture::new();
    let mut old = f.read(TEST);
    old["title"] = json!("Frozen revision one");
    f.write("validation/revisions/tests/TST-001/r1.yaml", &old);
    f.edit(TEST,|t|{t["revision"]=json!(2);t["revision_history"].as_array_mut().unwrap().push(json!({"revision":2,"date":"2026-10-15","actor":"fixture","reason":"Synthetic revision"}));});
    let commit = f.commit_all();
    assert_eq!(
        f.resolver()
            .record(&runtime::reference("TST-001", &json!(1), &commit))
            .unwrap()["title"],
        "Frozen revision one"
    );
}
#[test]
fn observed_cases_require_exact_bound_argv_ownership_exit_codes_and_evidence() {
    let f = Fixture::new();
    let mut run = f.read(RUN);
    run["tests"][0]["command"] = json!(["wrong"]);
    reject(check_run(&f, &run), "command differs");
    let mut run = f.read(RUN);
    run["tests"][0]["exit_code"] = json!(1);
    reject(check_run(&f, &run), "exit code zero");
    let mut run = f.read(RUN);
    run["tests"][0]["evidence_ids"] = json!([]);
    reject(check_run(&f, &run), "needs retained evidence");
    let mut run = f.read(RUN);
    run["tests"].as_array_mut().unwrap().pop();
    reject(check_run(&f, &run), "every declared test");
    let mut run = f.read(RUN);
    run["tests"][0]["test"]["revision"] = json!(2);
    reject(check_run(&f, &run), "test revision does not match");
    let mut run = f.read(RUN);
    run["source"]["registry_commit"] = json!("1".repeat(40));
    reject(check_run(&f, &run), "inconsistent registry commits");
}
#[test]
fn skipped_error_and_aborted_observations_keep_their_distinct_outcomes() {
    let f = Fixture::new();
    let mut run = f.read(RUN);
    run["tests"][0]["outcome"] = json!("skipped");
    run["tests"][0]["exit_code"] = Value::Null;
    run["outcome"] = json!("incomplete");
    check_run(&f, &run).unwrap();
    run["outcome"] = json!("passed");
    reject(check_run(&f, &run), "aggregate outcome");
    let mut run = f.read(RUN);
    run["tests"][0]["outcome"] = json!("error");
    run["tests"][0]["exit_code"] = Value::Null;
    run["outcome"] = json!("error");
    check_run(&f, &run).unwrap();
    run["outcome"] = json!("aborted");
    check_run(&f, &run).unwrap();
    let manifest: Value = serde_json::from_slice(
        &fs::read(project().join("tests/compatibility/python-v1/manifest.json")).unwrap(),
    )
    .unwrap();
    for case in manifest["aggregate_cases"].as_array().unwrap() {
        let tests = case["outcomes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|outcome| json!({"outcome":outcome}))
            .collect::<Vec<_>>();
        let flags = &case["flags"];
        assert_eq!(
            evidence::aggregate(
                &tests,
                flags["aborted"].as_bool().unwrap_or(false),
                flags["source_changed"].as_bool().unwrap_or(false),
                flags["infrastructure_error"].as_bool().unwrap_or(false)
            ),
            case["expected"]
        );
    }
}
#[test]
fn input_lockfile_evidence_scope_and_dirty_patch_requirements_are_verified() {
    let f = Fixture::new();
    let mut run = f.read(RUN);
    run["inputs"][0]["sha256"] = json!("0".repeat(64));
    reject(check_run(&f, &run), "Input hash differs");
    let mut run = f.read(RUN);
    run["source"]["dependency_lockfiles"][0]["sha256"] = json!("0".repeat(64));
    reject(check_run(&f, &run), "lockfile hash differs");
    let mut run = f.read(RUN);
    run["source"]["dirty"] = json!(true);
    run["source"]["patch_evidence_id"] = json!("EVD-fixture-001");
    reject(check_run(&f, &run), "source-patch evidence");
    f.edit("validation/evidence-manifests/EVD-fixture-001.yaml", |e| {
        e["supports"][0]["validation"]["revision"] = json!(2)
    });
    reject(check_run(&f, &f.read(RUN)), "evidence scope");
}
#[test]
fn positive_decisions_require_complete_case_and_evidence_obligations() {
    let f = Fixture::new();
    let mut decision = f.decision();
    decision["evidence_assessment"] = json!([]);
    reject(check_decision(&f, &decision, true), "explicit assessment");
    let mut decision = f.decision();
    decision["unresolved_findings"] = json!([{"description":"Synthetic critical defect","severity":"critical","invalidates_conclusion":true}]);
    reject(check_decision(&f, &decision, true), "invalidating finding");
    let mut decision = f.decision();
    decision["evidence_ids"].as_array_mut().unwrap().pop();
    decision["evidence_assessment"][0]["evidence_ids"] = decision["evidence_ids"].clone();
    reject(check_decision(&f, &decision, true), "supporting artifacts");
    f.edit(RUN, |r| {
        r["tests"][0]["outcome"] = json!("skipped");
        r["tests"][0]["exit_code"] = Value::Null;
        r["outcome"] = json!("incomplete");
    });
    reject(
        check_decision(&f, &f.decision(), true),
        "every mandatory applicable test",
    );
}
#[test]
fn reviewer_clock_concrete_scope_and_conditional_limits_are_required() {
    let f = Fixture::new();
    for reviewer in ["codex", " pending ", "automation"] {
        let mut d = f.decision();
        d["reviewer"] = json!(reviewer);
        reject(check_decision(&f, &d, true), "named human");
    }
    let mut d = f.decision();
    d["reviewed_at"] = json!("9999-01-01T00:00:00Z");
    reject(check_decision(&f, &d, true), "future");
    let mut d = f.decision();
    d["reviewed_at"] = json!("2019-01-01T00:00:00Z");
    reject(check_decision(&f, &d, true), "predates");
    let mut d = f.decision();
    d["rationale"] = json!("tbd");
    reject(check_decision(&f, &d, true), "concrete rationale");
    let mut d = f.decision();
    d["disposition"] = json!("conditional");
    d["follow_up"] = json!([{"action":"Synthetic follow-up","owner":"fixture","review_condition":"More observations"}]);
    d["applicability"]["limitations"] = json!([]);
    d["applicability"]["exclusions"] = json!([]);
    reject(check_decision(&f, &d, true), "explicit limitation");
    d["applicability"]["limitations"] = json!(["Synthetic scope limit"]);
    check_decision(&f, &d, true).unwrap();
}
#[test]
fn historical_decision_remains_historical_after_the_current_definition_changes() {
    let f = Fixture::new();
    let d = f.decision();
    f.edit(VAL, |v| v["objective"] = json!("New engineering scope"));
    check_decision(&f, &d, false).unwrap();
    reject(
        check_decision(&f, &d, true),
        "current validation definition",
    );
}
#[test]
fn later_contradictory_context_blocks_current_acceptance_but_not_an_older_historical_review() {
    let f = Fixture::new();
    let mut failed = f.read(RUN);
    failed["id"] = json!("RUN-fixture-later");
    failed["started_at"] = json!("2020-01-01T00:02:00Z");
    failed["ended_at"] = json!("2020-01-01T00:02:01Z");
    failed["tests"][0]["outcome"] = json!("failed");
    failed["tests"][0]["exit_code"] = json!(1);
    failed["outcome"] = json!("failed");
    // Unselected contrary observations need not reuse the selected run's producer
    // artifacts to block acceptance. This candidate is not published as evidence.
    f.write("validation/executions/RUN-fixture-later.yaml", &failed);
    let decision = f.decision();
    check_decision(&f, &decision, false).unwrap();
    reject(check_decision(&f, &decision, true), "later contradictory");
    let mut d = decision.clone();
    d["reviewed_at"] = json!("2020-01-01T00:03:00Z");
    reject(check_decision(&f, &d, false), "later contradictory");
}
#[test]
fn supersession_cycles_invalid_predecessors_and_bad_plan_references_are_rejected() {
    let f = Fixture::new();
    f.edit("validation/evidence-manifests/EVD-fixture-001.yaml", |e| {
        e["supersedes"] = json!("EVD-fixture-002")
    });
    f.edit("validation/evidence-manifests/EVD-fixture-002.yaml", |e| {
        e["supersedes"] = json!("EVD-fixture-001")
    });
    assert!(f.verify().iter().any(|e| e.contains("cycle")));
    let f = Fixture::new();
    let mut plan =
        registry::read_yaml(&project().join("validation/execution-plans/EXP-001.yaml")).unwrap();
    plan["experiment"] = runtime::reference("EXP-001", &json!(2), &f.commit);
    plan["implementation_commit"] = json!(f.commit);
    for t in plan["tests"].as_array_mut().unwrap() {
        t["revision"] = json!(1);
    }
    plan["tests"][0]["revision"] = json!(2);
    f.write("validation/execution-plans/fixture.yaml", &plan);
    assert!(f.verify().iter().any(|e| e.contains("foreign/stale test")));
}
#[test]
fn binding_split_matches_posix_shlex_without_running_a_shell() {
    let cases = [
        (
            "cmd 'event token' \"\" #literal",
            vec!["cmd", "event token", "", "#literal"],
        ),
        ("cmd a\\ b \"x\\$y\"", vec!["cmd", "a b", "x\\$y"]),
        ("a'b'\"c\"", vec!["abc"]),
        ("cmd \\\nline", vec!["cmd", "\nline"]),
    ];
    for (input, expected) in cases {
        assert_eq!(runtime::split_binding(input).unwrap(), expected);
    }
    for bad in ["'unfinished", "abc\\"] {
        assert!(runtime::split_binding(bad).is_err());
    }
}
#[cfg(unix)]
#[test]
fn symlink_and_parent_traversal_cannot_escape_artifact_or_git_roots() {
    let f = Fixture::new();
    let outside = tempfile::tempdir().unwrap();
    let target = f.root().join("alias");
    std::os::unix::fs::symlink(outside.path(), &target).unwrap();
    assert!(runtime::safe_path(f.root(), "alias/nonexistent").is_err());
    assert!(runtime::safe_path(f.root(), "../outside").is_err());
    let git = Git::new(f.root()).unwrap();
    assert!(git.blob(&f.commit, "alias/secret.txt").is_err());
    let mut evidence = f.read("validation/evidence-manifests/EVD-fixture-001.yaml");
    evidence["artifact"]["uri"] = json!("repo://alias/secret.txt");
    assert!(runtime::verify_artifact(f.root(), &evidence, &ArtifactOptions::default()).is_err());
}
#[test]
fn sha256_streams_across_chunk_boundaries_and_rejects_missing_git_content() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("artifact");
    fs::write(&path, b"abc").unwrap();
    assert_eq!(
        runtime::file_digest(&path).unwrap(),
        (
            3,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()
        )
    );
    let bytes = vec![b'x'; 2 * 1024 * 1024 + 17];
    fs::write(&path, &bytes).unwrap();
    assert_eq!(
        runtime::file_digest(&path).unwrap(),
        (bytes.len() as u64, runtime::bytes_digest(&bytes))
    );
    let f = Fixture::new();
    assert!(PinnedDefinitions::new(f.root())
        .unwrap()
        .blob(&f.commit, "missing.rs")
        .is_err());
}

#[test]
fn manual_observations_cannot_fabricate_automated_commands_or_exit_codes() {
    let f = Fixture::new();
    f.edit(EXP, |e| e["method"]["kind"] = json!("documentary"));
    for n in 1..=4 {
        f.edit(&format!("validation/register/tests/TST-{n:03}.yaml"), |t| {
            t["execution_type"] = json!("manual");
            t["execution_binding"]["implementation_path"] = Value::Null;
            t["execution_binding"]["entrypoint"] = json!("Synthetic review checklist");
        });
    }
    let commit = f.commit_all();
    let mut run = f.read(RUN);
    run["experiment"]["registry_commit"] = json!(commit);
    run["source"]["registry_commit"] = json!(commit);
    run["source"]["implementation_commit"] = json!(commit);
    for case in run["tests"].as_array_mut().unwrap() {
        case["test"]["registry_commit"] = json!(commit);
        case["command"] = json!([]);
        case["exit_code"] = Value::Null;
    }
    for n in 1..=4 {
        f.edit(
            &format!("validation/evidence-manifests/EVD-fixture-{n:03}.yaml"),
            |e| {
                e["supports"][0]["validation"]["registry_commit"] = json!(commit);
                e["provenance"]["source_version"] = json!(commit);
            },
        );
    }
    check_run(&f, &run).unwrap();
    run["tests"][0]["command"] = json!(["invented"]);
    reject(check_run(&f, &run), "Manual observations");
}
#[test]
fn an_observed_unbound_pinned_test_is_rejected() {
    let f = Fixture::new();
    f.edit(
        TEST,
        |t| {
            t["execution_binding"] =
                json!({"state":"unbound","implementation_path":null,"entrypoint":null})
        },
    );
    let commit = f.commit_all();
    let mut run = f.read(RUN);
    run["experiment"]["registry_commit"] = json!(commit);
    run["source"]["registry_commit"] = json!(commit);
    for t in run["tests"].as_array_mut().unwrap() {
        t["test"]["registry_commit"] = json!(commit);
    }
    reject(check_run(&f, &run), "method was unbound");
}
#[test]
fn supported_negative_decisions_do_not_require_every_case_to_pass() {
    let f = Fixture::new();
    f.edit(RUN, |r| {
        r["tests"][0]["outcome"] = json!("failed");
        r["tests"][0]["exit_code"] = json!(1);
        r["outcome"] = json!("failed");
    });
    let mut d = f.decision();
    d["disposition"] = json!("rejected");
    d["follow_up"] = json!([{"action":"Redesign the synthetic fixture","owner":"fixture","review_condition":"New evidence"}]);
    check_decision(&f, &d, true).unwrap();
    d["disposition"] = json!("inconclusive");
    check_decision(&f, &d, true).unwrap();
}
#[test]
fn dirty_observations_need_a_patch_and_cannot_support_positive_acceptance() {
    let f = Fixture::new();
    f.edit("validation/evidence-manifests/EVD-fixture-001.yaml", |e| {
        e["kind"] = json!("source_patch")
    });
    f.edit(RUN, |r| {
        r["source"]["dirty"] = json!(true);
        r["source"]["patch_evidence_id"] = json!("EVD-fixture-001");
    });
    check_run(&f, &f.read(RUN)).unwrap();
    reject(
        check_decision(&f, &f.decision(), true),
        "clean, valid execution",
    );
}
