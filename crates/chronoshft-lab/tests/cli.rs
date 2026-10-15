//! Public executable contract: stubs must never look like successful checks.
use std::process::Command;

fn invoke(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_chronoshft-lab"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_and_status_explain_authority_and_mkdocs_boundary() {
    for args in [&["--help"][..], &["migration", "status"][..]] {
        let output = invoke(args);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("Python"));
        assert!(text.contains("MkDocs"));
        assert!(text.contains("Stage 8") || text.contains("Stage 1:"));
    }
}

#[test]
fn unknown_incomplete_and_extra_status_arguments_are_usage_errors() {
    for args in [
        vec!["unknown"],
        vec!["registry"],
        vec!["migration", "status", "--force"],
        vec!["--version", "extra"],
    ] {
        let output = invoke(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn version_identifies_the_executable() {
    let output = invoke(&["--version"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        concat!("chronoshft-lab ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn output_errors_propagate_instead_of_returning_success() {
    let result = chronoshft_lab::cli::run(&["--help".into()], &mut FailingOutput, &mut Vec::new());
    assert!(result.is_err());
}

struct FailingOutput;
impl std::io::Write for FailingOutput {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "fixture output failure",
        ))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn registry_command_checks_real_data_and_rejects_bad_roots_or_options() {
    let output = invoke(&["registry", "check", "--require-tests"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("28 historical findings preserved"));
    assert_eq!(
        invoke(&[
            "registry",
            "check",
            "--root",
            "/nonexistent/chronoshft-fixture"
        ])
        .status
        .code(),
        Some(1)
    );
    for args in [
        vec!["registry", "check", "--root"],
        vec!["registry", "check", "--root", "--require-tests"],
        vec!["registry", "check", "--unknown"],
    ] {
        assert_eq!(invoke(&args).status.code(), Some(2));
    }
}

#[test]
fn history_and_evidence_commands_verify_live_data_without_changing_it() {
    for args in [&["history", "check"][..], &["evidence", "verify"][..]] {
        let output = invoke(args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
    }
    let output = invoke(&["evidence", "verify"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("No technical approval is inferred"));
}
#[test]
fn verification_commands_reject_bad_roots_missing_values_and_unknown_flags() {
    for operation in [["history", "check"], ["evidence", "verify"]] {
        assert_eq!(
            invoke(&[
                operation[0],
                operation[1],
                "--root",
                "/nonexistent/chronoshft-verifier"
            ])
            .status
            .code(),
            Some(1)
        );
        for flag in ["--root", "--unknown"] {
            assert_eq!(
                invoke(&[operation[0], operation[1], flag]).status.code(),
                Some(2)
            );
        }
    }
    assert_eq!(
        invoke(&[
            "history",
            "check",
            "--base-ref",
            "ffffffffffffffffffffffffffffffffffffffff"
        ])
        .status
        .code(),
        Some(1)
    );
    assert_eq!(
        invoke(&[
            "evidence",
            "verify",
            "--artifact-map",
            "/nonexistent/mirror.yaml"
        ])
        .status
        .code(),
        Some(1)
    );
}

#[test]
fn docs_commands_check_real_outputs_and_reject_invalid_arguments() {
    let output = invoke(&["docs", "generate", "--check"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Checked 39 validation pages"));
    for op in ["generate", "links"] {
        assert_eq!(
            invoke(&["docs", op, "--root", "/nonexistent/chronoshft-docs"])
                .status
                .code(),
            Some(1)
        );
        for flag in ["--root", "--unknown"] {
            assert_eq!(invoke(&["docs", op, flag]).status.code(), Some(2));
        }
        assert!(invoke(&["docs", op, "--help"]).status.success());
    }
    assert_eq!(invoke(&["docs", "links", "--check"]).status.code(), Some(2));
}

#[test]
fn execution_and_experiment_commands_have_explicit_usage_and_failure_codes() {
    for args in [
        vec!["execution", "capture"],
        vec!["execution", "capture", "--plan"],
        vec![
            "execution",
            "capture",
            "--plan",
            "--shared-ci",
            "later.yaml",
        ],
        vec!["execution", "import", "--run", "--resume", "later.yaml"],
        vec!["execution", "import"],
        vec!["execution", "import", "--run", "run.yaml"],
        vec!["experiment", "run-case"],
        vec!["experiment", "run-case", "unknown"],
        vec!["toolchain", "show", "--unknown"],
    ] {
        assert_eq!(invoke(&args).status.code(), Some(2), "{args:?}");
    }
    for args in [
        vec![
            "execution",
            "capture",
            "--plan",
            "/nonexistent/native-plan.yaml",
        ],
        vec![
            "execution",
            "import",
            "--run",
            "/nonexistent/native-run.yaml",
            "--evidence-dir",
            "/nonexistent/evidence",
        ],
    ] {
        assert_eq!(invoke(&args).status.code(), Some(1), "{args:?}");
    }
    for args in [
        vec!["execution", "capture", "--help"],
        vec!["execution", "import", "--help"],
        vec!["experiment", "run-case", "--help"],
        vec!["toolchain", "show", "--help"],
    ] {
        assert!(invoke(&args).status.success());
    }
}

#[test]
fn reviewer_commands_reject_bad_options_and_explain_explicit_attestation() {
    for args in [
        vec!["review", "prepare"],
        vec!["review", "check"],
        vec!["review", "check", "--register-reviewed"],
        vec![
            "review",
            "prepare",
            "--validation",
            "--output",
            "later.yaml",
        ],
        vec![
            "review",
            "check",
            "--candidate",
            "--register-reviewed",
            "later.yaml",
        ],
        vec!["review", "check", "--unknown"],
        vec!["review", "prepare", "--register-reviewed"],
    ] {
        assert_eq!(invoke(&args).status.code(), Some(2), "{args:?}");
    }
    assert_eq!(
        invoke(&[
            "review",
            "check",
            "--candidate",
            "/nonexistent/chronoshft-decision.yaml"
        ])
        .status
        .code(),
        Some(1)
    );
    for operation in ["prepare", "check"] {
        let output = invoke(&["review", operation, "--help"]);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
    }
    let output = invoke(&["review", "check", "--help"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("explicitly attests"));
}

#[test]
fn shared_gate_help_and_invalid_arguments_cannot_launch_a_build() {
    assert!(invoke(&["check", "--help"]).status.success());
    for args in [
        vec!["check", "--root"],
        vec!["check", "--unknown"],
        vec!["check", "--repair"],
        vec!["check", "--skip-tests"],
    ] {
        assert_eq!(invoke(&args).status.code(), Some(2));
    }
    let result = invoke(&["check", "--root", "/nonexistent/chronoshft-gate"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("gate root"));
}
