//! Frozen Python expectations and independent adversarial registry fixtures.
use chronoshft_lab::registry::{self, Options, Report};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const VAL: &str = "validation/register/validations/VAL-012.yaml";
const EXP: &str = "validation/register/experiments/EXP-001.yaml";
const TEST: &str = "validation/register/tests/TST-001.yaml";
const RUN: &str = "validation/executions/RUN-exp001-local-001.yaml";
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
struct Fixture(TempDir);
impl Fixture {
    fn live() -> Self {
        fn copy(source: &Path, target: &Path) {
            if source.is_dir() {
                fs::create_dir_all(target).unwrap();
                for entry in fs::read_dir(source).unwrap() {
                    let entry = entry.unwrap();
                    copy(&entry.path(), &target.join(entry.file_name()));
                }
            } else {
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::write(target, fs::read(source).unwrap()).unwrap();
            }
        }
        let temp = tempfile::tempdir().unwrap();
        for relative in [
            "validation",
            "docs/00-governance/technical-validation.md",
            "docs/assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md",
        ] {
            copy(&root().join(relative), &temp.path().join(relative));
        }
        Self(temp)
    }
    fn pinned(manifest: &Value) -> Self {
        let temp = tempfile::tempdir().unwrap();
        for input in manifest["reference_inputs"].as_array().unwrap() {
            let relative = input["path"].as_str().unwrap();
            if !relative.starts_with("validation/") && !relative.starts_with("docs/") {
                continue;
            }
            let output = Command::new("git")
                .arg("-C")
                .arg(root())
                .args([
                    "show",
                    &format!(
                        "{}:{relative}",
                        manifest["reference_commit"].as_str().unwrap()
                    ),
                ])
                .output()
                .unwrap();
            assert!(output.status.success(), "missing pinned input {relative}");
            assert_eq!(
                format!("{:x}", Sha256::digest(&output.stdout)),
                input["sha256"]
            );
            let path = temp.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, output.stdout).unwrap();
        }
        Self(temp)
    }
    fn edit(&self, relative: &str, change: impl FnOnce(&mut Value)) {
        let path = self.0.path().join(relative);
        let mut value = registry::read_yaml(&path).unwrap();
        change(&mut value);
        fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    }
    fn check(&self) -> Report {
        registry::validate(
            self.0.path(),
            Options {
                require_tests: true,
                require_coverage: true,
            },
        )
    }
    fn reject(&self, fragment: &str) {
        let report = self.check();
        assert!(!report.accepted(), "unexpected acceptance");
        assert!(
            report.errors.iter().any(|e| e.contains(fragment)),
            "missing {fragment}: {:?}",
            report.errors
        );
    }
}

#[test]
fn frozen_python_reference_and_all_ten_registry_overlays_match() {
    let folder = root().join("tests/compatibility/python-v1");
    let manifest: Value =
        serde_json::from_slice(&fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    let fixture = Fixture::pinned(&manifest);
    let report = fixture.check();
    assert!(report.accepted(), "{:?}", report.errors);
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    for id in report.records.keys() {
        *counts.entry(id.split('-').next().unwrap()).or_default() += 1;
    }
    assert_eq!(
        serde_json::to_value(counts).unwrap(),
        manifest["record_counts"]
    );
    for case in manifest["registry_cases"].as_array().unwrap() {
        let path = fixture.0.path().join(case["target"].as_str().unwrap());
        let old = fs::read(&path).unwrap();
        let content = fs::read(folder.join(case["fixture"].as_str().unwrap())).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&content)), case["sha256"]);
        fs::write(&path, content).unwrap();
        let report = fixture.check();
        fs::write(path, old).unwrap();
        assert_eq!(
            report.accepted(),
            case["accepted"].as_bool().unwrap(),
            "{}: {:?}",
            case["name"],
            report.errors
        );
        if let Some(fragment) = case["diagnostic_contains"].as_str() {
            assert!(
                report.errors.iter().any(|e| e.contains(fragment)),
                "{}: {:?}",
                case["name"],
                report.errors
            );
        }
    }
}

#[test]
fn historical_commit_references_keep_old_revisions_without_becoming_planning_references() {
    let f = Fixture::live();
    f.edit(RUN, |r| {
        r["experiment"]["revision"] = json!(1);
        r["tests"][0]["test"]["revision"] = json!(1);
    });
    let report = f.check();
    assert!(report.accepted(), "{:?}", report.errors);
    // Stage 3 will resolve those historical contents. Current planning must match now.
    f.edit(TEST, |t| t["experiment"]["revision"] = json!(1));
    f.reject("stale planning reference");
}

#[test]
fn research_provenance_and_original_source_are_preserved() {
    let f = Fixture::live();
    f.edit(VAL, |v| v["origin"]["finding"] = json!("proven"));
    f.reject("origin differs");
    let f = Fixture::live();
    let path =
        f.0.path()
            .join("docs/assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md");
    fs::write(path, "changed").unwrap();
    f.reject("source document hash differs");
    let f = Fixture::live();
    let path =
        f.0.path()
            .join("docs/00-governance/technical-validation.md");
    let text = fs::read_to_string(&path).unwrap().replace(
        "Writer appends before full-ring check",
        "Altered historical finding",
    );
    fs::write(path, text).unwrap();
    f.reject("historical research table differs");
}

#[test]
fn missing_snapshot_and_changed_snapshot_history_are_rejected() {
    let f = Fixture::live();
    fs::remove_file(
        f.0.path()
            .join("validation/revisions/tests/TST-001/r1.yaml"),
    )
    .unwrap();
    f.reject("missing prior revision snapshot");
    let f = Fixture::live();
    f.edit("validation/revisions/validations/VAL-012/r1.yaml", |v| {
        v["origin"]["finding"] = json!("rewritten")
    });
    f.reject("prior revision snapshot changed");
    let f = Fixture::live();
    f.edit("validation/revisions/experiments/EXP-001/r1.yaml", |v| {
        v["revision_history"][0]["reason"] = json!("rewritten")
    });
    f.reject("prior revision history differs");
}

#[test]
fn lifecycle_transition_continuity_dates_and_current_state_are_enforced() {
    let f = Fixture::live();
    f.edit(VAL, |v| v["lifecycle"]["state"] = json!("ready"));
    f.reject("lifecycle state disagrees");
    let f = Fixture::live();
    f.edit(VAL, |v| {
        v["lifecycle"]["history"][1]["to"] = json!("running");
        v["lifecycle"]["state"] = json!("running");
    });
    f.reject("illegal lifecycle transition");
    let f = Fixture::live();
    f.edit(EXP, |v| {
        v["lifecycle"]["history"][1]["from"] = json!("running")
    });
    f.reject("discontinuous");
    let f = Fixture::live();
    f.edit(EXP, |v| {
        v["lifecycle"]["history"][1]["date"] = json!("2026-10-14")
    });
    f.reject("dates are out of order");
}

#[test]
fn relationships_must_be_reciprocal_and_obligations_agree() {
    let f = Fixture::live();
    f.edit(VAL, |v| {
        v["coverage"]["experiments"]
            .as_array_mut()
            .unwrap()
            .remove(0);
    });
    f.reject("experiment/validation coverage is not reciprocal");
    let f = Fixture::live();
    f.edit(VAL, |v| {
        v["coverage"]["tests"][0]["mandatory"] = json!(false)
    });
    f.reject("mandatory obligation disagrees");
    let f = Fixture::live();
    f.edit(TEST, |v| v["hypothesis_ids"] = json!(["HYP-999"]));
    f.reject("unknown hypothesis");
    let f = Fixture::live();
    f.edit(EXP, |v| {
        let mut duplicate = v["tests"][0].clone();
        duplicate["revision"] = json!(1);
        v["tests"].as_array_mut().unwrap().push(duplicate);
    });
    f.reject("duplicate relationship ID");
}

#[test]
fn readiness_requires_bound_methods() {
    let f = Fixture::live();
    f.edit(
        TEST,
        |v| {
            v["execution_binding"] =
                json!({"state":"unbound","implementation_path":null,"entrypoint":null})
        },
    );
    f.reject("cannot advance with unbound mandatory tests");
}

#[test]
fn all_hypotheses_methods_and_validation_obligations_need_mandatory_coverage() {
    let f = Fixture::live();
    for n in 1..=4 {
        f.edit(&format!("validation/register/tests/TST-{n:03}.yaml"), |v| {
            v["hypothesis_ids"] = json!(["HYP-001"])
        });
    }
    f.reject("every hypothesis requires a mandatory test");
    let f = Fixture::live();
    for n in 5..=8 {
        f.edit(&format!("validation/register/tests/TST-{n:03}.yaml"), |v| {
            v["execution_type"] = json!("automated")
        });
    }
    f.reject("declared investigation methods");
    let f = Fixture::live();
    f.edit(VAL, |v| {
        for exp in v["coverage"]["experiments"].as_array_mut().unwrap() {
            exp["mandatory"] = json!(false);
        }
    });
    f.reject("mapped mandatory investigation");
}

#[test]
fn malformed_schema_missing_directory_bad_filenames_and_missing_targets_fail_closed() {
    let f = Fixture::live();
    f.edit("validation/schemas/test.schema.json", |v| {
        v["type"] = json!(123)
    });
    f.reject("invalid schema");
    let f = Fixture::live();
    fs::remove_dir_all(f.0.path().join("validation/decisions")).unwrap();
    f.reject("decision schema/registry");
    let f = Fixture::live();
    fs::rename(
        f.0.path().join(TEST),
        f.0.path().join("validation/register/tests/TST-999.yaml"),
    )
    .unwrap();
    f.reject("filename must match");
    let f = Fixture::live();
    f.edit(TEST, |v| v["experiment"]["id"] = json!("EXP-999"));
    f.reject("unknown record reference");
}

#[test]
fn registry_rejects_unexpected_entries_and_multi_document_yaml() {
    let f = Fixture::live();
    fs::write(
        f.0.path().join("validation/register/tests/extra.txt"),
        "unexpected",
    )
    .unwrap();
    f.reject("must be .yaml files");
    let f = Fixture::live();
    let path = f.0.path().join(TEST);
    let text = fs::read_to_string(&path).unwrap() + "\n---\n{}\n";
    fs::write(path, text).unwrap();
    f.reject("exactly one YAML document");
}

#[test]
fn run_accounting_and_chronology_cannot_claim_an_unsupported_pass() {
    let f = Fixture::live();
    f.edit(RUN, |r| r["ended_at"] = json!("2026-10-14T00:00:00Z"));
    f.reject("ends before it starts");
    let f = Fixture::live();
    f.edit(RUN, |r| {
        let mut t = r["tests"][0].clone();
        t["details"] = json!("duplicate case with distinct details");
        r["tests"].as_array_mut().unwrap().push(t);
    });
    f.reject("duplicate test results");
    let f = Fixture::live();
    f.edit(RUN, |r| r["source"]["changed_during_run"] = json!(true));
    f.reject("outcome disagrees");
    let f = Fixture::live();
    f.edit(RUN, |r| r["tests"][0]["outcome"] = json!("failed"));
    f.reject("outcome disagrees");
}

#[test]
fn strict_utc_dates_and_timestamps_reject_offsets_and_invalid_calendar_values() {
    for value in [
        "2026-02-30T00:00:00Z",
        "2026-10-15T24:00:00Z",
        "2026-10-15T12:00:60Z",
        "2026-10-15T12:00:00+00:00",
        "2026-10-15t12:00:00Z",
        "0000-01-01T00:00:00Z",
    ] {
        let f = Fixture::live();
        f.edit(RUN, |r| r["started_at"] = json!(value));
        f.reject("date-time");
    }
    let f = Fixture::live();
    f.edit(RUN, |r| {
        r["started_at"] = json!("2026-10-15T00:00:00.1234569Z");
        r["ended_at"] = json!("2026-10-15T00:00:00.1234561Z");
    });
    // Python datetime compares microseconds, truncating further precision.
    let report = f.check();
    assert!(report.accepted(), "{:?}", report.errors);
}

#[test]
fn schema_references_cannot_fetch_remote_documents() {
    let f = Fixture::live();
    f.edit("validation/schemas/test.schema.json", |v| {
        v["allOf"] = json!([{"$ref":"https://example.invalid/schema.json"}])
    });
    f.reject("test schema/registry");
}

#[test]
fn yaml_scalar_types_aliases_and_duplicate_keys_preserve_reference_semantics() {
    use registry::yaml::parse;
    let v=parse("a: yes\nb: 'yes'\nc: 012\nd: '012'\ne: 1e3\nf: 1.0e+3\ng: null\nh: 'null'\ni: 0x10\nj: 1:02\nk: 18446744073709551616\n").unwrap();
    assert_eq!(v["a"], json!(true));
    assert_eq!(v["b"], json!("yes"));
    assert_eq!(v["c"], json!(10));
    assert_eq!(v["d"], json!("012"));
    assert_eq!(v["e"], json!("1e3"));
    assert_eq!(v["f"], json!(1000.0));
    assert_eq!(v["g"], Value::Null);
    assert_eq!(v["h"], json!("null"));
    assert_eq!(v["i"], json!(16));
    assert_eq!(v["j"], json!(62));
    assert_eq!(v["k"].to_string(), "18446744073709551616");
    assert_eq!(
        parse("a: &id [1, 2]\nb: *id\n").unwrap(),
        json!({"a":[1,2],"b":[1,2]})
    );
    assert_eq!(
        parse("a: !!str yes\nb: !!int '2'\nc: !!bool true\n").unwrap(),
        json!({"a":"yes","b":2,"c":true})
    );
    assert_eq!(
        parse("a: !!int --1\nb: !!int 0o1\nc: !!bool yEs\nd: !!float --1\n").unwrap(),
        json!({"a":1,"b":1,"c":true,"d":1.0})
    );
    assert!(parse("a: !!int ---1\n").is_err());
    for value in [
        "a: 1\na: 2\n",
        "a: {x: 1, x: 2}\n",
        "a: &id [*id]\n",
        "a: !!python/object x\n",
        "a: 2026-10-15\n",
        "a: .nan\n",
        "a: .inf\n",
    ] {
        assert!(parse(value).is_err(), "{value}");
    }
    assert_eq!(
        parse("date: '2026-10-15'\n").unwrap()["date"],
        json!("2026-10-15")
    );
}

#[cfg(unix)]
#[test]
fn baseline_paths_and_registry_symlinks_cannot_escape_the_repository() {
    let f = Fixture::live();
    let outside = tempfile::NamedTempFile::new().unwrap();
    let source =
        f.0.path()
            .join("docs/assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md");
    fs::remove_file(&source).unwrap();
    std::os::unix::fs::symlink(outside.path(), &source).unwrap();
    f.reject("path escapes repository");
    let f = Fixture::live();
    let outside = tempfile::NamedTempFile::new().unwrap();
    fs::remove_file(f.0.path().join(TEST)).unwrap();
    std::os::unix::fs::symlink(outside.path(), f.0.path().join(TEST)).unwrap();
    f.reject("path escapes repository");
}

#[test]
fn decision_references_and_inconclusive_closure_are_checked_without_inventing_review() {
    let f = Fixture::live();
    // Synthetic temporary schema/relationship fixture only. Artifact and human
    // attestation checks remain the Stage 3/6 responsibility, not registry acceptance.
    let revision = registry::read_yaml(&f.0.path().join(VAL)).unwrap()["revision"].clone();
    let decision = json!({
        "schema_version":1,"id":"VDEC-999",
        "validation":{"id":"VAL-012","revision":revision,"registry_commit":"1111111111111111111111111111111111111111"},
        "disposition":"inconclusive","rationale":"Synthetic relationship fixture only",
        "execution_ids":["RUN-exp001-local-001"],"evidence_ids":["EVD-exp001-local-001-001"],
        "coverage_assessment":"Synthetic fixture; no technical acceptance",
        "evidence_assessment":[],"unresolved_findings":[],
        "applicability":{"implementation_commit":"1111111111111111111111111111111111111111","workload":"fixture","hardware":"fixture","dependencies":"fixture","exclusions":[],"limitations":["No project approval"]},
        "reviewer":"Synthetic Fixture Reviewer","reviewed_at":"2026-10-15T12:00:00Z",
        "architecture_spec_actions":["No production adoption"],
        "follow_up":[{"action":"Investigate missing scope","owner":"fixture","review_condition":"Additional observations"}],"supersedes":null
    });
    let relative = "validation/decisions/VDEC-999.yaml";
    fs::write(
        f.0.path().join(relative),
        serde_json::to_string(&decision).unwrap(),
    )
    .unwrap();
    assert!(f.check().accepted());
    f.edit(VAL, |v| {
        let mut previous="planned";
        for target in ["ready","running","reviewing","closed"] {
            v["lifecycle"]["history"].as_array_mut().unwrap().push(json!({"from":previous,"to":target,"date":"2026-10-15","actor":"fixture","reason":"Synthetic relationship fixture"}));
            previous=target;
        }
        v["lifecycle"]["state"]=json!("closed");v["current_decision_id"]=json!("VDEC-999");
    });
    f.reject("inconclusive decisions cannot close validations");
    f.edit(VAL, |v| {
        v["lifecycle"]["history"].as_array_mut().unwrap().pop();
        v["lifecycle"]["state"] = json!("reviewing");
    });
    f.edit(relative, |v| v["validation"]["revision"] = json!(1));
    f.reject("current decision must refer to this validation revision");
    f.edit(relative, |v| v["validation"]["revision"] = revision);
    f.edit(VAL, |v| v["prior_decision_ids"] = json!(["VDEC-998"]));
    f.reject("prior decision must exist");
}

#[test]
fn evidence_support_producer_and_supersession_must_reference_known_records() {
    let relative = "validation/evidence-manifests/EVD-exp001-local-001-001.yaml";
    let f = Fixture::live();
    f.edit(relative, |e| {
        e["supports"][0]["validation"]["id"] = json!("VAL-999")
    });
    f.reject("unknown record reference");
    let f = Fixture::live();
    f.edit(relative, |e| {
        e["provenance"]["execution_id"] = json!("RUN-missing")
    });
    f.reject("unknown producing execution");
    let f = Fixture::live();
    f.edit(relative, |e| e["supersedes"] = json!("EVD-missing"));
    f.reject("superseded record does not exist");
}

#[test]
fn yaml_anchor_expansion_and_deep_nesting_are_bounded() {
    let text = format!("a: {}0{}", "[".repeat(130), "]".repeat(130));
    assert!(registry::yaml::parse(&text).is_err());
    let mut text = String::from("a: &a [1,2,3,4,5,6,7,8,9,10]\n");
    let mut prior = String::from("a");
    for index in 0..6 {
        let key = format!("node{index}");
        text.push_str(&format!(
            "{key}: &{key} [{}]\n",
            vec![format!("*{prior}"); 10].join(",")
        ));
        prior = key;
    }
    assert!(registry::yaml::parse(&text).is_err());
}

#[test]
fn remaining_legacy_registry_mutations_are_rejected_natively() {
    for case in [
        "missing-validation",
        "initial-evidence",
        "invalid-state",
        "revision-gap",
        "unmapped",
        "stale-experiment",
        "duplicate-coverage",
        "duplicate-hypothesis",
        "empty-ready-experiment",
        "stale-validation",
        "wrong-owner",
        "outside-scope",
        "missing-test-link",
        "unbound-entrypoint",
        "bound-source",
        "empty-oracle",
        "empty-evidence",
    ] {
        let f = Fixture::live();
        match case {
            "missing-validation"=>fs::remove_file(f.0.path().join(VAL)).unwrap(),
            "initial-evidence"=>f.edit(VAL,|v|{v["revision"]=json!(1);v["revision_history"].as_array_mut().unwrap().truncate(1);v["required_evidence"]=json!(["Documentation build"]);}),
            "invalid-state"=>f.edit(VAL,|v|v["lifecycle"]["state"]=json!("validated")),
            "revision-gap"=>f.edit(VAL,|v|v["revision"]=json!(v["revision"].as_u64().unwrap()+1)),
            "unmapped"=>f.edit(VAL,|v|{v["coverage"]=json!({"status":"unmapped","experiments":[],"tests":[]});v["lifecycle"]["state"]=json!("identified");v["lifecycle"]["history"].as_array_mut().unwrap().truncate(1);}),
            "stale-experiment"=>f.edit(VAL,|v|v["coverage"]["experiments"][0]["revision"]=json!(999)),
            "duplicate-coverage"=>f.edit(VAL,|v|{let mut r=v["coverage"]["experiments"][0].clone();r["mandatory"]=json!(false);v["coverage"]["experiments"].as_array_mut().unwrap().push(r);}),
            "duplicate-hypothesis"=>f.edit("validation/register/experiments/EXP-003.yaml",|v|v["hypotheses"][1]["id"]=v["hypotheses"][0]["id"].clone()),
            "empty-ready-experiment"=>f.edit("validation/register/experiments/EXP-003.yaml",|v|{v["tests"]=json!([]);v["lifecycle"]["state"]=json!("ready");v["lifecycle"]["history"].as_array_mut().unwrap().push(json!({"from":"proposed","to":"ready","date":"2026-10-15","actor":"fixture","reason":"Premature fixture"}));}),
            "stale-validation"=>f.edit(TEST,|v|v["validations"][0]["revision"]=json!(1)),
            "wrong-owner"=>f.edit(TEST,|v|v["experiment"]["id"]=json!("EXP-002")),
            "outside-scope"=>f.edit(TEST,|v|v["validations"]=json!([{"id":"VAL-001","revision":3}])),
            "missing-test-link"=>f.edit(VAL,|v|v["coverage"]["tests"].as_array_mut().unwrap().retain(|r|r["id"]!="TST-001")),
            "unbound-entrypoint"=>f.edit(TEST,|v|v["execution_binding"]=json!({"state":"unbound","implementation_path":null,"entrypoint":"claimed_runner"})),
            "bound-source"=>f.edit(TEST,|v|v["execution_binding"]=json!({"state":"bound","implementation_path":null,"entrypoint":"test-command"})),
            "empty-oracle"=>f.edit(TEST,|v|v["oracle"]["description"]=json!("")),
            "empty-evidence"=>f.edit(TEST,|v|v["required_evidence"]=json!([])),
            _=>unreachable!(),
        }
        let report = f.check();
        let category = match case {
            "missing-validation" => "missing migrated record",
            "initial-evidence" => "initial closure evidence",
            "invalid-state" => "not one of",
            "revision-gap" => "revision history must account",
            "unmapped" => "complete coverage requires",
            "stale-experiment" | "stale-validation" => "stale planning reference",
            "duplicate-coverage" => "duplicate relationship ID",
            "duplicate-hypothesis" => "duplicate hypothesis ID",
            "empty-ready-experiment" | "empty-oracle" | "empty-evidence" => "should be non-empty",
            "wrong-owner" => "test/experiment relationship is not reciprocal",
            "outside-scope" => "outside its experiment",
            "missing-test-link" => "test/validation applicability is not reciprocal",
            "unbound-entrypoint" => "not of type 'null'",
            "bound-source" => "not of type 'string'",
            _ => unreachable!(),
        };
        assert!(
            report.errors.iter().any(|e| e.contains(category)),
            "{case}: expected {category}: {:?}",
            report.errors
        );
    }
    let f = Fixture::live();
    let old = f.check().records["VAL-012"]["origin"]["finding"]
        .as_str()
        .unwrap()
        .to_owned();
    let path =
        f.0.path()
            .join("docs/00-governance/technical-validation.md");
    let text = fs::read_to_string(&path)
        .unwrap()
        .replace(&old, "Coordinated fixture rewrite");
    fs::write(path, text).unwrap();
    f.edit(VAL, |v| {
        v["origin"]["finding"] = json!("Coordinated fixture rewrite")
    });
    f.reject("historical research table differs");
}
#[test]
fn live_baseline_counts_readiness_and_shared_scope_match_the_legacy_contract() {
    let f = Fixture::live();
    let r = f.check();
    assert!(r.accepted(), "{:?}", r.errors);
    let records = r.records;
    assert_eq!(
        records.keys().filter(|id| id.starts_with("VAL-")).count(),
        28
    );
    assert_eq!(
        records.keys().filter(|id| id.starts_with("EXP-")).count(),
        29
    );
    assert_eq!(
        records.keys().filter(|id| id.starts_with("TST-")).count(),
        90
    );
    assert!(!records.contains_key("EXP-004"));
    for (id, v) in &records {
        if id.starts_with("VAL-") {
            assert_eq!(v["lifecycle"]["state"], "planned");
            assert!(v["current_decision_id"].is_null());
        } else if id.starts_with("TST-") {
            assert_eq!(v["mandatory"], true);
            assert_eq!(
                v["execution_binding"]["state"],
                if ["TST-001", "TST-002", "TST-003", "TST-004"].contains(&id.as_str()) {
                    "bound"
                } else {
                    "unbound"
                }
            );
        }
    }
    assert_eq!(
        records["EXP-003"]["validates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from(["VAL-012", "VAL-013"])
    );
    assert_eq!(
        records["TST-009"]["validations"],
        json!([{"id":"VAL-013","revision":records["VAL-013"]["revision"]}])
    );
}
