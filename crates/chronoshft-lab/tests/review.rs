//! Synthetic human attestations in disposable Git fixtures, never project approvals.
mod support;
use chronoshft_lab::{
    registry, review,
    runtime::{self, ArtifactOptions},
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    process::{Command, Output},
};
use support::{project, Fixture, RUN, VAL};
fn check(f: &Fixture, candidate: &Value, register: bool) -> Result<Value, String> {
    review::check(f.root(), candidate, &ArtifactOptions::default(), register)
}
fn reject(f: &Fixture, candidate: &Value, fragment: &str) {
    let error = check(f, candidate, true).unwrap_err();
    assert!(error.contains(fragment), "expected {fragment}: {error}");
    assert!(!f.root().join("validation/decisions/VDEC-001.yaml").exists());
}
fn prepare(f: &Fixture) -> Value {
    review::prepare(
        f.root(),
        "VAL-012",
        "VDEC-001",
        &f.root().join("evidence/draft.yaml"),
    )
    .unwrap()
}
fn invoke(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"))
        .args(["review", args[0], "--root", root.to_str().unwrap()])
        .args(&args[1..])
        .env_remove("VALIDATION_ARTIFACT_MAP")
        .output()
        .unwrap()
}
#[test]
fn preparation_preserves_unapproved_fields_requirements_and_coverage() {
    let f = Fixture::new();
    let val_before = fs::read(f.root().join(VAL)).unwrap();
    let d = prepare(&f);
    let val = f.read(VAL);
    assert_eq!(d["draft"], true);
    assert_eq!(d["mandatory_experiments"], val["coverage"]["experiments"]);
    assert_eq!(d["mandatory_tests"], val["coverage"]["tests"]);
    assert_eq!(d["decision"]["execution_ids"], json!(["RUN-fixture-001"]));
    assert_eq!(d["decision"]["evidence_ids"], f.read(RUN)["evidence_ids"]);
    for key in [
        "disposition",
        "rationale",
        "reviewer",
        "reviewed_at",
        "coverage_assessment",
    ] {
        assert!(d["decision"][key].is_null());
    }
    assert_eq!(
        d["decision"]["evidence_assessment"][0]["requirement"],
        val["required_evidence"][0]
    );
    assert_eq!(
        d["decision"]["evidence_assessment"][0]["evidence_ids"],
        json!([])
    );
    assert!(runtime::validate_record(f.root(), "decision", &d["decision"]).is_err());
    assert_eq!(fs::read(f.root().join(VAL)).unwrap(), val_before);
    assert!(fs::read_dir(f.root().join("validation/decisions"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(
        registry::read_yaml(&f.root().join("evidence/draft.yaml")).unwrap(),
        d
    );
}
#[test]
fn draft_remains_editable_and_existing_output_is_never_replaced() {
    let f = Fixture::new();
    prepare(&f);
    let path = f.root().join("evidence/draft.yaml");
    let original = fs::read(&path).unwrap();
    assert!(!fs::metadata(&path).unwrap().permissions().readonly());
    assert!(review::prepare(f.root(), "VAL-012", "VDEC-002", &path).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::write(path, b"reviewer can edit this unapproved fixture").unwrap();
}
#[test]
fn preparation_preserves_failed_observations_and_does_not_select_evidence_for_a_conclusion() {
    let f = Fixture::new();
    let mut failed = f.read(RUN);
    failed["id"] = json!("RUN-fixture-002");
    failed["outcome"] = json!("failed");
    f.write("validation/executions/RUN-fixture-002.yaml", &failed);
    let mut foreign = failed.clone();
    foreign["id"] = json!("RUN-foreign-001");
    foreign["experiment"]["id"] = json!("EXP-999");
    f.write("validation/executions/RUN-foreign-001.yaml", &foreign);
    let d = prepare(&f);
    assert_eq!(
        d["decision"]["execution_ids"],
        json!(["RUN-fixture-001", "RUN-fixture-002"])
    );
    assert_eq!(d["decision"]["evidence_ids"], f.read(RUN)["evidence_ids"]);
    assert!(d["decision"]["disposition"].is_null());
    assert!(d["decision"]["evidence_assessment"][0]["evidence_ids"]
        .as_array()
        .unwrap()
        .is_empty());
}
#[test]
fn no_observations_prepare_an_empty_unapproved_package_at_head() {
    let f = Fixture::new();
    fs::remove_file(f.root().join(RUN)).unwrap();
    let d = prepare(&f);
    assert_eq!(d["decision"]["validation"]["registry_commit"], f.commit);
    assert_eq!(d["decision"]["execution_ids"], json!([]));
    assert_eq!(d["decision"]["evidence_ids"], json!([]));
    reject(&f, &d, "Draft review");
}
#[test]
fn a_single_matching_observation_boundary_is_reused_after_an_unrelated_commit() {
    let f = Fixture::new();
    f.bytes("unrelated.txt", b"unrelated committed fixture data");
    let head = f.commit_all();
    assert_ne!(head, f.commit);
    let d = prepare(&f);
    assert_eq!(d["decision"]["validation"]["registry_commit"], f.commit);
}
#[test]
fn mixed_observation_boundaries_keep_head_and_stay_unapproved() {
    let f = Fixture::new();
    f.bytes("unrelated.txt", b"new source boundary");
    let head = f.commit_all();
    let mut later = f.read(RUN);
    later["id"] = json!("RUN-fixture-002");
    later["experiment"]["registry_commit"] = json!(head);
    f.write("validation/executions/RUN-fixture-002.yaml", &later);
    let d = prepare(&f);
    assert_eq!(d["decision"]["validation"]["registry_commit"], head);
    assert_eq!(d["decision"]["execution_ids"].as_array().unwrap().len(), 2);
    assert_eq!(d["draft"], true);
}
#[test]
fn changed_definitions_keep_head_and_unavailable_pinned_revisions_fail() {
    let f = Fixture::new();
    f.edit(VAL, |v| v["objective"] = json!("Changed synthetic scope"));
    let head = f.commit_all();
    assert_eq!(
        prepare(&f)["decision"]["validation"]["registry_commit"],
        head
    );
    let f = Fixture::new();
    f.edit(VAL,|v|{v["revision"]=json!(4);v["revision_history"].as_array_mut().unwrap().push(json!({"revision":4,"date":"2026-10-15","actor":"fixture","reason":"Synthetic scope update"}));});
    f.commit_all();
    assert!(review::prepare(
        f.root(),
        "VAL-012",
        "VDEC-001",
        &f.root().join("draft.yaml")
    )
    .is_err());
    assert!(!f.root().join("draft.yaml").exists());
}
#[test]
fn draft_paths_inside_the_decision_registry_are_rejected_before_writing() {
    let f = Fixture::new();
    for relative in [
        "validation/decisions/VDEC-001.yaml",
        "validation/decisions/nested/draft.yaml",
        "validation/decisions",
    ] {
        assert!(
            review::prepare(f.root(), "VAL-012", "VDEC-001", &f.root().join(relative))
                .unwrap_err()
                .contains("outside the decision registry")
        );
    }
    assert!(fs::read_dir(f.root().join("validation/decisions"))
        .unwrap()
        .next()
        .is_none());
}
#[cfg(unix)]
#[test]
fn aliases_to_the_decision_registry_and_escaping_registry_symlinks_are_rejected() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    symlink(
        f.root().join("validation/decisions"),
        f.root().join("alias"),
    )
    .unwrap();
    assert!(review::prepare(
        f.root(),
        "VAL-012",
        "VDEC-001",
        &f.root().join("alias/new/draft.yaml")
    )
    .is_err());
    let f = Fixture::new();
    let external = tempfile::tempdir().unwrap();
    fs::remove_dir(f.root().join("validation/decisions")).unwrap();
    symlink(external.path(), f.root().join("validation/decisions")).unwrap();
    assert!(check(&f, &f.decision(), true).is_err());
    assert!(fs::read_dir(external.path()).unwrap().next().is_none());
}
#[test]
fn external_drafts_are_explicitly_allowed_and_identity_errors_do_not_create_files() {
    let f = Fixture::new();
    let external = tempfile::tempdir().unwrap();
    let path = external.path().join("review/draft.yaml");
    review::prepare(f.root(), "VAL-012", "VDEC-001", &path).unwrap();
    assert!(path.exists());
    for (val, decision) in [
        ("RUN-fixture-001", "VDEC-001"),
        ("VAL-999", "VDEC-001"),
        ("VAL-012", "../invalid"),
        ("VAL-012", "VDEC-abc"),
    ] {
        let path = external.path().join("invalid.yaml");
        assert!(review::prepare(f.root(), val, decision, &path).is_err());
        assert!(!path.exists());
    }
}
#[test]
fn bare_and_explicitly_reviewed_wrapped_checks_are_read_only() {
    let f = Fixture::new();
    let before = fs::read(f.root().join(VAL)).unwrap();
    let d = f.decision();
    assert_eq!(check(&f, &d, false).unwrap(), d);
    assert_eq!(
        check(&f, &json!({"draft":false,"decision":d}), false).unwrap(),
        d
    );
    assert!(fs::read_dir(f.root().join("validation/decisions"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(fs::read(f.root().join(VAL)).unwrap(), before);
}
#[test]
fn only_literal_false_clears_a_package_draft_marker() {
    let f = Fixture::new();
    let d = f.decision();
    for marker in [
        json!(true),
        json!("false"),
        json!(0),
        json!(null),
        json!([]),
    ] {
        reject(&f, &json!({"draft":marker,"decision":d}), "Draft review");
    }
    reject(&f, &json!({"decision":d}), "Draft review");
    let mut draft = prepare(&f);
    draft["draft"] = json!(false);
    assert!(check(&f, &draft, false).is_err());
    assert!(check(&f, &json!([]), false).is_err());
    assert!(check(&f, &json!({"draft":false,"decision":null}), false).is_err());
}
#[test]
fn explicit_registration_appends_only_the_supplied_decision() {
    let f = Fixture::new();
    let before = fs::read(f.root().join(VAL)).unwrap();
    let run_before = fs::read(f.root().join(RUN)).unwrap();
    let d = f.decision();
    check(&f, &d, true).unwrap();
    assert_eq!(f.read("validation/decisions/VDEC-001.yaml"), d);
    assert_eq!(fs::read(f.root().join(VAL)).unwrap(), before);
    assert_eq!(fs::read(f.root().join(RUN)).unwrap(), run_before);
    assert!(f.read(VAL)["current_decision_id"].is_null());
    assert_eq!(f.read(VAL)["lifecycle"]["state"], "reviewing");
    assert!(f.verify().is_empty());
    let bytes = fs::read(f.root().join("validation/decisions/VDEC-001.yaml")).unwrap();
    assert!(check(&f, &d, true)
        .unwrap_err()
        .contains("cannot be overwritten"));
    assert_eq!(
        fs::read(f.root().join("validation/decisions/VDEC-001.yaml")).unwrap(),
        bytes
    );
}
#[test]
fn registration_requires_reviewing_lifecycle_and_a_current_definition() {
    let f = Fixture::new();
    f.edit(VAL, |v| v["lifecycle"]["state"] = json!("running"));
    let d = f.decision();
    check(&f, &d, false).unwrap();
    reject(&f, &d, "reviewing state");
    let f = Fixture::new();
    f.edit(VAL, |v| v["objective"] = json!("Changed current scope"));
    reject(&f, &f.decision(), "current validation definition");
}
#[test]
fn substantive_review_identity_dates_and_required_assessments_are_enforced() {
    for kind in [
        "reviewer",
        "future",
        "early",
        "assessment",
        "envelope",
        "invalidating",
    ] {
        let f = Fixture::new();
        let mut d = f.decision();
        let expected = match kind {
            "reviewer" => {
                d["reviewer"] = json!("codex");
                "human reviewer"
            }
            "future" => {
                d["reviewed_at"] = json!("9999-01-01T00:00:00Z");
                "future"
            }
            "early" => {
                d["reviewed_at"] = json!("2019-12-31T00:00:00Z");
                "predates"
            }
            "assessment" => {
                d["evidence_assessment"] = json!([]);
                "explicit assessment"
            }
            "envelope" => {
                d["applicability"]["workload"] = json!("pending");
                "concrete rationale"
            }
            _ => {
                d["unresolved_findings"] = json!([{"severity":"high","description":"Synthetic invalidating gap","invalidates_conclusion":true}]);
                "invalidating finding"
            }
        };
        reject(&f, &d, expected);
    }
}
#[test]
fn partial_or_contradictory_observations_block_positive_registration() {
    let f = Fixture::new();
    f.edit(RUN, |r| {
        r["tests"][3]["outcome"] = json!("skipped");
        r["tests"][3]["exit_code"] = Value::Null;
        r["outcome"] = json!("incomplete");
    });
    reject(&f, &f.decision(), "passing observations");
    let f = Fixture::new();
    let mut failed = f.read(RUN);
    failed["id"] = json!("RUN-fixture-later");
    failed["started_at"] = json!("2020-01-01T00:02:00Z");
    failed["ended_at"] = json!("2020-01-01T00:02:01Z");
    failed["outcome"] = json!("failed");
    failed["tests"][0]["outcome"] = json!("failed");
    failed["tests"][0]["exit_code"] = json!(1);
    f.write("validation/executions/RUN-fixture-later.yaml", &failed);
    reject(&f, &f.decision(), "later contradictory");
}
#[test]
fn conditional_decisions_require_explicit_limits_and_follow_up() {
    let f = Fixture::new();
    let mut d = f.decision();
    d["disposition"] = json!("conditional");
    assert!(check(&f, &d, false).is_err());
    d["follow_up"] = json!([{"action":"Synthetic follow-up","owner":"Synthetic reviewer","review_condition":"Fixture recheck"}]);
    d["applicability"]["limits"] = json!([]);
    assert!(check(&f, &d, false).is_err());
    d["applicability"].as_object_mut().unwrap().remove("limits");
    d["applicability"]["limitations"] = json!([]);
    d["applicability"]["exclusions"] = json!([]);
    assert!(check(&f, &d, false)
        .unwrap_err()
        .contains("explicit limitation"));
    d["applicability"]["limitations"] = json!(["Synthetic fixture boundary"]);
    check(&f, &d, false).unwrap();
}
#[test]
fn negative_conclusions_retain_evidence_and_follow_up_without_claiming_a_pass() {
    for disposition in ["rejected", "inconclusive"] {
        let f = Fixture::new();
        f.edit(RUN, |r| {
            r["outcome"] = json!("failed");
            r["tests"][0]["outcome"] = json!("failed");
            r["tests"][0]["exit_code"] = json!(1);
        });
        let mut d = f.decision();
        d["disposition"] = json!(disposition);
        d["follow_up"] = json!([{"action":"Resolve synthetic gap","owner":"Synthetic fixture reviewer","review_condition":"New observation"}]);
        check(&f, &d, true).unwrap();
        assert_eq!(
            f.read("validation/decisions/VDEC-001.yaml")["disposition"],
            disposition
        );
    }
}
#[test]
fn registration_enforces_same_validation_supersession_and_rejects_cycles() {
    for kind in ["missing", "foreign", "cycle", "self"] {
        let f = Fixture::new();
        let mut prior = f.decision();
        prior["id"] = json!("VDEC-002");
        let mut candidate = f.decision();
        candidate["supersedes"] = json!(if kind == "self" {
            "VDEC-001"
        } else {
            "VDEC-002"
        });
        match kind {
            "foreign" => {
                prior["validation"]["id"] = json!("VAL-013");
                f.write("validation/decisions/VDEC-002.yaml", &prior);
            }
            "cycle" => {
                prior["supersedes"] = json!("VDEC-003");
                f.write("validation/decisions/VDEC-002.yaml", &prior);
                prior["id"] = json!("VDEC-003");
                prior["supersedes"] = json!("VDEC-002");
                f.write("validation/decisions/VDEC-003.yaml", &prior);
            }
            _ => (),
        };
        assert!(check(&f, &candidate, true).is_err(), "{kind}");
        assert!(!f.root().join("validation/decisions/VDEC-001.yaml").exists());
    }
    let f = Fixture::new();
    let prior = f.decision();
    f.write("validation/decisions/VDEC-001.yaml", &prior);
    let mut candidate = prior.clone();
    candidate["id"] = json!("VDEC-002");
    candidate["supersedes"] = json!("VDEC-001");
    check(&f, &candidate, true).unwrap();
    assert_eq!(f.read("validation/decisions/VDEC-002.yaml"), candidate);
    assert!(f.read(VAL)["current_decision_id"].is_null());
}
#[test]
fn explicit_artifact_mirrors_are_used_and_corrupt_content_cannot_register() {
    let f = Fixture::new();
    let mirror = tempfile::tempdir().unwrap();
    let mut mapping = BTreeMap::new();
    for n in 1..=4 {
        let relative = format!("validation/evidence-manifests/EVD-fixture-{n:03}.yaml");
        let mut e = f.read(&relative);
        let file = mirror.path().join(format!("item-{n}"));
        fs::copy(
            f.root()
                .join(format!("validation/artifacts/fixture-{n}.txt")),
            &file,
        )
        .unwrap();
        e["artifact"]["uri"] = json!(format!("https://fixture.invalid/item-{n}"));
        f.write(&relative, &e);
        mapping.insert(format!("EVD-fixture-{n:03}"), file);
    }
    let d = f.decision();
    assert!(check(&f, &d, false).is_err());
    let options = ArtifactOptions {
        mapping,
        root: None,
    };
    review::check(f.root(), &d, &options, false).unwrap();
    fs::write(options.mapping.values().next().unwrap(), b"tampered").unwrap();
    assert!(review::check(f.root(), &d, &options, true).is_err());
    assert!(!f.root().join("validation/decisions/VDEC-001.yaml").exists());
}
#[test]
fn cli_prepare_check_and_explicit_attestation_keep_selection_separate() {
    let f = Fixture::new();
    let output = f.root().join("evidence/cli-draft.yaml");
    let result = invoke(
        f.root(),
        &[
            "prepare",
            "--validation",
            "VAL-012",
            "--decision-id",
            "VDEC-001",
            "--output",
            output.to_str().unwrap(),
        ],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("unapproved"));
    let result = invoke(
        f.root(),
        &[
            "check",
            "--candidate",
            output.to_str().unwrap(),
            "--register-reviewed",
        ],
    );
    assert_eq!(result.status.code(), Some(1));
    let d = f.decision();
    f.write(
        "evidence/cli-reviewed.yaml",
        &json!({"draft":false,"decision":d}),
    );
    let path = f.root().join("evidence/cli-reviewed.yaml");
    let result = invoke(f.root(), &["check", "--candidate", path.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!f.root().join("validation/decisions/VDEC-001.yaml").exists());
    let result = invoke(
        f.root(),
        &[
            "check",
            "--candidate",
            path.to_str().unwrap(),
            "--register-reviewed",
        ],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(f.read("validation/decisions/VDEC-001.yaml"), d);
    assert!(f.read(VAL)["current_decision_id"].is_null());
}
#[test]
fn live_project_package_remains_unapproved_and_creates_no_project_record() {
    let output = tempfile::tempdir().unwrap();
    let before = runtime::load_records(&project()).unwrap();
    let d = review::prepare(
        &project(),
        "VAL-012",
        "VDEC-001",
        &output.path().join("draft.yaml"),
    )
    .unwrap();
    assert_eq!(d["draft"], true);
    assert!(d["decision"]["disposition"].is_null());
    assert!(review::check(&project(), &d, &ArtifactOptions::default(), false).is_err());
    assert_eq!(runtime::load_records(&project()).unwrap(), before);
}
