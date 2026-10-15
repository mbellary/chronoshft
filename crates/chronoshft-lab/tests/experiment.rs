//! Real Cargo builds/assertions in isolated copies of the EXP-001 prototype.
use chronoshft_lab::experiment::CASES;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn copy(source: &Path, target: &Path) {
    if source.is_dir() {
        fs::create_dir_all(target).unwrap();
        for e in fs::read_dir(source).unwrap() {
            let e = e.unwrap();
            if e.file_name() != "target" {
                copy(&e.path(), &target.join(e.file_name()));
            }
        }
    } else {
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(source, target).unwrap();
    }
}
fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    copy(&root().join("experiments"), &d.path().join("experiments"));
    fs::write(
        d.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"experiments/exp-001-spsc-queue\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(d.path())
        .output()
        .unwrap();
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    fs::create_dir_all(d.path().join("target/validation-reports")).unwrap();
    fs::write(
        d.path().join("target/validation-reports/TST-001.txt"),
        "STALE OBSERVATION",
    )
    .unwrap();
    d
}
fn run(d: &Path, case: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"))
        .args([
            "experiment",
            "run-case",
            case,
            "--root",
            d.to_str().unwrap(),
        ])
        .env_remove("CHRONOSHFT_REPORT_DIR")
        .env("CARGO_TARGET_DIR", d.join("target/build"))
        .output()
        .unwrap()
}
#[test]
fn every_declared_case_passes_and_produces_a_fresh_report() {
    let d = fixture();
    for (case, id) in CASES {
        let output = run(d.path(), case);
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        let report =
            fs::read_to_string(d.path().join(format!("target/validation-reports/{id}.txt")))
                .unwrap();
        assert!(report.contains("PASS:"));
        assert!(!report.contains("STALE"));
    }
}
#[test]
fn compiler_failure_is_tool_error_and_removes_the_stale_report() {
    let d = fixture();
    let path = d.path().join("experiments/exp-001-spsc-queue/src/lib.rs");
    let mut source = fs::read_to_string(&path).unwrap();
    source.push_str("\nintentional invalid Rust fixture\n");
    fs::write(path, source).unwrap();
    let output = run(d.path(), "fifo_exactly_once");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Tool error"));
    assert!(!d
        .path()
        .join("target/validation-reports/TST-001.txt")
        .exists());
}
#[test]
fn a_missing_exact_case_cannot_be_reported_as_an_empty_pass() {
    let d = fixture();
    let path = d
        .path()
        .join("experiments/exp-001-spsc-queue/tests/queue_contracts.rs");
    let source = fs::read_to_string(&path)
        .unwrap()
        .replace("fn fifo_exactly_once()", "fn renamed_fixture_case()");
    fs::write(path, source).unwrap();
    let output = run(d.path(), "fifo_exactly_once");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requested test is missing"));
    assert!(!d
        .path()
        .join("target/validation-reports/TST-001.txt")
        .exists());
}
#[test]
fn assertion_failure_keeps_a_fresh_report_and_returns_101() {
    let d = fixture();
    let path = d
        .path()
        .join("experiments/exp-001-spsc-queue/tests/queue_contracts.rs");
    let source = fs::read_to_string(&path).unwrap().replacen(
        "assert_eq!(producer.capacity(), capacity);",
        "assert_eq!(producer.capacity(), capacity + 1);",
        1,
    );
    fs::write(path, source).unwrap();
    let output = run(d.path(), "fifo_exactly_once");
    assert_eq!(output.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&output.stdout).contains("FAILED"));
    let report =
        fs::read_to_string(d.path().join("target/validation-reports/TST-001.txt")).unwrap();
    assert!(report.starts_with("TST-001:"));
    assert!(!report.contains("STALE"));
}
#[test]
fn toolchain_show_observes_actual_versions() {
    let output = Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"))
        .args(["toolchain", "show"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("rustc --version --verbose"));
    assert!(text.contains("cargo --version --verbose"));
    assert!(text.contains("host:"));
    assert!(text.contains("release:"));
}
