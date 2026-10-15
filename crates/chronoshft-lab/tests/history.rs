mod support;
use chronoshft_lab::{history, runtime::Git};
use serde_json::json;
use std::fs;
use support::*;

fn errors(f: &Fixture, base: &str) -> Vec<String> {
    history::check(f.root(), Some(base)).unwrap()
}
fn contains(f: &Fixture, base: &str, fragment: &str) {
    let errors = errors(f, base);
    assert!(
        errors.iter().any(|e| e.contains(fragment)),
        "expected {fragment}: {errors:?}"
    );
}
#[test]
fn live_history_and_unchanged_fixture_match_their_git_baselines() {
    assert!(history::check(&project(), Some("HEAD")).unwrap().is_empty());
    let f = Fixture::new();
    assert!(errors(&f, &f.commit).is_empty());
    assert!(history::check(f.root(), Some(&"0".repeat(40))).is_err());
}
#[test]
fn immutable_runs_evidence_and_artifacts_cannot_be_edited_or_removed() {
    let f = Fixture::new();
    let base = f.commit_all();
    f.edit(RUN, |r| r["tests"][0]["details"] = json!("changed"));
    contains(&f, &base, "immutable content changed");
    let f = Fixture::new();
    let base = f.commit_all();
    fs::remove_file(
        f.root()
            .join("validation/evidence-manifests/EVD-fixture-001.yaml"),
    )
    .unwrap();
    contains(&f, &base, "immutable content changed");
    let f = Fixture::new();
    let base = f.commit_all();
    f.bytes("validation/artifacts/fixture-1.txt", b"changed");
    contains(&f, &base, "immutable content changed");
}
#[test]
fn definition_edits_need_revision_snapshots_and_historical_prefixes() {
    let f = Fixture::new();
    f.edit(TEST, |t| t["title"] = json!("changed"));
    contains(&f, &f.commit, "definition changes require a new revision");
    let f = Fixture::new();
    let prior = f.read(TEST);
    f.edit(TEST,|t|{t["title"]=json!("revised");t["revision"]=json!(2);t["revision_history"].as_array_mut().unwrap().push(json!({"revision":2,"date":"2026-10-15","actor":"fixture","reason":"Synthetic revision"}));});
    contains(&f, &f.commit, "snapshot is missing");
    f.write("validation/revisions/tests/TST-001/r1.yaml", &prior);
    assert!(errors(&f, &f.commit).is_empty());
    f.edit("validation/revisions/tests/TST-001/r1.yaml", |t| {
        t["title"] = json!("fake prior")
    });
    contains(&f, &f.commit, "snapshot differs");
}
#[test]
fn origins_lifecycle_history_and_stable_ids_cannot_be_rewritten() {
    let f = Fixture::new();
    f.edit(VAL, |v| v["origin"]["finding"] = json!("rewritten"));
    contains(&f, &f.commit, "provenance was rewritten");
    let f = Fixture::new();
    f.edit(VAL, |v| {
        v["lifecycle"]["history"][0]["reason"] = json!("rewritten")
    });
    contains(&f, &f.commit, "lifecycle transitions were rewritten");
    let f = Fixture::new();
    fs::remove_file(f.root().join(TEST)).unwrap();
    contains(&f, &f.commit, "stable registry records cannot be removed");
}
#[test]
fn replacing_a_current_decision_requires_retaining_the_previous_identity() {
    let f = Fixture::new();
    f.write("validation/decisions/VDEC-001.yaml", &f.decision());
    f.edit(VAL, |v| v["current_decision_id"] = json!("VDEC-001"));
    let base = f.commit_all();
    f.edit(VAL, |v| v["current_decision_id"] = serde_json::Value::Null);
    contains(&f, &base, "previous decision must remain");
    f.edit(VAL, |v| v["prior_decision_ids"] = json!(["VDEC-001"]));
    assert!(errors(&f, &base).is_empty());
}
#[test]
fn automatic_baselines_follow_events_and_an_initial_event_is_explicit() {
    let f = Fixture::new();
    let git = Git::new(f.root()).unwrap();
    assert_eq!(
        history::automatic_base(&git, None, None).unwrap(),
        Some("HEAD".into())
    );
    let path = f.root().join("event.json");
    fs::write(
        &path,
        json!({"pull_request":{"base":{"sha":f.commit}}}).to_string(),
    )
    .unwrap();
    assert_eq!(
        history::automatic_base(&git, Some("pull_request"), Some(&path)).unwrap(),
        Some(f.commit.clone())
    );
    fs::write(&path, json!({"before":"0".repeat(40)}).to_string()).unwrap();
    assert_eq!(
        history::automatic_base(&git, Some("push"), Some(&path)).unwrap(),
        None
    );
    assert!(history::check(f.root(), None).unwrap().is_empty());
    fs::write(&path, "{}").unwrap();
    assert_eq!(
        history::automatic_base(&git, Some("workflow_dispatch"), Some(&path)).unwrap(),
        None
    );
    f.commit_all();
    assert_eq!(
        history::automatic_base(&git, Some("workflow_dispatch"), Some(&path)).unwrap(),
        Some(f.commit.clone())
    );
    assert!(history::automatic_base(&git, Some("pull_request"), Some(&path)).is_err());
}
