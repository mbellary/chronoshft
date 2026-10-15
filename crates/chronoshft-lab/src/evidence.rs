//! Verify retained observations and supplied reviewer decisions; no registration.
use crate::registry::{self, Records};
use crate::runtime::{self, Check, *};
use chrono::{DateTime, NaiveDateTime};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn when(v: &Value) -> Result<NaiveDateTime, String> {
    registry::instant(text(v)).ok_or_else(|| "Invalid UTC timestamp".into())
}
fn triple(v: &Value) -> (String, String, String) {
    (
        text(&v["id"]).to_owned(),
        v["revision"].to_string(),
        text(&v["registry_commit"]).to_owned(),
    )
}
fn item<'a>(records: &'a Records, id: &str, description: &str) -> Result<&'a Value, String> {
    records
        .get(id)
        .ok_or_else(|| format!("{description}: {id}"))
}

pub fn verify_evidence(
    root: &Path,
    evidence: &Value,
    resolver: &mut PinnedDefinitions,
    records: &Records,
    options: &ArtifactOptions,
) -> Check {
    validate_record(root, "evidence", evidence)?;
    verify_artifact(root, evidence, options)?;
    for support in array(&evidence["supports"]) {
        resolver.record(&support["validation"])?;
    }
    let producer = &evidence["provenance"]["execution_id"];
    ensure(
        producer.is_null()
            || (text(producer).starts_with("RUN-") && records.contains_key(text(producer))),
        format!("{}: missing producing run", text(&evidence["id"])),
    )
}
pub fn aggregate(
    tests: &[Value],
    aborted: bool,
    source_changed: bool,
    infrastructure_error: bool,
) -> &'static str {
    let outcomes: BTreeSet<&str> = tests.iter().map(|t| text(&t["outcome"])).collect();
    if aborted {
        "aborted"
    } else if source_changed || infrastructure_error || outcomes.contains("error") {
        "error"
    } else if outcomes.contains("failed") {
        "failed"
    } else if outcomes.is_empty() || outcomes.contains("skipped") {
        "incomplete"
    } else {
        "passed"
    }
}

pub fn verify_run(
    root: &Path,
    run: &Value,
    resolver: &mut PinnedDefinitions,
    records: &Records,
    options: &ArtifactOptions,
) -> Check {
    validate_record(root, "execution", run)?;
    ensure(
        when(&run["ended_at"])? >= when(&run["started_at"])?,
        "Execution ends before it starts",
    )?;
    safe_path(root, text(&run["working_directory"]))?;
    let experiment = resolver.record(&run["experiment"])?;
    ensure(
        matches!(
            text(&experiment["lifecycle"]["state"]),
            "ready" | "running" | "reviewing" | "completed"
        ),
        "Execution references an experiment that was not ready",
    )?;
    let commit = text(&run["experiment"]["registry_commit"]);
    ensure(
        run["source"]["registry_commit"] == commit,
        format!("{}: inconsistent registry commits", text(&run["id"])),
    )?;
    let implementation = text(&run["source"]["implementation_commit"]);
    resolver.commit(implementation)?;
    let expected: BTreeMap<&str, &Value> = array(&experiment["tests"])
        .iter()
        .map(|t| (text(&t["id"]), &t["revision"]))
        .collect();
    let tests = array(&run["tests"]);
    let actual: BTreeSet<&str> = tests.iter().map(|t| text(&t["test"]["id"])).collect();
    ensure(
        actual.len() == tests.len() && actual == expected.keys().copied().collect(),
        format!(
            "{}: every declared test needs one outcome, including omitted/skipped cases",
            text(&run["id"])
        ),
    )?;
    for case in tests {
        let reference = &case["test"];
        let id = text(&reference["id"]);
        ensure(
            reference["registry_commit"] == commit && &reference["revision"] == expected[id],
            format!(
                "{}: test revision does not match the pinned experiment",
                text(&run["id"])
            ),
        )?;
        let test = resolver.record(reference)?;
        ensure(
            test["experiment"]
                == serde_json::json!({"id":experiment["id"],"revision":experiment["revision"]}),
            format!(
                "{}: test ownership differs from the pinned experiment",
                text(&run["id"])
            ),
        )?;
        if case["outcome"] == "skipped" {
            ensure(
                case["exit_code"].is_null(),
                "Skipped cases cannot have an exit code",
            )?;
        } else {
            ensure(
                test["execution_binding"]["state"] == "bound",
                format!("{id}: observed test method was unbound at the pinned revision"),
            )?;
            if test["execution_type"] == "automated" {
                let command = split_binding(text(&test["execution_binding"]["entrypoint"]))?;
                ensure(
                    case["command"] == serde_json::json!(command),
                    format!("{id}: command differs from the pinned binding"),
                )?;
                resolver.blob(
                    implementation,
                    text(&test["execution_binding"]["implementation_path"]),
                )?;
                if case["outcome"] == "passed" {
                    ensure(
                        case["exit_code"] == 0,
                        "Automated pass requires exit code zero",
                    )?;
                }
                if case["outcome"] == "failed" {
                    ensure(
                        !case["exit_code"].is_null() && case["exit_code"] != 0,
                        "Automated assertion failure requires a nonzero exit code",
                    )?;
                }
            } else {
                ensure(
                    array(&case["command"]).is_empty() && case["exit_code"].is_null(),
                    "Manual observations must not claim an automated command/exit code",
                )?;
            }
            ensure(
                !array(&case["evidence_ids"]).is_empty(),
                format!("{id}: an observed case needs retained evidence"),
            )?;
        }
        ensure(
            strings(&case["evidence_ids"]).is_subset(&strings(&run["evidence_ids"])),
            "Case evidence must be listed in the run",
        )?;
        for id in strings(&case["evidence_ids"]) {
            let evidence = item(records, id, "Missing case evidence")?;
            let supported: BTreeSet<_> = array(&evidence["supports"])
                .iter()
                .map(|s| triple(&s["validation"]))
                .collect();
            let required: BTreeSet<_> = array(&test["validations"])
                .iter()
                .map(|v| {
                    (
                        text(&v["id"]).to_owned(),
                        v["revision"].to_string(),
                        commit.to_owned(),
                    )
                })
                .collect();
            ensure(
                required.is_subset(&supported),
                format!("{id}: evidence scope does not cover its test applicability"),
            )?;
        }
    }
    ensure(
        run["outcome"]
            == aggregate(
                tests,
                run["outcome"] == "aborted",
                flag(&run["source"]["changed_during_run"]),
                !array(&run["tool_errors"]).is_empty(),
            ),
        format!(
            "{}: aggregate outcome does not match observations",
            text(&run["id"])
        ),
    )?;
    for id in strings(&run["evidence_ids"]) {
        let evidence = item(records, id, "Missing run evidence")?;
        let producer = &evidence["provenance"]["execution_id"];
        ensure(
            producer.is_null() || producer == &run["id"],
            format!("{id}: producing run differs from linked observation"),
        )?;
        verify_evidence(root, evidence, resolver, records, options)?;
    }
    for input in array(&run["inputs"]) {
        let path=text(&input["reference"]).strip_prefix("repo://").ok_or("Input references must pin repository content; retain external inputs as evidence first")?;
        ensure(
            input["sha256"] == bytes_digest(&resolver.blob(implementation, path)?),
            "Input hash differs from the pinned implementation content",
        )?;
    }
    for lock in array(&run["source"]["dependency_lockfiles"]) {
        ensure(
            lock["sha256"] == bytes_digest(&resolver.blob(implementation, text(&lock["path"]))?),
            "Dependency lockfile hash differs from the pinned content",
        )?;
    }
    if flag(&run["source"]["dirty"]) {
        let patch = records.get(text(&run["source"]["patch_evidence_id"]));
        ensure(
            patch.is_some_and(|p| {
                p["kind"] == "source_patch"
                    && strings(&run["evidence_ids"]).contains(text(&p["id"]))
            }),
            "Dirty observations require a retained source-patch evidence item",
        )?;
    }
    Ok(())
}

pub fn verify_decision(
    root: &Path,
    decision: &Value,
    resolver: &mut PinnedDefinitions,
    records: &Records,
    options: &ArtifactOptions,
    require_current: bool,
) -> Check {
    validate_record(root, "decision", decision)?;
    let val = resolver.record(&decision["validation"])?;
    let id = text(&val["id"]);
    let current = records.get(id);
    let current_scope =
        require_current || current.is_some_and(|v| v["current_decision_id"] == decision["id"]);
    if current_scope {
        ensure(
            current.is_some_and(|v| definition(v) == definition(&val)),
            "Decision does not apply to the current validation definition",
        )?;
    }
    ensure(
        ![
            "",
            "pending",
            "unassigned",
            "reviewer",
            "codex",
            "automation",
        ]
        .contains(&text(&decision["reviewer"]).trim().to_lowercase().as_str()),
        "A named human reviewer attestation is required",
    )?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    let now = DateTime::from_timestamp(
        now.as_secs()
            .try_into()
            .map_err(|_| "System timestamp overflow")?,
        now.subsec_micros() * 1000,
    )
    .ok_or("System timestamp out of range")?
    .naive_utc();
    let reviewed = when(&decision["reviewed_at"])?;
    ensure(reviewed <= now, "Review timestamp is in the future")?;
    let mut selected = Vec::new();
    for run_id in array(&decision["execution_ids"]).iter().map(text) {
        ensure(
            run_id.starts_with("RUN-"),
            format!("Missing decision execution: {run_id}"),
        )?;
        let run = item(records, run_id, "Missing decision execution")?;
        verify_run(root, run, resolver, records, options)?;
        ensure(
            when(&run["ended_at"])? <= reviewed,
            "Review predates its supporting execution",
        )?;
        ensure(
            ids(&resolver.record(&run["experiment"])?["validates"]).contains(id),
            "Decision cites an execution unrelated to its validation",
        )?;
        selected.push(run);
    }
    for evidence_id in strings(&decision["evidence_ids"]) {
        ensure(
            evidence_id.starts_with("EVD-"),
            format!("Missing decision evidence: {evidence_id}"),
        )?;
        let evidence = item(records, evidence_id, "Missing decision evidence")?;
        verify_evidence(root, evidence, resolver, records, options)?;
        ensure(
            array(&evidence["supports"])
                .iter()
                .any(|s| s["validation"] == decision["validation"]),
            format!("{evidence_id}: evidence does not support this validation revision/commit"),
        )?;
    }
    if !matches!(text(&decision["disposition"]), "accepted" | "conditional") {
        return Ok(());
    }
    for value in [
        &decision["rationale"],
        &decision["coverage_assessment"],
        &decision["applicability"]["workload"],
        &decision["applicability"]["hardware"],
        &decision["applicability"]["dependencies"],
    ] {
        ensure(
            !["", "pending", "unassigned", "tbd"]
                .contains(&text(value).trim().to_lowercase().as_str()),
            "Positive decision needs a concrete rationale, coverage and operating envelope",
        )?;
    }
    if decision["disposition"] == "conditional" {
        ensure(
            !array(&decision["applicability"]["limitations"]).is_empty()
                || !array(&decision["applicability"]["exclusions"]).is_empty(),
            "Conditional acceptance must state an explicit limitation or exclusion",
        )?;
    }
    ensure(
        !array(&decision["unresolved_findings"])
            .iter()
            .any(|f| flag(&f["invalidates_conclusion"])),
        "Positive decision cannot retain an invalidating finding",
    )?;
    let assessments = array(&decision["evidence_assessment"]);
    let requirements: BTreeSet<&str> = assessments
        .iter()
        .map(|a| text(&a["requirement"]))
        .collect();
    ensure(
        requirements.len() == assessments.len()
            && requirements == strings(&val["required_evidence"]),
        "Every required evidence obligation needs an explicit assessment",
    )?;
    for assessment in assessments {
        ensure(
            !text(&assessment["rationale"]).trim().is_empty(),
            "Evidence assessments need a substantive rationale",
        )?;
        ensure(
            strings(&assessment["evidence_ids"]).is_subset(&strings(&decision["evidence_ids"])),
            "Assessed evidence is not in the decision",
        )?;
    }
    let registry_commit = text(&decision["validation"]["registry_commit"]);
    let implementation = text(&decision["applicability"]["implementation_commit"]);
    let mandatory = |key: &str| -> BTreeMap<String, Value> {
        array(&val["coverage"][key])
            .iter()
            .filter(|r| flag(&r["mandatory"]))
            .map(|r| (text(&r["id"]).to_owned(), r["revision"].clone()))
            .collect()
    };
    let expected_tests = mandatory("tests");
    let expected_experiments = mandatory("experiments");
    ensure(
        !expected_tests.is_empty() && !expected_experiments.is_empty(),
        "Positive closure needs declared mandatory test/experiment coverage",
    )?;
    let mut seen: BTreeMap<&str, (NaiveDateTime, &str, &Value)> = BTreeMap::new();
    for run in &selected {
        ensure(
            run["source"]["implementation_commit"] == implementation
                && run["experiment"]["registry_commit"] == registry_commit,
            "Positive decision mixes source/registry applicability boundaries",
        )?;
        ensure(
            !flag(&run["source"]["dirty"])
                && !flag(&run["source"]["changed_during_run"])
                && !matches!(text(&run["outcome"]), "error" | "aborted"),
            "Positive decision needs clean, valid execution observations",
        )?;
        ensure(
            expected_experiments.get(text(&run["experiment"]["id"]))
                == Some(&run["experiment"]["revision"]),
            "Decision execution is outside mandatory experiment coverage",
        )?;
        for case in array(&run["tests"]) {
            let id = text(&case["test"]["id"]);
            if expected_tests.get(id) == Some(&case["test"]["revision"])
                && case["outcome"] != "skipped"
            {
                let time = when(&run["ended_at"])?;
                if seen.get(id).is_none_or(|prior| time > prior.0) {
                    seen.insert(id, (time, text(&case["outcome"]), run));
                }
            }
        }
    }
    ensure(
        seen.keys().copied().collect::<BTreeSet<_>>()
            == expected_tests.keys().map(String::as_str).collect()
            && seen.values().all(|v| v.1 == "passed"),
        "Positive decision lacks passing observations for every mandatory applicable test",
    )?;
    for (id, (_, _, run)) in &seen {
        let case = array(&run["tests"])
            .iter()
            .find(|c| text(&c["test"]["id"]) == *id)
            .ok_or("Missing selected case")?;
        ensure(
            strings(&case["evidence_ids"]).is_subset(&strings(&decision["evidence_ids"])),
            "Decision must retain the supporting artifacts for each mandatory passing case",
        )?;
    }
    for (exp_id, revision) in &expected_experiments {
        let pinned = resolver.record(&reference(exp_id, revision, registry_commit))?;
        ensure(
            selected
                .iter()
                .filter(|r| r["experiment"]["id"] == *exp_id)
                .any(|run| {
                    array(&run["tests"]).iter().any(|c| {
                        c["outcome"] == "passed"
                            && expected_tests.contains_key(text(&c["test"]["id"]))
                    })
                }),
            format!(
                "{}: mandatory experiment has no passing applicable observation",
                text(&pinned["id"])
            ),
        )?;
    }
    for run in records.values().filter(|r| {
        text(&r["id"]).starts_with("RUN-")
            && r["source"]["implementation_commit"] == implementation
            && r["experiment"]["registry_commit"] == registry_commit
    }) {
        for case in array(&run["tests"]) {
            let Some(prior) = seen.get(text(&case["test"]["id"])) else {
                continue;
            };
            if !matches!(text(&case["outcome"]), "failed" | "error") {
                continue;
            }
            let chosen = prior.2;
            let same_context = run["environment"] == chosen["environment"]
                && run["inputs"] == chosen["inputs"]
                && run["random_seeds"] == chosen["random_seeds"];
            let ended = when(&run["ended_at"])?;
            ensure(
                !(same_context && (current_scope || ended <= reviewed) && ended >= prior.0),
                "A later contradictory observation invalidates the selected pass",
            )?;
        }
    }
    Ok(())
}

pub struct Verification {
    pub errors: Vec<String>,
    pub records: Records,
}
pub fn verify_all(root: &Path, options: &ArtifactOptions) -> Result<Verification, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let records = load_records(&root)?;
    let mut resolver = PinnedDefinitions::new(&root)?;
    let mut errors = Vec::new();
    let plans = safe_path(&root, "validation/execution-plans")?;
    if plans.is_dir() {
        let mut paths = fs::read_dir(plans)
            .map_err(|e| e.to_string())?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        paths.sort();
        for path in paths
            .into_iter()
            .filter(|p| p.extension().is_some_and(|s| s == "yaml"))
        {
            let result = (|| -> Check {
                let plan = registry::read_yaml(&safe_path(&root, &path)?)?;
                validate_record(&root, "capture-plan", &plan)?;
                let mut reference = plan["experiment"].clone();
                if reference["registry_commit"] == "HEAD" {
                    reference["registry_commit"] = resolver.git.resolve("HEAD")?.into();
                }
                let experiment = resolver.record(&reference)?;
                let expected: BTreeMap<&str, &Value> = array(&experiment["tests"])
                    .iter()
                    .map(|t| (text(&t["id"]), &t["revision"]))
                    .collect();
                ensure(
                    ids(&plan["tests"]).len() == array(&plan["tests"]).len(),
                    "Duplicate capture-plan test identity",
                )?;
                for t in array(&plan["tests"]) {
                    ensure(
                        expected.get(text(&t["id"])) == Some(&&t["revision"]),
                        "Capture plan references a foreign/stale test",
                    )?;
                    resolver.record(&runtime::reference(
                        text(&t["id"]),
                        &t["revision"],
                        text(&reference["registry_commit"]),
                    ))?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                errors.push(format!(
                    "{}: {error}",
                    path.strip_prefix(&root).unwrap_or(&path).display()
                ));
            }
        }
    }
    for (id, record) in &records {
        let result = (|| -> Check {
            if id.starts_with("EVD-") {
                verify_evidence(&root, record, &mut resolver, &records, options)?;
            } else if id.starts_with("RUN-") {
                verify_run(&root, record, &mut resolver, &records, options)?;
            } else if id.starts_with("VDEC-") {
                verify_decision(&root, record, &mut resolver, &records, options, false)?;
            }
            if let Some(predecessor) = record.get("supersedes").filter(|v| !v.is_null()) {
                let prior = item(
                    &records,
                    text(predecessor),
                    "Invalid supersession predecessor",
                )?;
                ensure(
                    text(&prior["id"]).split('-').next() == id.split('-').next()
                        && prior["id"] != *id,
                    "Invalid supersession predecessor",
                )?;
                let mut visited = BTreeSet::from([id.as_str()]);
                let mut cursor = prior;
                while let Some(next) = cursor.get("supersedes").filter(|v| !v.is_null()) {
                    ensure(
                        visited.insert(text(&cursor["id"])),
                        "Supersession cycle detected",
                    )?;
                    cursor = item(&records, text(next), "Invalid supersession predecessor")?;
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            errors.push(format!("{id}: {error}"));
        }
    }
    Ok(Verification { errors, records })
}
