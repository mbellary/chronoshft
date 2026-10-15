//! Compare the worktree with a trusted Git baseline; never mutate Git or data.
use crate::registry;
use crate::runtime::{array, definition, ensure, safe_path, text, Check, Git};
use serde_json::Value;
use std::fs;
use std::path::Path;

const IMMUTABLE: &[&str] = &[
    "validation/executions/",
    "validation/evidence-manifests/",
    "validation/decisions/",
    "validation/revisions/",
    "validation/artifacts/",
];
const SOURCE: &str = "docs/assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md";
const TABLE: &str = "docs/00-governance/technical-validation.md";
fn prefix(old: &Value, new: &Value) -> bool {
    new.as_array()
        .is_some_and(|n| old.as_array().is_some_and(|o| n.starts_with(o)))
}

pub fn automatic_base(
    git: &Git,
    event_name: Option<&str>,
    event_path: Option<&Path>,
) -> Result<Option<String>, String> {
    if let (Some(name), Some(path)) = (event_name, event_path) {
        let event: Value = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        match name {
            "pull_request" => {
                return event["pull_request"]["base"]["sha"]
                    .as_str()
                    .map(|v| Some(v.to_owned()))
                    .ok_or_else(|| "PR event has no base SHA".into())
            }
            "push" => {
                let before = event["before"]
                    .as_str()
                    .ok_or("Push event has no previous SHA")?;
                return Ok(if !before.is_empty() && before.bytes().all(|b| b == b'0') {
                    None
                } else {
                    Some(before.to_owned())
                });
            }
            "workflow_dispatch" => return Ok(git.resolve("HEAD^").ok()),
            _ => (),
        }
    }
    Ok(Some("HEAD".into()))
}

pub fn check(root: &Path, base: Option<&str>) -> Result<Vec<String>, String> {
    let git = Git::new(root)?;
    let Some(base) = base else {
        return Ok(Vec::new());
    };
    let commit = git.resolve(base)?;
    let names = git.read(&["ls-tree", "-r", "-z", "--name-only", &commit])?;
    let paths = names
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| {
            std::str::from_utf8(s)
                .map(str::to_owned)
                .map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut errors = Vec::new();
    for relative in &paths {
        let path = safe_path(&git.root, relative)?;
        let immutable = IMMUTABLE.iter().any(|prefix| relative.starts_with(prefix))
            || relative == "validation/baseline.yaml"
            || relative == SOURCE;
        if immutable
            && (!path.is_file()
                || fs::read(&path).map_err(|e| e.to_string())? != git.blob(&commit, relative)?)
        {
            errors.push(format!("{relative}: immutable content changed or was removed; append a correction with a new identity"));
        }
        if relative.starts_with("validation/register/") && relative.ends_with(".yaml") {
            if !path.is_file() {
                errors.push(format!(
                    "{relative}: stable registry records cannot be removed"
                ));
                continue;
            }
            let old = registry::yaml::parse(
                std::str::from_utf8(&git.blob(&commit, relative)?).map_err(|e| e.to_string())?,
            )?;
            let new = registry::read_yaml(&path)?;
            let old_revision = old["revision"]
                .as_u64()
                .ok_or("Invalid baseline revision")?;
            let new_revision = new["revision"].as_u64().ok_or("Invalid current revision")?;
            if old["id"] != new["id"] || new_revision < old_revision {
                errors.push(format!(
                    "{relative}: identity changed or revision decreased"
                ));
                continue;
            }
            if old.get("origin") != new.get("origin") {
                errors.push(format!("{relative}: historical provenance was rewritten"));
            }
            if !prefix(&old["revision_history"], &new["revision_history"]) {
                errors.push(format!(
                    "{relative}: recorded revision history was rewritten"
                ));
            }
            if old_revision == new_revision && definition(&old) != definition(&new) {
                errors.push(format!(
                    "{relative}: definition changes require a new revision"
                ));
            }
            if new_revision > old_revision {
                let folder = relative.split('/').nth(2).ok_or("Invalid registry path")?;
                let archive = safe_path(
                    &git.root,
                    format!(
                        "validation/revisions/{folder}/{}/r{old_revision}.yaml",
                        text(&old["id"])
                    ),
                )?;
                if !archive.is_file() {
                    errors.push(format!(
                        "{relative}: previous definition snapshot is missing"
                    ));
                } else {
                    let prior = registry::read_yaml(&archive)?;
                    if definition(&prior) != definition(&old)
                        || !prefix(&old["revision_history"], &prior["revision_history"])
                    {
                        errors.push(format!("{relative}: previous definition snapshot differs from the Git baseline"));
                    }
                    if old.get("lifecycle").is_some()
                        && !prefix(&old["lifecycle"]["history"], &prior["lifecycle"]["history"])
                    {
                        errors.push(format!("{relative}: snapshot rewrote lifecycle history"));
                    }
                }
            }
            if old.get("lifecycle").is_some() {
                let reset = relative.starts_with("validation/register/experiments/")
                    && new_revision > old_revision
                    && old["lifecycle"]["state"] == "completed"
                    && new["lifecycle"]["state"] == "proposed";
                if !reset && !prefix(&old["lifecycle"]["history"], &new["lifecycle"]["history"]) {
                    errors.push(format!(
                        "{relative}: lifecycle transitions were rewritten instead of appended"
                    ));
                }
            }
            if old.get("current_decision_id").is_some_and(|v| !v.is_null())
                && old["current_decision_id"] != new["current_decision_id"]
                && !array(&new["prior_decision_ids"]).contains(&old["current_decision_id"])
            {
                errors.push(format!(
                    "{relative}: previous decision must remain in prior_decision_ids"
                ));
            }
        }
    }
    if paths.iter().any(|p| p == TABLE) {
        let old = String::from_utf8(git.blob(&commit, TABLE)?).map_err(|e| e.to_string())?;
        let path = safe_path(&git.root, TABLE)?;
        let current = if path.is_file() {
            fs::read_to_string(path).map_err(|e| e.to_string())?
        } else {
            String::new()
        };
        let rows: Vec<&str> = current.lines().collect();
        if old
            .lines()
            .filter(|l| l.starts_with("| VAL-"))
            .any(|row| !rows.contains(&row))
        {
            errors.push(format!(
                "{TABLE}: original research rows were changed or removed"
            ));
        }
    }
    Ok(errors)
}

pub(crate) fn require_base(value: &str) -> Check {
    ensure(
        !value.is_empty() && !value.starts_with('-'),
        "Invalid Git baseline reference",
    )
}
