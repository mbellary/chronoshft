//! Native YAML checks for executable CI and publication boundaries.
use std::{fs, path::PathBuf};
use yaml_rust2::{Yaml, YamlLoader};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn workflow(name: &str) -> Yaml {
    let docs = YamlLoader::load_from_str(
        &fs::read_to_string(root().join(format!(".github/workflows/{name}.yml"))).unwrap(),
    )
    .unwrap();
    assert_eq!(docs.len(), 1);
    docs[0].clone()
}
fn command(step: &Yaml) -> &str {
    step["run"].as_str().unwrap_or("")
}
fn guarded(step: &Yaml) {
    assert!(step["if"].is_badvalue());
    assert!(step["continue-on-error"].is_badvalue());
}
#[test]
fn every_gate_input_triggers_push_and_pull_request_validation() {
    let w = workflow("documentation");
    let required = [
        "experiments/**",
        "crates/**",
        ".cargo/**",
        "Cargo.toml",
        "Cargo.lock",
        "docs/**",
        "research/**",
        "validation/**",
        "tests/**",
        "mkdocs.yml",
        "pyproject.toml",
        "uv.lock",
        ".python-version",
        "README.md",
        ".github/workflows/documentation.yml",
        ".github/workflows/validation-experiments.yml",
    ];
    for event in ["push", "pull_request"] {
        let paths = w["on"][event]["paths"].as_vec().unwrap();
        for path in required {
            assert!(
                paths.iter().any(|v| v.as_str() == Some(path)),
                "missing {event}: {path}"
            );
        }
    }
}
#[test]
fn documentation_ci_uses_the_same_native_gate_locked_environment_and_reference_comparison() {
    let alias = fs::read_to_string(root().join(".cargo/config.toml")).unwrap();
    assert!(alias.contains("run --offline --locked -p chronoshft-lab --"));
    let w = workflow("documentation");
    let steps = w["jobs"]["build"]["steps"].as_vec().unwrap();
    let install = steps
        .iter()
        .find(|s| command(s).contains("uv sync"))
        .unwrap();
    for required in ["--locked", "--extra docs"] {
        assert!(command(install).contains(required));
    }
    let gate = steps
        .iter()
        .find(|s| command(s).contains("chronoshft-lab -- check"))
        .unwrap();
    guarded(gate);
    assert_eq!(
        command(gate),
        "cargo +stable run --offline --locked -p chronoshft-lab -- check"
    );
    assert!(steps.iter().all(|s| !command(s).contains("unittest")
        && !command(s).contains("scripts/")
        && !command(s).contains("--extra validation")));
    let lint = steps
        .iter()
        .find(|s| command(s).contains("cargo +stable clippy"))
        .unwrap();
    for required in [
        "rustup toolchain install stable",
        "cargo +stable fetch --locked",
        "cargo +stable fmt --all --check",
        "--offline --locked --workspace --all-targets -- -D warnings",
    ] {
        assert!(command(lint).contains(required));
    }
    let checkout = steps
        .iter()
        .find(|s| {
            s["uses"]
                .as_str()
                .is_some_and(|s| s.starts_with("actions/checkout@"))
        })
        .unwrap();
    assert_eq!(checkout["with"]["fetch-depth"].as_i64(), Some(0));
}
#[test]
fn capture_is_manual_native_committed_plan_only_and_retains_failure_artifacts() {
    let w = workflow("validation-experiments");
    let events = w["on"].as_hash().unwrap();
    assert_eq!(events.len(), 1);
    assert!(events.contains_key(&Yaml::String("workflow_dispatch".into())));
    assert_eq!(w["permissions"]["contents"].as_str(), Some("read"));
    assert_eq!(w["permissions"].as_hash().unwrap().len(), 1);
    let steps = w["jobs"]["capture"]["steps"].as_vec().unwrap();
    let capture = steps
        .iter()
        .find(|s| command(s).contains("chronoshft-lab -- execution capture"))
        .unwrap();
    guarded(capture);
    for arg in [
        "--offline --locked",
        "--committed-plan",
        "--shared-ci",
        "\"$VALIDATION_PLAN\"",
        "RUN-ci-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}",
    ] {
        assert!(command(capture).contains(arg));
    }
    assert!(!command(capture).contains("${{ inputs.plan }}"));
    assert_eq!(
        capture["env"]["VALIDATION_PLAN"].as_str(),
        Some("${{ inputs.plan }}")
    );
    let upload = steps
        .iter()
        .find(|s| {
            s["uses"]
                .as_str()
                .is_some_and(|s| s.starts_with("actions/upload-artifact@"))
        })
        .unwrap();
    assert_eq!(upload["if"].as_str(), Some("always()"));
    let path = upload["with"]["path"].as_str().unwrap();
    assert!(path.contains("evidence/") && path.contains("validation/artifacts/"));
    assert!(steps
        .iter()
        .any(|s| command(s).contains("cargo +stable fetch --locked")));
}
#[test]
fn pages_publication_follows_successful_gates_and_excludes_pull_requests() {
    let w = workflow("documentation");
    let build = &w["jobs"]["build"];
    assert!(build["continue-on-error"].is_badvalue());
    let steps = build["steps"].as_vec().unwrap();
    let gate = steps
        .iter()
        .position(|s| command(s).contains("chronoshft-lab -- check"))
        .unwrap();
    let upload = steps
        .iter()
        .position(|s| {
            s["uses"]
                .as_str()
                .is_some_and(|s| s.starts_with("actions/upload-pages-artifact@"))
        })
        .unwrap();
    assert!(gate < upload);
    let condition = steps[upload]["if"].as_str().unwrap();
    assert!(
        condition.contains("github.event_name != 'pull_request'")
            && condition.contains("refs/heads/main")
    );
    let deploy = &w["jobs"]["deploy"];
    assert_eq!(deploy["needs"].as_str(), Some("build"));
    assert!(deploy["if"]
        .as_str()
        .unwrap()
        .contains("github.event_name != 'pull_request'"));
    assert_eq!(w["permissions"]["contents"].as_str(), Some("read"));
    assert_eq!(w["permissions"].as_hash().unwrap().len(), 1);
    assert_eq!(deploy["permissions"]["pages"].as_str(), Some("write"));
    assert_eq!(deploy["permissions"]["id-token"].as_str(), Some("write"));
}
