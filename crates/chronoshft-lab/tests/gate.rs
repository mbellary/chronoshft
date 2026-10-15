//! Shared-gate sequence, fail-fast behavior and process boundary without recursion.
use chronoshft_lab::gate;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn fixture() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}
#[test]
fn plan_uses_native_checks_locked_regressions_strict_mkdocs_and_readonly_generation() {
    let d = fixture();
    let exe = Path::new("/selected/chronoshft-lab");
    let p = gate::plan(d.path(), exe);
    assert_eq!(p.len(), 7);
    for i in [0, 1, 2, 3, 6] {
        assert_eq!(p[i].argv[0], exe.to_str().unwrap());
        assert_eq!(
            &p[i].argv[p[i].argv.len() - 2..],
            &["--root", d.path().to_str().unwrap()]
        );
    }
    assert_eq!(&p[0].argv[1..4], &["registry", "check", "--require-tests"]);
    assert_eq!(&p[1].argv[1..3], &["history", "check"]);
    assert_eq!(&p[2].argv[1..3], &["evidence", "verify"]);
    assert_eq!(&p[3].argv[1..4], &["docs", "generate", "--check"]);
    assert_eq!(
        p[4].argv,
        vec!["cargo", "test", "--offline", "--locked", "--workspace"]
    );
    assert_eq!(
        p[5].argv,
        vec![
            "uv", "run", "--locked", "--extra", "docs", "python", "-m", "mkdocs", "build",
            "--strict"
        ]
    );
    assert_eq!(&p[6].argv[1..3], &["docs", "links"]);
    assert!(p.iter().all(|s| !s
        .argv
        .iter()
        .any(|a| a.ends_with(".py") || a == "capture" || a == "--register-reviewed")));
}
#[test]
fn successful_gate_runs_every_stage_from_the_selected_root() {
    let d = fixture();
    let mut seen = Vec::new();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = gate::run_with(
        d.path(),
        Path::new("/selected/lab"),
        &mut out,
        &mut err,
        |step, cwd| {
            assert_eq!(cwd, d.path().canonicalize().unwrap());
            seen.push(step.clone());
            Ok(0)
        },
    )
    .unwrap();
    assert_eq!(code, 0);
    assert_eq!(seen, gate::plan(d.path(), Path::new("/selected/lab")));
    assert!(err.is_empty());
    let output = String::from_utf8(out).unwrap();
    assert!(output.contains("[1/7]") && output.contains("[7/7]"));
    assert!(output.contains("acceptance are separate gates"));
}
#[test]
fn each_failed_stage_stops_every_later_stage_and_preserves_the_exit_code() {
    let d = fixture();
    for failed in 0..7 {
        let mut seen = 0;
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = gate::run_with(
            d.path(),
            Path::new("/selected/lab"),
            &mut out,
            &mut err,
            |_, _| {
                seen += 1;
                Ok(if seen == failed + 1 { 7 } else { 0 })
            },
        )
        .unwrap();
        assert_eq!(code, 7);
        assert_eq!(seen, failed + 1);
        assert!(String::from_utf8(err)
            .unwrap()
            .contains("remaining checks were not run"));
        assert!(!String::from_utf8(out).unwrap().contains("gates passed"));
    }
}
#[test]
fn launch_failure_fails_closed_before_later_work() {
    let d = fixture();
    for failed in 0..7 {
        let mut seen = 0;
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = gate::run_with(
            d.path(),
            Path::new("/selected/lab"),
            &mut out,
            &mut err,
            |_, _| {
                seen += 1;
                if seen == failed + 1 {
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "synthetic missing tool",
                    ))
                } else {
                    Ok(0)
                }
            },
        )
        .unwrap();
        assert_eq!(code, 1);
        assert_eq!(seen, failed + 1);
        assert!(String::from_utf8(err)
            .unwrap()
            .contains("could not start check"));
    }
}
struct BrokenWriter;
impl io::Write for BrokenWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "fixture"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn failed_progress_output_cannot_launch_work_or_report_success() {
    let d = fixture();
    let mut seen = 0;
    assert!(gate::run_with(
        d.path(),
        Path::new("/selected/lab"),
        &mut BrokenWriter,
        &mut Vec::new(),
        |_, _| {
            seen += 1;
            Ok(0)
        }
    )
    .is_err());
    assert_eq!(seen, 0);
}
#[test]
fn invalid_roots_never_invoke_any_stage() {
    let d = fixture();
    let file = d.path().join("file");
    fs::write(&file, b"fixture").unwrap();
    for path in [file, d.path().join("missing")] {
        let mut seen = 0;
        let result = gate::run_with(
            &path,
            Path::new("/selected/lab"),
            &mut Vec::new(),
            &mut Vec::new(),
            |_, _| {
                seen += 1;
                Ok(0)
            },
        )
        .unwrap();
        assert_eq!(result, 1);
        assert_eq!(seen, 0);
    }
}
#[test]
fn real_cli_invalid_registry_stops_before_tests_or_mkdocs() {
    let d = fixture();
    fs::write(d.path().join("mkdocs.yml"), "site_name: Fixture\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"))
        .args(["check", "--root", d.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("[1/7]"));
    assert!(!stdout.contains("[2/7]"));
    assert!(!d.path().join("site").exists());
}
#[test]
fn authored_local_instructions_select_the_shared_native_gate() {
    let readme = fs::read_to_string(root().join("README.md")).unwrap();
    let automation =
        fs::read_to_string(root().join("docs/10-validation-lab/automation-and-ci.md")).unwrap();
    assert!(readme.contains("cargo lab check"));
    assert!(automation.contains("cargo lab check"));
    assert!(automation.contains("MkDocs"));
}
#[cfg(unix)]
#[test]
fn real_cli_stops_on_a_failed_regression_process_before_mkdocs() {
    use std::os::unix::fs::PermissionsExt;
    let d = fixture();
    let cargo = d.path().join("cargo");
    let uv = d.path().join("uv");
    fs::write(&cargo, "#!/bin/sh\nexit 17\n").unwrap();
    fs::write(&uv, "#!/bin/sh\nexit 19\n").unwrap();
    for path in [&cargo, &uv] {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut paths = vec![d.path().to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let result = Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"))
        .args(["check", "--root", root().to_str().unwrap()])
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env_remove("VALIDATION_ARTIFACT_MAP")
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(17),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("[5/7]"));
    assert!(!stdout.contains("[6/7]"));
    assert!(!stdout.contains("[7/7]"));
}
