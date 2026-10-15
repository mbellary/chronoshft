//! Read-only registry consistency checks. Historical Git/evidence resolution is
//! a separate migration stage; this module never invokes Python or writes data.
pub mod yaml;

use chrono::{NaiveDate, NaiveDateTime};
use jsonschema::Validator;
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub const CATALOGS: &[(&str, &str)] = &[
    ("validation", "register/validations"),
    ("experiment", "register/experiments"),
    ("test", "register/tests"),
    ("execution", "executions"),
    ("evidence", "evidence-manifests"),
    ("decision", "decisions"),
];
pub type Records = BTreeMap<String, Value>;
type Check = Result<(), String>;

#[derive(Clone, Copy, Default)]
pub struct Options {
    pub require_coverage: bool,
    pub require_tests: bool,
}
#[derive(Default)]
pub struct Report {
    pub errors: Vec<String>,
    pub records: Records,
}
impl Report {
    pub fn accepted(&self) -> bool {
        self.errors.is_empty()
    }
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn text(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}
fn ids(value: &Value) -> BTreeSet<&str> {
    array(value).iter().map(|r| text(&r["id"])).collect()
}
fn flag(value: &Value) -> bool {
    value.as_bool().unwrap_or(false)
}
fn ensure(condition: bool, message: impl Into<String>) -> Check {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

pub fn local_path(root: &Path, relative: impl AsRef<Path>) -> Result<PathBuf, String> {
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    ensure(path.starts_with(root), "path escapes repository")?;
    Ok(path)
}
pub fn read_yaml(path: &Path) -> Result<Value, String> {
    yaml::parse(&fs::read_to_string(path).map_err(|e| e.to_string())?)
}
fn hash(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}

static DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$").unwrap());
static INSTANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([0-9]{4}-[0-9]{2}-[0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})(?:\.([0-9]+))?Z$")
        .unwrap()
});
fn valid_date(value: &str) -> bool {
    DATE.is_match(value)
        && !value.starts_with("0000")
        && NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}
pub(crate) fn instant(value: &str) -> Option<NaiveDateTime> {
    let captures = INSTANT.captures(value)?;
    if !valid_date(&captures[1]) {
        return None;
    }
    let date = NaiveDate::parse_from_str(&captures[1], "%Y-%m-%d").ok()?;
    let fraction = captures.get(5).map(|v| v.as_str()).unwrap_or("");
    let micros = format!("{:0<6}", &fraction[..fraction.len().min(6)])
        .parse::<u32>()
        .ok()?;
    date.and_hms_micro_opt(
        captures[2].parse().ok()?,
        captures[3].parse().ok()?,
        captures[4].parse().ok()?,
        micros,
    )
}

pub fn schema_validator(schema: &Value) -> Result<Validator, String> {
    jsonschema::draft202012::meta::validate(schema).map_err(|e| format!("invalid schema: {e}"))?;
    jsonschema::draft202012::options()
        .should_validate_formats(true)
        .with_format("date", valid_date)
        .with_format("date-time", |v| instant(v).is_some())
        .build(schema)
        .map_err(|e| e.to_string())
}
pub(crate) fn schema_at(root: &Path, kind: &str) -> Result<Validator, String> {
    let path = local_path(root, format!("validation/schemas/{kind}.schema.json"))?;
    let schema: Value = serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    schema_validator(&schema)
}
pub(crate) fn schema_errors(validator: &Validator, value: &Value) -> Vec<String> {
    validator
        .iter_errors(value)
        .map(|error| {
            use jsonschema::error::{TypeKind, ValidationErrorKind};
            let category = match error.kind() {
                ValidationErrorKind::AdditionalProperties { .. } => {
                    "Additional properties are not allowed; ".into()
                }
                ValidationErrorKind::MinLength { .. } | ValidationErrorKind::MinItems { .. } => {
                    "should be non-empty; ".into()
                }
                ValidationErrorKind::Format { format } => format!("not a '{format}'; "),
                ValidationErrorKind::Type {
                    kind: TypeKind::Single(kind),
                } => format!("not of type '{kind}'; "),
                _ => String::new(),
            };
            format!("{}: {category}{error}", error.instance_path())
        })
        .collect()
}

fn baseline(root: &Path, records: &Records) -> Check {
    let base = read_yaml(&local_path(root, "validation/baseline.yaml")?)?;
    let problems = schema_errors(&schema_at(root, "baseline")?, &base);
    ensure(problems.is_empty(), problems.join("; "))?;
    let expected: Vec<String> = (1..=28).map(|n| format!("VAL-{n:03}")).collect();
    ensure(
        base["record_count"] == 28 && base["validation_ids"] == json!(expected),
        "baseline must preserve VAL-001–VAL-028",
    )?;
    let source = &base["source_document"];
    let bytes = fs::read(local_path(root, text(&source["path"]))?).map_err(|e| e.to_string())?;
    ensure(
        hash(bytes) == text(&source["sha256"]),
        "preserved source document hash differs from baseline",
    )?;
    let document = fs::read_to_string(local_path(root, text(&base["baseline_path"]))?)
        .map_err(|e| e.to_string())?;
    let rows: Vec<&str> = document
        .lines()
        .filter(|line| {
            line.starts_with("| VAL-")
                && line.split(" / ").next().is_some_and(|start| {
                    expected
                        .iter()
                        .any(|id| start.strip_prefix("| ") == Some(id.as_str()))
                })
        })
        .collect();
    ensure(
        rows.len() == 28 && hash(rows.join("\n") + "\n") == text(&base["rows_sha256"]),
        "historical research table differs from pinned baseline",
    )?;
    for row in rows {
        let cells: Vec<&str> = row.trim_matches('|').split('|').map(str::trim).collect();
        ensure(cells.len() == 4, "invalid historical research row")?;
        let (id, sources) = cells[0]
            .split_once(" / ")
            .ok_or("invalid historical identity")?;
        let record = records
            .get(id)
            .ok_or_else(|| format!("missing migrated record: {id}"))?;
        let origin = json!({"assessment_id":base["assessment_id"], "assessment_version":base["assessment_version"], "assessment_date":base["assessment_date"], "baseline_path":base["baseline_path"], "source_refs":sources, "source_status":cells[1], "finding":cells[2], "required_closure_evidence":cells[3]});
        ensure(
            record["origin"] == origin,
            format!("{id}: origin differs from historical research assessment"),
        )?;
        if record["revision"] == 1 {
            ensure(
                record["required_evidence"] == json!([cells[3]]),
                format!("{id}: initial closure evidence was not preserved"),
            )?;
        }
    }
    Ok(())
}

fn references<'a>(value: &'a Value, found: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if map.contains_key("id") && map.contains_key("revision") {
                found.push(value);
            }
            for (key, child) in map {
                if !matches!(key.as_str(), "origin" | "revision_history") {
                    references(child, found);
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                references(child, found);
            }
        }
        _ => (),
    }
}
fn lifecycle(record: &Value, kind: &str) -> Check {
    let initial = if kind == "validation" {
        "identified"
    } else {
        "proposed"
    };
    let mut previous = Value::Null;
    let mut last_date = "";
    for entry in array(&record["lifecycle"]["history"]) {
        let target = text(&entry["to"]);
        let date = text(&entry["date"]);
        ensure(
            entry["from"] == previous && date >= last_date,
            "lifecycle history is discontinuous or dates are out of order",
        )?;
        if previous.is_null() {
            ensure(
                target == initial,
                format!("lifecycle must start at {initial}"),
            )?;
        } else {
            let allowed: &[&str] = match (kind, text(&previous)) {
                ("validation", "identified") => &["planned"],
                ("validation", "planned") => &["ready"],
                ("validation", "ready") => &["running", "planned"],
                ("validation", "running") => &["reviewing", "planned"],
                ("validation", "reviewing") => &["closed", "running", "planned"],
                ("validation", "closed") => &["reopened"],
                ("validation", "reopened") => &["planned"],
                ("experiment", "proposed") => &["ready"],
                ("experiment", "ready") => &["running", "proposed"],
                ("experiment", "running") => &["reviewing"],
                ("experiment", "reviewing") => &["completed", "running", "proposed"],
                _ => &[],
            };
            ensure(
                allowed.contains(&target),
                format!(
                    "illegal lifecycle transition: {} -> {target}",
                    text(&previous)
                ),
            )?;
        }
        previous = entry["to"].clone();
        last_date = date;
    }
    ensure(
        previous == record["lifecycle"]["state"],
        "lifecycle state disagrees with its history",
    )
}

fn relationships(record: &Value, kind: &str, records: &Records) -> Check {
    let identity = text(&record["id"]);
    let groups = match kind {
        "validation" => vec![
            &record["coverage"]["experiments"],
            &record["coverage"]["tests"],
        ],
        "experiment" => vec![&record["validates"], &record["tests"]],
        "test" => vec![&record["validations"]],
        _ => vec![],
    };
    for group in groups {
        ensure(
            ids(group).len() == array(group).len(),
            "duplicate relationship ID",
        )?;
    }
    let mut refs = Vec::new();
    references(record, &mut refs);
    for reference in refs {
        let id = text(&reference["id"]);
        let target = records
            .get(id)
            .ok_or_else(|| format!("unknown record reference: {id}"))?;
        if reference.get("registry_commit").is_none() {
            ensure(
                reference["revision"] == target["revision"],
                format!("stale planning reference: {id}"),
            )?;
        }
    }
    if let Some(history) = record.get("revision_history") {
        let entries = array(history);
        ensure(
            record["revision"] == json!(entries.len())
                && entries
                    .iter()
                    .enumerate()
                    .all(|(i, e)| e["revision"] == json!(i + 1)),
            "revision history must account for every revision",
        )?;
        ensure(
            entries
                .windows(2)
                .all(|w| text(&w[0]["date"]) <= text(&w[1]["date"])),
            "revision history dates are out of order",
        )?;
    }
    match kind {
        "validation" => {
            lifecycle(record, kind)?;
            let coverage = &record["coverage"];
            ensure(
                coverage["status"] != "mapped" || !array(&coverage["experiments"]).is_empty(),
                "mapped coverage needs at least one experiment",
            )?;
            for exp in array(&coverage["experiments"]) {
                ensure(
                    ids(&records[text(&exp["id"])]["validates"]).contains(identity),
                    "validation/experiment coverage is not reciprocal",
                )?;
            }
            let linked = ids(&coverage["experiments"]);
            for test in array(&coverage["tests"]) {
                let target = &records[text(&test["id"])];
                ensure(
                    linked.contains(text(&target["experiment"]["id"])),
                    "coverage test belongs to an unlinked experiment",
                )?;
                ensure(
                    ids(&target["validations"]).contains(identity),
                    "validation/test applicability is not reciprocal",
                )?;
                ensure(
                    test["mandatory"] == target["mandatory"],
                    "validation/test mandatory obligation disagrees",
                )?;
            }
            for previous in array(&record["prior_decision_ids"]) {
                ensure(
                    records
                        .get(text(previous))
                        .is_some_and(|d| d["validation"]["id"] == identity),
                    "prior decision must exist and refer to this validation",
                )?;
            }
            let decision = &record["current_decision_id"];
            if !decision.is_null() {
                ensure(
                    records.get(text(decision)).is_some_and(|d| {
                        d["validation"]["id"] == identity
                            && d["validation"]["revision"] == record["revision"]
                    }),
                    "current decision must refer to this validation revision",
                )?;
            }
            if record["lifecycle"]["state"] == "closed" {
                ensure(
                    records
                        .get(text(decision))
                        .is_some_and(|d| d["disposition"] != "inconclusive"),
                    "inconclusive decisions cannot close validations",
                )?;
            }
            ensure(
                record["lifecycle"]["state"] != "reopened" || decision.is_null(),
                "reopened scope must be pending; retain prior decisions separately",
            )?;
        }
        "experiment" => {
            lifecycle(record, kind)?;
            ensure(
                ids(&record["hypotheses"]).len() == array(&record["hypotheses"]).len(),
                "duplicate hypothesis ID",
            )?;
            for val in array(&record["validates"]) {
                ensure(
                    ids(&records[text(&val["id"])]["coverage"]["experiments"]).contains(identity),
                    "experiment/validation coverage is not reciprocal",
                )?;
            }
            for test in array(&record["tests"]) {
                ensure(
                    records[text(&test["id"])]["experiment"]["id"] == identity,
                    "experiment lists a test owned by another experiment",
                )?;
            }
            if record["lifecycle"]["state"] != "proposed" {
                ensure(
                    !array(&record["tests"]).iter().any(|t| {
                        let t = &records[text(&t["id"])];
                        flag(&t["mandatory"]) && t["execution_binding"]["state"] != "bound"
                    }),
                    "experiment cannot advance with unbound mandatory tests",
                )?;
            }
        }
        "test" => {
            let experiment = &records[text(&record["experiment"]["id"])];
            ensure(
                ids(&experiment["tests"]).contains(identity),
                "test/experiment relationship is not reciprocal",
            )?;
            ensure(
                array(&record["hypothesis_ids"])
                    .iter()
                    .all(|h| ids(&experiment["hypotheses"]).contains(text(h))),
                "unknown hypothesis in owning experiment",
            )?;
            ensure(
                ids(&record["validations"]).is_subset(&ids(&experiment["validates"])),
                "test applies to a validation outside its experiment",
            )?;
            for val in array(&record["validations"]) {
                ensure(
                    ids(&records[text(&val["id"])]["coverage"]["tests"]).contains(identity),
                    "test/validation applicability is not reciprocal",
                )?;
            }
        }
        "execution" => {
            ensure(
                instant(text(&record["ended_at"])) >= instant(text(&record["started_at"])),
                "execution ends before it starts",
            )?;
            let tests = array(&record["tests"]);
            let test_ids: BTreeSet<&str> = tests.iter().map(|t| text(&t["test"]["id"])).collect();
            ensure(
                test_ids.len() == tests.len(),
                "execution contains duplicate test results",
            )?;
            if record["outcome"] != "aborted" {
                let outcomes: BTreeSet<&str> = tests.iter().map(|t| text(&t["outcome"])).collect();
                let aggregate = if outcomes.contains("error")
                    || !array(&record["tool_errors"]).is_empty()
                    || flag(&record["source"]["changed_during_run"])
                {
                    "error"
                } else if outcomes.contains("failed") {
                    "failed"
                } else if tests.is_empty() || outcomes.contains("skipped") {
                    "incomplete"
                } else {
                    "passed"
                };
                ensure(
                    record["outcome"] == aggregate
                        || (aggregate == "passed" && record["outcome"] == "incomplete"),
                    "execution outcome disagrees with test results",
                )?;
            }
        }
        _ => (),
    }
    if matches!(kind, "execution" | "decision") {
        for key in ["evidence_ids", "execution_ids"] {
            for id in array(&record[key]) {
                ensure(
                    records.contains_key(text(id)),
                    format!("unknown reference: {}", text(id)),
                )?;
            }
        }
    }
    if kind == "evidence" {
        let run = &record["provenance"]["execution_id"];
        ensure(
            run.is_null() || records.contains_key(text(run)),
            format!("unknown producing execution: {}", text(run)),
        )?;
    }
    if let Some(prior) = record.get("supersedes").filter(|p| !p.is_null()) {
        ensure(
            records.contains_key(text(prior)),
            "superseded record does not exist",
        )?;
    }
    Ok(())
}

fn snapshots(root: &Path, record: &Value, kind: &str, validator: &Validator) -> Check {
    if record.get("revision_history").is_none() {
        return Ok(());
    }
    let directory = match kind {
        "validation" => "validations",
        "experiment" => "experiments",
        "test" => "tests",
        _ => return Err("unexpected revision history".into()),
    };
    // relationships already checks revision == history length; no untrusted
    // integer conversion can cause an unbounded snapshot loop.
    for revision in 1..array(&record["revision_history"]).len() {
        let relative = format!(
            "validation/revisions/{directory}/{}/r{revision}.yaml",
            text(&record["id"])
        );
        let path = root.join(&relative);
        ensure(
            path.is_file(),
            format!(
                "missing prior revision snapshot: {} r{revision}",
                text(&record["id"])
            ),
        )?;
        let prior = read_yaml(&local_path(root, relative)?)?;
        let errors = schema_errors(validator, &prior);
        ensure(errors.is_empty(), errors.join("; "))?;
        ensure(
            prior["id"] == record["id"] && prior["revision"] == json!(revision),
            "prior revision snapshot identity does not match its path",
        )?;
        ensure(
            array(&prior["revision_history"]) == &array(&record["revision_history"])[..revision],
            "prior revision history differs from retained history",
        )?;
        if kind == "validation" {
            ensure(
                prior["origin"] == record["origin"],
                "prior revision snapshot changed the historical research origin",
            )?;
            if revision == 1 {
                ensure(
                    prior["required_evidence"]
                        == json!([prior["origin"]["required_closure_evidence"]]),
                    "initial closure evidence was not preserved in the prior snapshot",
                )?;
            }
            lifecycle(&prior, kind)?;
        } else if kind == "experiment" {
            lifecycle(&prior, kind)?;
        }
    }
    Ok(())
}

fn coverage(records: &Records) -> Vec<String> {
    let mut errors = Vec::new();
    for (id, exp) in records.iter().filter(|(id, _)| id.starts_with("EXP-")) {
        let tests: Vec<&Value> = array(&exp["tests"])
            .iter()
            .map(|r| &records[text(&r["id"])])
            .filter(|t| flag(&t["mandatory"]))
            .collect();
        let covered: BTreeSet<&str> = tests
            .iter()
            .flat_map(|t| array(&t["hypothesis_ids"]).iter().map(text))
            .collect();
        if tests.is_empty() || covered != ids(&exp["hypotheses"]) {
            errors.push(format!("{id}: every hypothesis requires a mandatory test"));
        }
        let modes: BTreeSet<&str> = tests.iter().map(|t| text(&t["execution_type"])).collect();
        let required: &[&str] = match text(&exp["method"]["kind"]) {
            "mixed" => &["manual", "automated"],
            "documentary" => &["manual"],
            _ => &["automated"],
        };
        if !required.iter().all(|m| modes.contains(m)) {
            errors.push(format!(
                "{id}: mandatory tests do not cover the declared investigation methods"
            ));
        }
        for val in array(&exp["validates"]) {
            if !tests
                .iter()
                .any(|t| ids(&t["validations"]).contains(text(&val["id"])))
            {
                errors.push(format!(
                    "{id}: no mandatory applicable test for {}",
                    text(&val["id"])
                ));
            }
        }
    }
    for (id, val) in records.iter().filter(|(id, _)| id.starts_with("VAL-")) {
        if !array(&val["coverage"]["tests"])
            .iter()
            .any(|t| flag(&t["mandatory"]))
        {
            errors.push(format!(
                "{id}: test coverage requires a mandatory applicable test"
            ));
        }
    }
    errors
}

pub fn validate(root: &Path, options: Options) -> Report {
    let mut report = Report::default();
    let root = match root.canonicalize() {
        Ok(root) => root,
        Err(e) => {
            report.errors.push(format!("repository root: {e}"));
            return report;
        }
    };
    let mut kinds = BTreeMap::new();
    let mut validators = BTreeMap::new();
    for &(kind, directory) in CATALOGS {
        let result = (|| -> Check {
            let validator = schema_at(&root, kind)?;
            let folder = local_path(&root, format!("validation/{directory}"))?;
            ensure(
                folder.is_dir(),
                format!("missing registry directory: {directory}"),
            )?;
            let mut paths = fs::read_dir(folder)
                .map_err(|e| e.to_string())?
                .map(|e| e.map(|e| e.path()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            paths.sort();
            for path in paths {
                if path.file_name().is_some_and(|n| n == ".gitkeep") || publication_temporary(&path)
                {
                    continue;
                }
                let result = (|| -> Check {
                    ensure(
                        path.extension().is_some_and(|s| s == "yaml") && path.is_file(),
                        "registry entries must be .yaml files",
                    )?;
                    let record = read_yaml(&local_path(&root, &path)?)?;
                    let errors = schema_errors(&validator, &record);
                    ensure(errors.is_empty(), errors.join("; "))?;
                    let id = text(&record["id"]);
                    ensure(
                        path.file_stem().and_then(|s| s.to_str()) == Some(id),
                        "filename must match record ID",
                    )?;
                    ensure(
                        !report.records.contains_key(id),
                        format!("duplicate record ID: {id}"),
                    )?;
                    kinds.insert(id.to_owned(), kind);
                    report.records.insert(id.to_owned(), record);
                    Ok(())
                })();
                if let Err(error) = result {
                    report.errors.push(format!(
                        "{}: {error}",
                        path.strip_prefix(&root).unwrap_or(&path).display()
                    ));
                }
            }
            validators.insert(kind, validator);
            Ok(())
        })();
        if let Err(error) = result {
            report
                .errors
                .push(format!("{kind} schema/registry: {error}"));
        }
    }
    if !report.errors.is_empty() {
        return report;
    }
    if let Err(error) = baseline(&root, &report.records) {
        report.errors.push(format!("baseline migration: {error}"));
    }
    for (id, record) in &report.records {
        let kind = kinds[id];
        if let Err(error) = relationships(record, kind, &report.records)
            .and_then(|()| snapshots(&root, record, kind, &validators[kind]))
        {
            report.errors.push(format!("{id}: {error}"));
        }
    }
    if options.require_coverage || options.require_tests {
        for (id, val) in report
            .records
            .iter()
            .filter(|(id, _)| kinds[*id] == "validation")
        {
            let c = &val["coverage"];
            if c["status"] != "mapped"
                || !array(&c["experiments"])
                    .iter()
                    .any(|e| flag(&e["mandatory"]))
            {
                report.errors.push(format!(
                    "{id}: complete coverage requires a mapped mandatory investigation"
                ));
            }
        }
    }
    if options.require_tests && report.errors.is_empty() {
        report.errors.extend(coverage(&report.records));
    }
    report
}

/// Reserved orphan/active temporary names from exclusive publication. They are
/// never registry records. Ignore only regular files with the known temp suffix;
/// symlinks, directories and arbitrary unexpected entries still fail validation.
pub(crate) fn publication_temporary(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix(".publish-"))
        .is_some_and(|s| {
            matches!(s.len(), 6 | 8) && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        })
        && std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_file())
}
