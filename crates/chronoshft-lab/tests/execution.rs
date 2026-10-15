#![cfg(unix)]
//! Real child processes in disposable Git repositories; no project observations.
mod support;
use chronoshft_lab::{
    capture, process, publication, registry,
    runtime::{self, ArtifactOptions},
};
use serde_json::{json, Value};
use std::{
    fs,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use support::Fixture;
struct CaptureFixture {
    f: Fixture,
    plan: Value,
}
impl CaptureFixture {
    fn new() -> Self {
        let mut f = Fixture::new();
        f.bytes(".gitignore", b"evidence/\ntarget/\n");
        f.bytes(
            "fixture.sh",
            br#"echo "synthetic native process $1 ${MODE:-passed}"
case "${MODE:-passed}" in
 failed) exit 1;;
 tool_error) exit 2;;
 timeout) sleep 10;;
 descendants) (sleep 0.5; echo leaked > evidence/leaked)& sleep 10;;
 background) (sleep 0.5; echo leaked > evidence/leaked)& exit 0;;
 mutate) echo changed > fixture.txt;;
 delete) rm fixture.txt;;
 report) mkdir -p target; echo fresh > target/report.txt;;
 signal) kill -TERM $$;;
esac
exit 0
"#,
        );
        for n in 1..=4 {
            f.edit(&format!("validation/register/tests/TST-{n:03}.yaml"), |t| {
                t["execution_binding"]["implementation_path"] = json!("fixture.sh");
                t["execution_binding"]["entrypoint"] =
                    json!(format!("/bin/sh fixture.sh TST-{n:03}"));
            });
        }
        f.commit = f.commit_all();
        let plan = json!({"schema_version":1,"run_id":"RUN-native-001","experiment":runtime::reference("EXP-001",&json!(2),&f.commit),"implementation_commit":f.commit,"working_directory":".","toolchain_command":["/bin/echo","Synthetic native fixture toolchain"],"configuration":"Synthetic native process test; no product acceptance","deviations":[],"inputs":[{"name":"fixture input","path":"fixture.txt"}],"dependency_lockfiles":["fixture.lock"],"random_seeds":["fixture-seed"],"instrumentation":"Native file-backed logs; synthetic fixture only","timeout_seconds":5,"retention":"repository","max_repository_artifact_bytes":100000,"environment_variables":{"MODE":"passed"},"tests":(1..=4).map(|n|json!({"id":format!("TST-{n:03}"),"revision":1,"command":["/bin/sh","fixture.sh",format!("TST-{n:03}")],"failure_exit_codes":[1],"artifacts":[]})).collect::<Vec<_>>()});
        Self { f, plan }
    }
    fn mode(&mut self, mode: &str) {
        self.plan["environment_variables"]["MODE"] = json!(mode);
    }
    fn only(&mut self, ids: &[usize]) {
        let original = self.plan["tests"].clone();
        self.plan["tests"] = json!(ids
            .iter()
            .map(|n| original[n - 1].clone())
            .collect::<Vec<_>>());
    }
    fn run(&self) -> Value {
        let run = capture::capture(self.f.root(), &self.plan, false).unwrap();
        let errors = self.f.verify();
        assert!(errors.is_empty(), "{errors:?}");
        run
    }
    fn manifests(&self, run: &Value) -> Vec<Value> {
        run["evidence_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| {
                self.f.read(&format!(
                    "validation/evidence-manifests/{}.yaml",
                    i.as_str().unwrap()
                ))
            })
            .collect()
    }
    fn bundle(&self) -> std::path::PathBuf {
        self.f.root().join(format!(
            "evidence/{}",
            self.plan["run_id"].as_str().unwrap()
        ))
    }
}
#[test]
fn passing_capture_accounts_for_all_tests_and_retains_reproduction_and_hashes() {
    let f = CaptureFixture::new();
    let run = f.run();
    assert_eq!(run["outcome"], "passed");
    assert_eq!(run["tests"].as_array().unwrap().len(), 4);
    assert!(run["environment"]["configuration"]
        .as_str()
        .unwrap()
        .contains("Process environment:"));
    assert!(run["environment"]["toolchain"]
        .as_str()
        .unwrap()
        .contains("Synthetic native fixture toolchain"));
    assert_eq!(run["source"]["implementation_commit"], f.f.commit);
    assert_eq!(
        run["source"]["dependency_lockfiles"][0]["sha256"],
        runtime::bytes_digest(b"synthetic lockfile\n")
    );
    assert_eq!(run["evidence_ids"].as_array().unwrap().len(), 12);
    assert_eq!(
        registry::read_yaml(&f.bundle().join("capture-state.json")).unwrap()["state"],
        "published"
    );
    assert!(f.bundle().join("results.json").is_file());
    assert!(f.bundle().join("manifests/RUN-native-001.yaml").is_file());
    assert!(run["tests"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["exit_code"] == 0));
}
#[test]
fn head_is_resolved_and_source_must_be_clean_before_reservation() {
    let mut f = CaptureFixture::new();
    f.plan["implementation_commit"] = json!("HEAD");
    f.plan["experiment"]["registry_commit"] = json!("HEAD");
    let run = f.run();
    assert_eq!(run["experiment"]["registry_commit"], f.f.commit);
    let f = CaptureFixture::new();
    f.f.bytes("uncommitted.txt", b"source");
    assert!(capture::capture(f.f.root(), &f.plan, false)
        .unwrap_err()
        .contains("clean committed"));
    assert!(!f.bundle().exists());
}
#[test]
fn failed_assertions_and_tool_exits_have_distinct_outcomes_and_retained_logs() {
    for (mode, expected) in [
        ("failed", "failed"),
        ("tool_error", "error"),
        ("signal", "error"),
    ] {
        let mut f = CaptureFixture::new();
        f.mode(mode);
        let run = f.run();
        assert_eq!(run["outcome"], expected);
        assert_eq!(run["tests"][0]["outcome"], expected);
        assert!(f.bundle().join("TST-001/stdout.log").is_file());
        if mode == "signal" {
            assert_eq!(run["tests"][0]["exit_code"], -15);
        }
    }
}
#[test]
fn omitted_and_reordered_selections_account_for_every_test_in_execution_order() {
    let mut f = CaptureFixture::new();
    f.only(&[4, 1]);
    let run = f.run();
    assert_eq!(run["outcome"], "incomplete");
    assert_eq!(run["tests"][1]["outcome"], "skipped");
    assert!(run["tests"][1]["exit_code"].is_null());
    assert_eq!(
        run["procedure"].as_array().unwrap()[2..],
        json!(["/bin/sh fixture.sh TST-001", "/bin/sh fixture.sh TST-004"])
            .as_array()
            .unwrap()[..]
    );
}
#[test]
fn repeated_bound_commands_remain_repeated_in_procedure() {
    let mut f = CaptureFixture::new();
    for n in 1..=2 {
        f.f.edit(&format!("validation/register/tests/TST-{n:03}.yaml"), |t| {
            t["execution_binding"]["entrypoint"] = json!("/bin/sh fixture.sh TST-001")
        });
        f.plan["tests"][n - 1]["command"] = json!(["/bin/sh", "fixture.sh", "TST-001"]);
    }
    f.f.commit = f.f.commit_all();
    f.plan["implementation_commit"] = json!(f.f.commit);
    f.plan["experiment"]["registry_commit"] = json!(f.f.commit);
    f.only(&[1, 2]);
    let run = f.run();
    assert_eq!(run["procedure"][2], run["procedure"][3]);
}
#[test]
fn timeout_kills_descendants_before_finalizing_and_retains_error() {
    let mut f = CaptureFixture::new();
    f.mode("descendants");
    f.only(&[1]);
    f.plan["timeout_seconds"] = json!(0.15);
    let run = f.run();
    // Time the supervised child, excluding Git/schema/artifact verification.
    // The fixture would sleep ten seconds if timeout supervision failed.
    assert!(run["tests"][0]["duration_seconds"].as_f64().unwrap() < 5.0);
    assert_eq!(run["outcome"], "error");
    assert!(run["tests"][0]["exit_code"].is_null());
    assert!(run["tests"][0]["details"]
        .as_str()
        .unwrap()
        .contains("timed out"));
    std::thread::sleep(Duration::from_millis(650));
    assert!(!f.f.root().join("evidence/leaked").exists());
}
#[test]
fn normal_parent_exit_also_cleans_up_background_descendants() {
    let mut f = CaptureFixture::new();
    f.mode("background");
    let run = f.run();
    assert_eq!(run["outcome"], "passed");
    std::thread::sleep(Duration::from_millis(650));
    assert!(!f.f.root().join("evidence/leaked").exists());
}
#[test]
fn source_mutation_or_deletion_stops_remaining_cases_and_blocks_success() {
    for mode in ["mutate", "delete"] {
        let mut f = CaptureFixture::new();
        f.mode(mode);
        let run = f.run();
        assert_eq!(run["outcome"], "error");
        assert_eq!(run["source"]["changed_during_run"], true);
        assert_eq!(run["tests"][1]["outcome"], "skipped");
    }
}
#[test]
fn cancellation_preserves_an_aborted_observation_and_no_more_cases_launch() {
    let mut f = CaptureFixture::new();
    f.mode("timeout");
    let flag = Arc::new(AtomicBool::new(false));
    let cancellation = flag.clone();
    let bundle = f.bundle();
    let worker = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !bundle.join("TST-001/stdout.log").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        cancellation.store(true, Ordering::SeqCst);
    });
    let run = capture::capture_with_interrupt(f.f.root(), &f.plan, false, &flag).unwrap();
    worker.join().unwrap();
    assert_eq!(run["outcome"], "aborted");
    assert_eq!(run["tests"][1]["outcome"], "skipped");
    assert!(f.f.verify().is_empty());
}
#[test]
fn missing_toolchain_probe_and_missing_outputs_are_registered_as_errors() {
    for kind in ["probe", "artifact"] {
        let mut f = CaptureFixture::new();
        if kind == "probe" {
            f.plan["toolchain_command"] = json!(["/nonexistent/chronoshft-tool"]);
        } else {
            f.plan["tests"][0]["artifacts"] =
                json!([{"path":"target/missing.txt","kind":"test_report"}]);
        }
        let run = f.run();
        assert_eq!(run["outcome"], "error");
        if kind == "probe" {
            assert!(run["tests"]
                .as_array()
                .unwrap()
                .iter()
                .all(|t| t["outcome"] == "skipped"));
        } else {
            assert!(run["tests"][0]["details"]
                .as_str()
                .unwrap()
                .contains("required artifact unavailable"));
        }
    }
}
#[test]
fn declared_outputs_are_retained_and_bundle_mirrors_verify() {
    let mut f = CaptureFixture::new();
    f.mode("report");
    f.only(&[1]);
    f.plan["retention"] = json!("bundle");
    f.plan["tests"][0]["artifacts"] = json!([{"path":"target/report.txt","kind":"test_report"}]);
    let run = f.run();
    let items = f.manifests(&run);
    let mirror = tempfile::tempdir().unwrap();
    let mut mapping = std::collections::BTreeMap::new();
    for item in &items {
        let src = f.f.root().join(
            item["artifact"]["uri"]
                .as_str()
                .unwrap()
                .trim_start_matches("repo://"),
        );
        let dest = mirror.path().join(item["id"].as_str().unwrap());
        fs::copy(src, &dest).unwrap();
        mapping.insert(item["id"].as_str().unwrap().to_owned(), dest);
    }
    let options = ArtifactOptions {
        mapping,
        root: None,
    };
    chronoshft_lab::evidence::verify_run(
        f.f.root(),
        &run,
        &mut f.f.resolver(),
        &f.f.records(),
        &options,
    )
    .unwrap();
    assert_eq!(run["tests"][0]["evidence_ids"].as_array().unwrap().len(), 3);
}
#[test]
fn plan_binding_revision_scope_and_path_errors_fail_before_reservation() {
    for kind in [
        "binding",
        "revision",
        "duplicate",
        "foreign",
        "path",
        "implementation",
        "manual",
        "unbound",
        "performance",
    ] {
        let mut f = CaptureFixture::new();
        match kind {
            "binding" => f.plan["tests"][0]["command"] = json!(["/bin/true"]),
            "revision" => f.plan["tests"][0]["revision"] = json!(9),
            "duplicate" => {
                let x = f.plan["tests"][0].clone();
                f.plan["tests"].as_array_mut().unwrap().push(x);
            }
            "foreign" => f.plan["tests"][0]["id"] = json!("TST-999"),
            "path" => {
                f.plan["tests"][0]["artifacts"] = json!([{"path":"../escape.txt","kind":"other"}])
            }
            "implementation" => {
                f.plan["implementation_commit"] = json!("ffffffffffffffffffffffffffffffffffffffff")
            }
            _ => {
                f.f.edit("validation/register/tests/TST-001.yaml", |t| match kind {
                    "manual" => t["execution_type"] = json!("manual"),
                    "unbound" => {
                        t["execution_binding"] =
                            json!({"state":"unbound","implementation_path":null,"entrypoint":null})
                    }
                    _ => t["classification"] = json!("performance"),
                });
                f.f.commit = f.f.commit_all();
                f.plan["implementation_commit"] = json!(f.f.commit);
                f.plan["experiment"]["registry_commit"] = json!(f.f.commit);
            }
        }
        assert!(
            capture::capture(f.f.root(), &f.plan, kind == "performance").is_err(),
            "{kind}"
        );
        assert!(!f.bundle().exists(), "{kind}");
    }
}
#[test]
fn committed_plan_guard_checks_location_bytes_and_records_override() {
    let mut f = CaptureFixture::new();
    f.plan["implementation_commit"] = json!("HEAD");
    f.plan["experiment"]["registry_commit"] = json!("HEAD");
    f.f.write("validation/execution-plans/native.yaml", &f.plan);
    f.f.commit = f.f.commit_all();
    let path = f.f.root().join("validation/execution-plans/native.yaml");
    let run =
        capture::from_file(f.f.root(), &path, Some("RUN-native-override"), true, false).unwrap();
    assert_eq!(run["id"], "RUN-native-override");
    assert_eq!(run["source"]["implementation_commit"], f.f.commit);
    f.f.write("external.yaml", &f.plan);
    assert!(capture::from_file(
        f.f.root(),
        &f.f.root().join("external.yaml"),
        None,
        true,
        false
    )
    .unwrap_err()
    .contains("committed plan"));
    f.plan["configuration"] = json!("changed");
    f.f.write("validation/execution-plans/native.yaml", &f.plan);
    assert!(capture::from_file(f.f.root(), &path, None, true, false)
        .unwrap_err()
        .contains("differs"));
}
#[test]
fn reused_run_ids_or_reserved_bundles_never_overwrite() {
    let f = CaptureFixture::new();
    let run = f.run();
    let before = fs::read(f.f.root().join("validation/executions/RUN-native-001.yaml")).unwrap();
    assert!(capture::capture(f.f.root(), &f.plan, false)
        .unwrap_err()
        .contains("cannot be reused"));
    assert_eq!(
        fs::read(f.f.root().join("validation/executions/RUN-native-001.yaml")).unwrap(),
        before
    );
    let items = f.manifests(&run);
    assert!(
        publication::publish_run(f.f.root(), &run, &items, &ArtifactOptions::default(), false)
            .is_err()
    );
    let f = CaptureFixture::new();
    fs::create_dir_all(f.bundle()).unwrap();
    assert!(capture::capture(f.f.root(), &f.plan, false)
        .unwrap_err()
        .contains("never reuse"));
}
#[test]
fn partial_publication_resumes_identical_content_only() {
    let f = CaptureFixture::new();
    let run = f.run();
    let evidence = f.manifests(&run);
    let run_path = f.f.root().join("validation/executions/RUN-native-001.yaml");
    fs::remove_file(&run_path).unwrap();
    let existing = f.f.root().join(format!(
        "validation/evidence-manifests/{}.yaml",
        evidence[0]["id"].as_str().unwrap()
    ));
    let before = fs::read(&existing).unwrap();
    let removed = f.f.root().join(format!(
        "validation/evidence-manifests/{}.yaml",
        evidence[1]["id"].as_str().unwrap()
    ));
    fs::remove_file(removed).unwrap();
    f.f.bytes(
        "validation/evidence-manifests/.publish-ABC123",
        b"orphaned incomplete staging bytes",
    );
    publication::publish_run(
        f.f.root(),
        &run,
        &evidence,
        &ArtifactOptions::default(),
        true,
    )
    .unwrap();
    assert_eq!(fs::read(existing).unwrap(), before);
    assert!(run_path.exists());
    let mut changed = run.clone();
    changed["instrumentation"] = json!("altered");
    assert!(publication::publish_run(
        f.f.root(),
        &changed,
        &evidence,
        &ArtifactOptions::default(),
        true
    )
    .unwrap_err()
    .contains("Resume cannot change"));
}
#[test]
fn import_validates_all_content_before_publishing_any_manifest() {
    let f = CaptureFixture::new();
    let run = f.run();
    let mut evidence = f.manifests(&run);
    fs::remove_file(f.f.root().join("validation/executions/RUN-native-001.yaml")).unwrap();
    for item in &evidence {
        fs::remove_file(f.f.root().join(format!(
            "validation/evidence-manifests/{}.yaml",
            item["id"].as_str().unwrap()
        )))
        .unwrap();
    }
    evidence[0]["artifact"]["sha256"] = json!("0".repeat(64));
    assert!(publication::publish_run(
        f.f.root(),
        &run,
        &evidence,
        &ArtifactOptions::default(),
        false
    )
    .is_err());
    assert!(!f
        .f
        .root()
        .join("validation/executions/RUN-native-001.yaml")
        .exists());
    assert!(!f
        .f
        .root()
        .join(format!(
            "validation/evidence-manifests/{}.yaml",
            evidence[1]["id"].as_str().unwrap()
        ))
        .exists());
}
#[test]
fn exclusive_writes_and_copies_do_not_replace_existing_content() {
    let d = tempfile::tempdir().unwrap();
    let target = d.path().join("immutable");
    publication::immutable_write(&target, b"first").unwrap();
    assert_eq!(
        publication::immutable_write(&target, b"second")
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::AlreadyExists
    );
    assert_eq!(
        publication::immutable_copy(&target, &target)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(&target).unwrap(), b"first");
    assert!(fs::read_dir(d.path()).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".publish-")));
}
#[test]
fn process_launch_error_is_retained_without_an_exit_code() {
    let d = tempfile::tempdir().unwrap();
    let result = process::run(
        &["/nonexistent/native-fixture".into()],
        d.path(),
        &d.path().join("stdout.log"),
        &d.path().join("stderr.log"),
        Duration::from_secs(1),
        &Default::default(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(result.error && !result.aborted);
    assert!(result.code.is_none());
    assert!(!fs::read(d.path().join("stderr.log")).unwrap().is_empty());
}
#[test]
fn posix_command_display_round_trips_quotes_empty_arguments_and_repeats() {
    let values = json!(["tool", "a b", "", "a'b", "é", "plain/path", "plain/path"]);
    let rendered = capture::shell_join(values.as_array().unwrap());
    assert_eq!(
        runtime::split_binding(&rendered).unwrap(),
        values
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
}
#[test]
fn actual_sigint_publishes_aborted_observation() {
    use nix::{
        sys::signal::{kill, Signal},
        unistd::Pid,
    };
    let mut f = CaptureFixture::new();
    f.mode("timeout");
    f.plan["implementation_commit"] = json!("HEAD");
    f.plan["experiment"]["registry_commit"] = json!("HEAD");
    f.f.write("validation/execution-plans/native.yaml", &f.plan);
    f.f.commit_all();
    let plan = f.f.root().join("validation/execution-plans/native.yaml");
    let mut child = Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"))
        .args([
            "execution",
            "capture",
            "--root",
            f.f.root().to_str().unwrap(),
            "--plan",
            plan.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while !f.bundle().join("TST-001/stdout.log").exists() {
        if let Some(status) = child.try_wait().unwrap() {
            panic!("capture exited before interruption: {status}");
        }
        assert!(start.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(5));
    }
    kill(Pid::from_raw(child.id() as i32), Signal::SIGINT).unwrap();
    assert_eq!(child.wait().unwrap().code(), Some(1));
    assert_eq!(
        f.f.read("validation/executions/RUN-native-001.yaml")["outcome"],
        "aborted"
    );
    assert!(f.f.verify().is_empty());
}
#[test]
fn probe_source_mutation_prevents_all_case_launches() {
    let mut f = CaptureFixture::new();
    f.plan["toolchain_command"] = json!([
        "/bin/sh",
        "-c",
        "printf changed > fixture.txt; printf 'fixture version'"
    ]);
    let run = f.run();
    assert_eq!(run["outcome"], "error");
    assert_eq!(run["source"]["changed_during_run"], true);
    assert!(run["tests"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["outcome"] == "skipped"));
}
#[test]
fn readonly_publication_is_exclusive_under_concurrent_writers() {
    let d = tempfile::tempdir().unwrap();
    let path = Arc::new(d.path().join("shared"));
    let barrier = Arc::new(std::sync::Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|n| {
            let path = path.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                publication::immutable_write(&path, format!("writer {n}").as_bytes()).is_ok()
            })
        })
        .collect();
    assert_eq!(
        workers
            .into_iter()
            .map(|worker| usize::from(worker.join().unwrap()))
            .sum::<usize>(),
        1
    );
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        fs::metadata(&*path).unwrap().permissions().mode() & 0o777,
        0o444
    );
}
#[test]
fn manual_staged_observation_imports_without_inventing_commands() {
    let f = Fixture::new();
    f.edit(support::EXP, |e| e["method"]["kind"] = json!("documentary"));
    for n in 1..=4 {
        f.edit(&format!("validation/register/tests/TST-{n:03}.yaml"), |t| {
            t["execution_type"] = json!("manual");
            t["execution_binding"]["implementation_path"] = Value::Null;
            t["execution_binding"]["entrypoint"] = json!("Synthetic manual checklist");
        });
    }
    let commit = f.commit_all();
    let mut run = f.read(support::RUN);
    run["experiment"]["registry_commit"] = json!(commit);
    run["source"]["registry_commit"] = json!(commit);
    run["source"]["implementation_commit"] = json!(commit);
    for t in run["tests"].as_array_mut().unwrap() {
        t["test"]["registry_commit"] = json!(commit);
        t["command"] = json!([]);
        t["exit_code"] = Value::Null;
    }
    let mut evidence = Vec::new();
    for n in 1..=4 {
        let path = format!("validation/evidence-manifests/EVD-fixture-{n:03}.yaml");
        let mut e = f.read(&path);
        e["supports"][0]["validation"]["registry_commit"] = json!(commit);
        e["provenance"]["source_version"] = json!(commit);
        evidence.push(e);
        fs::remove_file(f.root().join(path)).unwrap();
    }
    fs::remove_file(f.root().join(support::RUN)).unwrap();
    let stage = tempfile::tempdir().unwrap();
    fs::write(
        stage.path().join("run.yaml"),
        publication::yaml_text(&run).unwrap(),
    )
    .unwrap();
    fs::create_dir(stage.path().join("evidence")).unwrap();
    for e in &evidence {
        fs::write(
            stage
                .path()
                .join(format!("evidence/{}.yaml", e["id"].as_str().unwrap())),
            publication::yaml_text(e).unwrap(),
        )
        .unwrap();
    }
    publication::import(
        f.root(),
        &stage.path().join("run.yaml"),
        &stage.path().join("evidence"),
        &ArtifactOptions::default(),
        false,
    )
    .unwrap();
    assert!(f.verify().is_empty());
    assert!(f.read(support::RUN)["tests"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["exit_code"].is_null() && t["command"] == json!([])));
}
#[test]
fn escaping_bundle_or_output_symlinks_are_rejected_before_launch() {
    use std::os::unix::fs::symlink;
    for kind in ["bundle", "output"] {
        let mut f = CaptureFixture::new();
        let outside = tempfile::tempdir().unwrap();
        if kind == "bundle" {
            symlink(outside.path(), f.f.root().join("evidence")).unwrap();
        } else {
            fs::create_dir_all(f.f.root().join("target")).unwrap();
            symlink(outside.path(), f.f.root().join("target/escape")).unwrap();
            f.plan["tests"][0]["artifacts"] =
                json!([{"path":"target/escape/report.txt","kind":"other"}]);
        }
        assert!(capture::capture(f.f.root(), &f.plan, false).is_err());
        assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
    }
}
#[test]
fn existing_modified_content_addressed_artifact_is_not_replaced() {
    let f = CaptureFixture::new();
    let digest = runtime::bytes_digest(b"Synthetic native fixture toolchain\n");
    let path = format!("validation/artifacts/sha256/{digest}");
    f.f.bytes(&path, b"tampered");
    assert!(capture::capture(f.f.root(), &f.plan, false)
        .unwrap_err()
        .contains("differs from its digest"));
    assert_eq!(fs::read(f.f.root().join(path)).unwrap(), b"tampered");
    assert!(f.bundle().join("capture-state.json").exists());
    assert!(!f
        .f
        .root()
        .join("validation/executions/RUN-native-001.yaml")
        .exists());
}
fn copy_source(source: &std::path::Path, target: &std::path::Path) {
    if source.is_dir() {
        fs::create_dir_all(target).unwrap();
        for e in fs::read_dir(source).unwrap() {
            let e = e.unwrap();
            if e.file_name() != "target" {
                copy_source(&e.path(), &target.join(e.file_name()));
            }
        }
    } else {
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(source, target).unwrap();
    }
}
#[test]
fn fresh_exp001_capture_uses_only_native_rust_commands_and_verifies() {
    let mut f = CaptureFixture::new();
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "crates/chronoshft-lab/Cargo.toml",
        "crates/chronoshft-lab/src",
        "experiments",
    ] {
        copy_source(&support::project().join(path), &f.f.root().join(path));
    }
    let command = |case: &str| {
        json!([
            "cargo",
            "run",
            "--offline",
            "--locked",
            "-p",
            "chronoshft-lab",
            "--",
            "experiment",
            "run-case",
            case
        ])
    };
    f.f.edit(support::EXP,|e|{e["method"]["implementation_path"]=json!("experiments/exp-001-spsc-queue/src/lib.rs");e["objective"]=json!("Native migration smoke capture of the sequential EXP-001 prototype; no product acceptance");});
    for (index, (case, _)) in chronoshft_lab::experiment::CASES.iter().enumerate() {
        let n = index + 1;
        let command = command(case);
        f.f.edit(&format!("validation/register/tests/TST-{n:03}.yaml"), |t| {
            t["execution_binding"]["implementation_path"] =
                json!("experiments/exp-001-spsc-queue/tests/queue_contracts.rs");
            t["execution_binding"]["entrypoint"] =
                json!(capture::shell_join(command.as_array().unwrap()));
        });
        f.plan["tests"][index]["command"] = command;
        f.plan["tests"][index]["failure_exit_codes"] = json!([101]);
        f.plan["tests"][index]["artifacts"] = json!([{"path":format!("target/validation-reports/TST-{n:03}.txt"),"kind":"test_report"}]);
    }
    f.plan["run_id"] = json!("RUN-exp001-rust-stage5-001");
    f.plan["implementation_commit"] = json!("HEAD");
    f.plan["experiment"]["registry_commit"] = json!("HEAD");
    f.plan["toolchain_command"] = json!([
        "cargo",
        "run",
        "--offline",
        "--locked",
        "-p",
        "chronoshft-lab",
        "--",
        "toolchain",
        "show"
    ]);
    f.plan["dependency_lockfiles"] = json!(["Cargo.lock"]);
    f.plan["configuration"]=json!("Isolated native Rust migration smoke capture; four actual sequential EXP-001 cases, debug profile, offline locked Cargo; synthetic governance fixture; no performance or concurrency acceptance");
    f.plan["environment_variables"] =
        json!({"CHRONOSHFT_REPORT_DIR":"target/validation-reports","CARGO_TERM_COLOR":"never"});
    f.plan["timeout_seconds"] = json!(180);
    f.plan["max_repository_artifact_bytes"] = json!(2000000);
    for (name, path) in [
        (
            "prototype contract",
            "experiments/exp-001-spsc-queue/README.md",
        ),
        (
            "independent test model",
            "experiments/exp-001-spsc-queue/tests/queue_contracts.rs",
        ),
        (
            "native runner source",
            "crates/chronoshft-lab/src/experiment.rs",
        ),
    ] {
        f.plan["inputs"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":name,"path":path}));
    }
    f.f.write("validation/execution-plans/native-exp001.yaml", &f.plan);
    f.f.commit = f.f.commit_all();
    let run = capture::from_file(
        f.f.root(),
        &f.f.root()
            .join("validation/execution-plans/native-exp001.yaml"),
        None,
        true,
        false,
    )
    .unwrap();
    assert_eq!(
        run["outcome"],
        "passed",
        "{}",
        serde_json::to_string_pretty(&run).unwrap()
    );
    assert_eq!(run["tests"].as_array().unwrap().len(), 4);
    assert_eq!(run["evidence_ids"].as_array().unwrap().len(), 16);
    assert!(run["tests"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["command"][0] == "cargo" && t["exit_code"] == 0));
    assert!(f.f.verify().is_empty());
    for (_, id) in chronoshft_lab::experiment::CASES {
        assert!(fs::read_to_string(
            f.f.root()
                .join(format!("target/validation-reports/{id}.txt"))
        )
        .unwrap()
        .contains("PASS:"));
    }
    if std::env::var_os("CHRONOSHFT_RETAIN_NATIVE_SMOKE").is_some() {
        let path = f.f.directory.keep();
        println!("Retained native EXP-001 smoke checkout: {}", path.display());
    }
}
#[test]
fn cli_capture_and_resume_import_publish_and_verify_the_same_records() {
    let mut f = CaptureFixture::new();
    f.plan["implementation_commit"] = json!("HEAD");
    f.plan["experiment"]["registry_commit"] = json!("HEAD");
    f.f.write("validation/execution-plans/native.yaml", &f.plan);
    f.f.commit_all();
    let output = Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"))
        .args([
            "execution",
            "capture",
            "--root",
            f.f.root().to_str().unwrap(),
            "--plan",
            f.f.root()
                .join("validation/execution-plans/native.yaml")
                .to_str()
                .unwrap(),
            "--committed-plan",
        ])
        .env_remove("VALIDATION_ARTIFACT_MAP")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Recorded RUN-native-001: passed"));
    for resume in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"));
        command
            .args([
                "execution",
                "import",
                "--root",
                f.f.root().to_str().unwrap(),
                "--run",
                f.bundle()
                    .join("manifests/RUN-native-001.yaml")
                    .to_str()
                    .unwrap(),
                "--evidence-dir",
                f.bundle().join("manifests/evidence").to_str().unwrap(),
            ])
            .env_remove("VALIDATION_ARTIFACT_MAP");
        if resume {
            command.arg("--resume");
        }
        let output = command.output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(if resume { 0 } else { 1 }),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(f.f.verify().is_empty());
}
