//! Final activation boundaries; historical sources stay available through Git.
use chronoshft_lab::{registry, runtime};
use serde_json::Value;
use std::{fs, path::PathBuf};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
#[test]
fn custom_python_is_retired_while_mkdocs_and_the_frozen_sources_remain() {
    let root = root();
    let m: Value = serde_json::from_slice(
        &fs::read(root.join("tests/compatibility/python-v1/manifest.json")).unwrap(),
    )
    .unwrap();
    let git = runtime::Git::new(&root).unwrap();
    for key in ["python_scripts", "python_test_modules"] {
        for entry in m[key].as_array().unwrap() {
            let path = entry
                .as_str()
                .unwrap_or_else(|| entry["path"].as_str().unwrap());
            assert!(!root.join(path).exists(), "retired entrypoint: {path}");
            assert!(!git
                .blob(m["reference_commit"].as_str().unwrap(), path)
                .unwrap()
                .is_empty());
        }
    }
    let deps = fs::read_to_string(root.join("pyproject.toml")).unwrap();
    for required in [
        "docs = [",
        "mkdocs>=",
        "mkdocs-material>=",
        "pymdown-extensions>=",
    ] {
        assert!(deps.contains(required));
    }
    assert!(!deps.contains("validation = [") && !deps.contains("jsonschema"));
    let lock = fs::read_to_string(root.join("uv.lock")).unwrap();
    assert!(!lock.contains("name = \"jsonschema\""));
    assert!(lock.contains("name = \"mkdocs\"") && lock.contains("name = \"pyyaml\""));
    assert!(!root.join("experiments/Cargo.toml").exists());
    assert!(!root.join("experiments/Cargo.lock").exists());
    let capture =
        fs::read_to_string(root.join(".github/workflows/validation-experiments.yml")).unwrap();
    assert!(!capture.contains("uv sync") && !capture.contains("Set up Python"));
}
#[test]
fn the_native_plan_matches_all_current_bindings_and_preserves_prior_definitions() {
    let root = root();
    let plan = registry::read_yaml(&root.join("validation/execution-plans/EXP-001.yaml")).unwrap();
    assert_eq!(plan["experiment"]["revision"], 4);
    assert_eq!(
        plan["dependency_lockfiles"],
        serde_json::json!(["Cargo.lock"])
    );
    let git = runtime::Git::new(&root).unwrap();
    let old = "2be4fa2af0f4cf50c94c3a6c7b0ff3e172e2a77c";
    for test in plan["tests"].as_array().unwrap() {
        let id = test["id"].as_str().unwrap();
        let current =
            registry::read_yaml(&root.join(format!("validation/register/tests/{id}.yaml")))
                .unwrap();
        assert_eq!(current["revision"], test["revision"]);
        assert_eq!(current["revision"], 3);
        let argv: Vec<_> = test["command"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(current["execution_binding"]["entrypoint"], argv.join(" "));
        assert_eq!(
            &argv[..9],
            [
                "cargo",
                "run",
                "--offline",
                "--locked",
                "-p",
                "chronoshft-lab",
                "--",
                "experiment",
                "run-case"
            ]
        );
        assert_eq!(argv.len(), 10);
        assert!(chronoshft_lab::experiment::CASES
            .iter()
            .any(|(case, test_id)| *case == argv[9] && *test_id == id));
        let snapshot = root.join(format!("validation/revisions/tests/{id}/r2.yaml"));
        assert_eq!(
            fs::read(snapshot).unwrap(),
            git.blob(old, &format!("validation/register/tests/{id}.yaml"))
                .unwrap()
        );
    }
}

#[test]
fn native_cutover_observation_is_clean_and_passes_exactly_the_four_bound_cases() {
    let root = root();
    let run =
        registry::read_yaml(&root.join("validation/executions/RUN-exp001-rust-001.yaml")).unwrap();
    assert_eq!(run["outcome"], "passed");
    assert_eq!(run["experiment"]["revision"], 4);
    assert_eq!(run["source"]["dirty"], false);
    assert_eq!(run["tests"].as_array().unwrap().len(), 4);
    assert_eq!(run["evidence_ids"].as_array().unwrap().len(), 16);
    for case in run["tests"].as_array().unwrap() {
        assert_eq!(case["outcome"], "passed");
        assert_eq!(case["exit_code"], 0);
        assert_eq!(case["test"]["revision"], 3);
        assert_eq!(case["command"][0], "cargo");
        assert_eq!(case["command"][8], "run-case");
    }
    let old =
        registry::read_yaml(&root.join("validation/executions/RUN-exp001-local-001.yaml")).unwrap();
    assert_eq!(old["tests"][0]["command"][0], "python3");
    assert_ne!(
        old["source"]["implementation_commit"],
        run["source"]["implementation_commit"]
    );
}
