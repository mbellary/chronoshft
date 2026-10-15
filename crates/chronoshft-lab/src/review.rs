//! Unapproved review packages and explicit registration of supplied attestations.
//! These checks do not authenticate a reviewer or select/close a validation.
use crate::{evidence, publication, registry, runtime::*};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const INSTRUCTIONS:&str="Unapproved package. Select applicable runs/evidence; complete every field and the evidence assessment. A human reviewer must attest the decision. Set draft: false only after that review.";
fn resolved_output(path: &Path) -> Result<PathBuf, String> {
    let absolute = std::path::absolute(path).map_err(|e| e.to_string())?;
    let filesystem_root = absolute
        .ancestors()
        .last()
        .ok_or("Output has no filesystem root")?;
    safe_path(filesystem_root, &absolute)
}
/// Prepare editable, exclusively created draft data outside the decision ledger.
/// Run discovery includes failures/skips and is not a review or evidence selection.
pub fn prepare(
    root: &Path,
    validation_id: &str,
    decision_id: &str,
    output: &Path,
) -> Result<Value, String> {
    ensure(
        Regex::new(r"^VAL-[0-9]{3,}$")
            .unwrap()
            .is_match(validation_id),
        "Review requires a VAL identity",
    )?;
    ensure(
        Regex::new(r"^VDEC-[0-9]{3,}$")
            .unwrap()
            .is_match(decision_id),
        "Review requires a VDEC identity",
    )?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let output = resolved_output(output)?;
    ensure(
        !output.starts_with(safe_path(&root, "validation/decisions")?),
        "Unapproved drafts must remain outside the decision registry",
    )?;
    let records = load_records(&root)?;
    let val = records
        .get(validation_id)
        .ok_or_else(|| format!("Unknown validation: {validation_id}"))?;
    let mut resolver = PinnedDefinitions::new(&root)?;
    let mut commit = resolver.git.resolve("HEAD")?;
    let pinned = resolver.record(&reference(validation_id, &val["revision"], &commit))?;
    let experiments = ids(&pinned["coverage"]["experiments"]);
    let runs: Vec<_> = records
        .values()
        .filter(|r| {
            text(&r["id"]).starts_with("RUN-") && experiments.contains(text(&r["experiment"]["id"]))
        })
        .collect();
    let sources: BTreeSet<_> = runs
        .iter()
        .map(|r| text(&r["experiment"]["registry_commit"]))
        .collect();
    if sources.len() == 1 {
        let observed_commit = *sources.first().unwrap();
        let observed =
            resolver.record(&reference(validation_id, &val["revision"], observed_commit))?;
        if definition(&observed) == definition(val) {
            commit = observed_commit.into();
        }
    }
    let evidence: BTreeSet<_> = runs
        .iter()
        .flat_map(|r| array(&r["evidence_ids"]).iter().map(text))
        .collect();
    let candidate = json!({"schema_version":1,"id":decision_id,"validation":reference(validation_id,&val["revision"],&commit),"disposition":null,"rationale":null,"execution_ids":runs.iter().map(|r|text(&r["id"])).collect::<Vec<_>>(),"evidence_ids":evidence,"coverage_assessment":null,"evidence_assessment":array(&val["required_evidence"]).iter().map(|requirement|json!({"requirement":requirement,"evidence_ids":[],"rationale":null})).collect::<Vec<_>>(),"unresolved_findings":[],"applicability":{"implementation_commit":null,"workload":null,"hardware":null,"dependencies":null,"exclusions":[],"limitations":[]},"reviewer":null,"reviewed_at":null,"architecture_spec_actions":[],"follow_up":[],"supersedes":val["current_decision_id"]});
    let draft = json!({"draft":true,"instructions":INSTRUCTIONS,"decision":candidate,"mandatory_experiments":val["coverage"]["experiments"],"mandatory_tests":val["coverage"]["tests"]});
    let bytes = publication::yaml_text(&draft)?;
    fs::create_dir_all(output.parent().ok_or("Review output has no parent")?)
        .map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&output)
        .map_err(|e| {
            format!(
                "Cannot exclusively create review package {}: {e}",
                output.display()
            )
        })?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    Ok(draft)
}
fn predecessor(candidate: &Value, records: &registry::Records) -> Check {
    let mut visited = BTreeSet::from([text(&candidate["id"])]);
    let validation = &candidate["validation"]["id"];
    let mut cursor = candidate;
    while let Some(id) = cursor.get("supersedes").filter(|v| !v.is_null()) {
        let id = text(id);
        ensure(visited.insert(id), "Decision supersession cycle detected")?;
        cursor = records
            .get(id)
            .ok_or("Supersession must reference a decision for the same validation")?;
        ensure(
            id.starts_with("VDEC-") && &cursor["validation"]["id"] == validation,
            "Supersession must reference a decision for the same validation",
        )?;
    }
    Ok(())
}
/// Check a bare decision or a wrapper explicitly marked `draft: false`. A check
/// alone is read-only. `register_reviewed` is the caller's explicit attestation;
/// successful registration writes only a new VDEC, never a lifecycle/reference.
pub fn check(
    root: &Path,
    supplied: &Value,
    options: &ArtifactOptions,
    register_reviewed: bool,
) -> Result<Value, String> {
    ensure(supplied.is_object(), "Review candidate must be a mapping")?;
    let candidate = if let Some(decision) = supplied.get("decision") {
        ensure(
            supplied.get("draft") == Some(&Value::Bool(false)),
            "Draft review packages cannot be registered or treated as approved",
        )?;
        decision
    } else {
        supplied
    };
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let records = load_records(&root)?;
    let mut resolver = PinnedDefinitions::new(&root)?;
    evidence::verify_decision(&root, candidate, &mut resolver, &records, options, true)?;
    if register_reviewed {
        let current = records
            .get(text(&candidate["validation"]["id"]))
            .ok_or("Current validation is missing")?;
        ensure(
            current["lifecycle"]["state"] == "reviewing",
            "The validation must explicitly be in reviewing state before decision registration",
        )?;
        let identity = text(&candidate["id"]);
        ensure(
            !records.contains_key(identity),
            "Decision IDs cannot be overwritten or reused",
        )?;
        predecessor(candidate, &records)?;
        let path = safe_path(&root, format!("validation/decisions/{identity}.yaml"))?;
        publication::immutable_write(&path, &publication::yaml_text(candidate)?)
            .map_err(|e| format!("Exclusive decision registration failed: {e}"))?;
    }
    Ok(candidate.clone())
}
