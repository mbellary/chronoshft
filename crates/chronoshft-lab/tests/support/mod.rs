#![allow(dead_code)]
//! Isolated synthetic source/observations for verifier tests; never project ledgers.
use chronoshft_lab::{
    registry,
    runtime::{self, ArtifactOptions, PinnedDefinitions},
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub const VAL: &str = "validation/register/validations/VAL-012.yaml";
pub const EXP: &str = "validation/register/experiments/EXP-001.yaml";
pub const TEST: &str = "validation/register/tests/TST-001.yaml";
pub const RUN: &str = "validation/executions/RUN-fixture-001.yaml";
pub fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
pub struct Fixture {
    pub directory: TempDir,
    pub commit: String,
}
impl Fixture {
    pub fn root(&self) -> &Path {
        self.directory.path()
    }
    pub fn read(&self, path: &str) -> Value {
        registry::read_yaml(&self.root().join(path)).unwrap()
    }
    pub fn write(&self, path: &str, value: &Value) {
        self.bytes(path, serde_json::to_vec_pretty(value).unwrap().as_slice());
    }
    pub fn bytes(&self, path: &str, bytes: &[u8]) {
        let path = self.root().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    pub fn edit(&self, path: &str, change: impl FnOnce(&mut Value)) {
        let mut value = self.read(path);
        change(&mut value);
        self.write(path, &value);
    }
    pub fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(self.root())
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    pub fn commit_all(&self) -> String {
        self.git(&["add", "."]);
        self.git(&["commit", "-qm", "Synthetic verifier fixture only"]);
        self.git(&["rev-parse", "HEAD"])
    }
    pub fn resolver(&self) -> PinnedDefinitions {
        PinnedDefinitions::new(self.root()).unwrap()
    }
    pub fn records(&self) -> registry::Records {
        runtime::load_records(self.root()).unwrap()
    }
    pub fn verify(&self) -> Vec<String> {
        chronoshft_lab::evidence::verify_all(self.root(), &ArtifactOptions::default())
            .unwrap()
            .errors
    }
    pub fn new() -> Self {
        let mut f = Self {
            directory: tempfile::tempdir().unwrap(),
            commit: String::new(),
        };
        for entry in fs::read_dir(project().join("validation/schemas")).unwrap() {
            let entry = entry.unwrap();
            f.bytes(
                &format!("validation/schemas/{}", entry.file_name().to_str().unwrap()),
                &fs::read(entry.path()).unwrap(),
            );
        }
        for directory in [
            "register/validations",
            "register/experiments",
            "register/tests",
            "executions",
            "evidence-manifests",
            "decisions",
            "execution-plans",
        ] {
            fs::create_dir_all(f.root().join("validation").join(directory)).unwrap();
        }
        let load = |path: &str| registry::read_yaml(&project().join(path)).unwrap();
        let mut val = load(VAL);
        val["revision"] = json!(3);
        val["revision_history"].as_array_mut().unwrap().truncate(3);
        val["objective"] = json!("Synthetic verifier fixture, not project acceptance");
        val["required_evidence"] = json!(["Synthetic observation coverage"]);
        val["coverage"]["experiments"] = json!([{"id":"EXP-001","revision":2,"mandatory":true}]);
        val["coverage"]["tests"] = json!((1..=4)
            .map(|n| json!({"id":format!("TST-{n:03}"),"revision":1,"mandatory":true}))
            .collect::<Vec<_>>());
        let mut previous = "planned";
        for state in ["ready", "running", "reviewing"] {
            val["lifecycle"]["history"].as_array_mut().unwrap().push(json!({"from":previous,"to":state,"date":"2026-10-15","actor":"fixture","reason":"Synthetic only"}));
            previous = state;
        }
        val["lifecycle"]["state"] = json!("reviewing");
        f.write(VAL, &val);
        let mut exp = load(EXP);
        exp["revision"] = json!(2);
        exp["revision_history"].as_array_mut().unwrap().truncate(2);
        exp["validates"] = json!([{"id":"VAL-012","revision":3}]);
        exp["tests"] = json!((1..=4)
            .map(|n| json!({"id":format!("TST-{n:03}"),"revision":1}))
            .collect::<Vec<_>>());
        exp["method"]["implementation_path"] = json!("fixture.rs");
        exp["lifecycle"]["state"] = json!("ready");
        exp["lifecycle"]["history"]
            .as_array_mut()
            .unwrap()
            .truncate(2);
        f.write(EXP, &exp);
        for n in 1..=4 {
            let path = format!("validation/register/tests/TST-{n:03}.yaml");
            let mut t = load(&path);
            t["revision"] = json!(1);
            t["revision_history"].as_array_mut().unwrap().truncate(1);
            t["experiment"] = json!({"id":"EXP-001","revision":2});
            t["validations"] = json!([{"id":"VAL-012","revision":3}]);
            t["execution_binding"] = json!({"state":"bound","implementation_path":"fixture.rs","entrypoint":format!("fixture-tool 'event token' TST-{n:03}")});
            f.write(&path, &t);
        }
        f.bytes(
            "fixture.rs",
            b"// Synthetic verifier source only, no experiment executed.\n",
        );
        f.bytes("fixture.txt", b"synthetic input\n");
        f.bytes("fixture.lock", b"synthetic lockfile\n");
        f.git(&["init", "-q"]);
        f.git(&["config", "user.name", "Synthetic Fixture"]);
        f.git(&["config", "user.email", "fixture@example.invalid"]);
        f.commit = f.commit_all();
        let mut run = load("validation/executions/RUN-exp001-local-001.yaml");
        run["id"] = json!("RUN-fixture-001");
        run["procedure"] = json!(["Synthetic verifier fixture; no experiment executed"]);
        run["instrumentation"] = json!("Synthetic JSON verifier fixture only");
        run["started_at"] = json!("2020-01-01T00:00:00Z");
        run["ended_at"] = json!("2020-01-01T00:00:01Z");
        run["experiment"] = runtime::reference("EXP-001", &json!(2), &f.commit);
        run["source"]["implementation_commit"] = json!(f.commit);
        run["source"]["registry_commit"] = json!(f.commit);
        run["source"]["dependency_lockfiles"] = json!([{"path":"fixture.lock","sha256":runtime::bytes_digest(b"synthetic lockfile\n")}]);
        run["inputs"] = json!([{"name":"synthetic","reference":"repo://fixture.txt","sha256":runtime::bytes_digest(b"synthetic input\n")}]);
        run["environment"] = json!({"toolchain":"Synthetic toolchain","os":"Synthetic OS","cpu":"Synthetic CPU","configuration":"Synthetic context","deviations":[]});
        run["random_seeds"] = json!(["synthetic-seed"]);
        let mut evidence_ids = Vec::new();
        for n in 1..=4 {
            let id = format!("EVD-fixture-{n:03}");
            let path = format!("validation/artifacts/fixture-{n}.txt");
            let bytes = format!("Synthetic observation {n}, not project evidence.\n").into_bytes();
            f.bytes(&path, &bytes);
            let mut e = load("validation/evidence-manifests/EVD-exp001-local-001-001.yaml");
            e["id"] = json!(id);
            e["provenance"]["execution_id"] = json!("RUN-fixture-001");
            e["provenance"]["source_version"] = json!(f.commit);
            e["provenance"]["description"] = json!("Synthetic verifier fixture only");
            e["provenance"]["retrieved_at"] = json!("2020-01-01T00:00:01Z");
            e["artifact"]["uri"] = json!(format!("repo://{path}"));
            e["artifact"]["sha256"] = json!(runtime::bytes_digest(&bytes));
            e["artifact"]["size_bytes"] = json!(bytes.len());
            e["supports"] = json!([{"validation":runtime::reference("VAL-012",&json!(3),&f.commit),"scope":"Synthetic fixture observation"}]);
            f.write(&format!("validation/evidence-manifests/{id}.yaml"), &e);
            evidence_ids.push(id.clone());
            let case = &mut run["tests"][n - 1];
            case["test"] = runtime::reference(&format!("TST-{n:03}"), &json!(1), &f.commit);
            case["command"] = json!(["fixture-tool", "event token", format!("TST-{n:03}")]);
            case["evidence_ids"] = json!([id]);
        }
        run["evidence_ids"] = json!(evidence_ids);
        f.write(RUN, &run);
        f
    }
    pub fn decision(&self) -> Value {
        let run = self.read(RUN);
        json!({"schema_version":1,"id":"VDEC-001","validation":runtime::reference("VAL-012",&json!(3),&self.commit),"disposition":"accepted","rationale":"Synthetic fixture attestation only","execution_ids":["RUN-fixture-001"],"evidence_ids":run["evidence_ids"],"coverage_assessment":"All synthetic cases","evidence_assessment":[{"requirement":"Synthetic observation coverage","evidence_ids":run["evidence_ids"],"rationale":"Synthetic coverage only"}],"unresolved_findings":[],"applicability":{"implementation_commit":self.commit,"workload":"Synthetic workload","hardware":"Synthetic hardware","dependencies":"Synthetic fixture dependencies","exclusions":["No product conclusion"],"limitations":["Verifier unit fixture only"]},"reviewer":"Synthetic Human Reviewer","reviewed_at":"2020-01-01T00:01:00Z","architecture_spec_actions":["No production adoption"],"follow_up":[],"supersedes":null})
    }
}
