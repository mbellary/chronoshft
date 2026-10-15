//! Git-pinned definitions and artifact verification shared with append-only capture.
use crate::registry::{self, Records};
use jsonschema::Validator;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub type Check = Result<(), String>;
pub(crate) fn text(v: &Value) -> &str {
    v.as_str().unwrap_or("")
}
pub(crate) fn array(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}
pub(crate) fn flag(v: &Value) -> bool {
    v.as_bool().unwrap_or(false)
}
pub(crate) fn ensure(condition: bool, message: impl Into<String>) -> Check {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
pub(crate) fn strings(v: &Value) -> BTreeSet<&str> {
    array(v).iter().map(text).collect()
}
pub(crate) fn ids(v: &Value) -> BTreeSet<&str> {
    array(v).iter().map(|v| text(&v["id"])).collect()
}
pub fn definition(record: &Value) -> Value {
    let mut result = record.clone();
    if let Some(map) = result.as_object_mut() {
        for key in [
            "lifecycle",
            "current_decision_id",
            "prior_decision_ids",
            "revision_history",
        ] {
            map.remove(key);
        }
    }
    result
}

/// Resolve existing ancestors (including symlinks), permitting a deleted tail
/// for Git historical reads. A lexical '..' or symlink cannot escape the root.
pub fn safe_path(root: &Path, relative: impl AsRef<Path>) -> Result<PathBuf, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let relative = relative.as_ref();
    let relative = if relative.is_absolute() {
        relative
            .strip_prefix(&root)
            .map_err(|_| "path escapes repository")?
    } else {
        relative
    };
    let mut path = root.clone();
    for part in relative.components() {
        match part {
            Component::Normal(name) => {
                path.push(name);
                if fs::symlink_metadata(&path).is_ok() {
                    path = path.canonicalize().map_err(|e| e.to_string())?;
                }
            }
            Component::CurDir => (),
            Component::ParentDir => {
                ensure(path != root, "path escapes repository")?;
                path.pop();
            }
            _ => return Err("path escapes repository".into()),
        }
        ensure(path.starts_with(&root), "path escapes repository")?;
    }
    Ok(path)
}

#[derive(Clone)]
pub struct Git {
    pub root: PathBuf,
}
impl Git {
    pub fn new(root: &Path) -> Result<Self, String> {
        let git = Self {
            root: root.canonicalize().map_err(|e| e.to_string())?,
        };
        ensure(
            git.string(&["rev-parse", "--is-inside-work-tree"])? == "true",
            "Git worktree required",
        )?;
        Ok(git)
    }
    pub fn read(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()
            .map_err(|e| format!("Git could not start: {e}"))?;
        ensure(
            output.status.success(),
            format!(
                "Git reference/content unavailable: {}",
                args.iter().take(2).copied().collect::<Vec<_>>().join(" ")
            ),
        )?;
        Ok(output.stdout)
    }
    pub fn string(&self, args: &[&str]) -> Result<String, String> {
        String::from_utf8(self.read(args)?)
            .map(|s| s.trim().to_owned())
            .map_err(|e| e.to_string())
    }
    pub fn resolve(&self, reference: &str) -> Result<String, String> {
        self.string(&[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{reference}^{{commit}}"),
        ])
    }
    pub fn exact(&self, commit: &str) -> Check {
        ensure(
            commit.len() == 40
                && commit
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
            "An exact 40-character Git commit is required",
        )?;
        ensure(
            self.resolve(commit)? == commit,
            "Git reference must identify the exact commit",
        )
    }
    pub fn blob(&self, commit: &str, path: &str) -> Result<Vec<u8>, String> {
        self.exact(commit)?;
        safe_path(&self.root, path)?;
        self.read(&["show", &format!("{commit}:{path}")])
    }
}

pub fn reference(id: &str, revision: &Value, commit: &str) -> Value {
    json!({"id":id,"revision":revision,"registry_commit":commit})
}

pub struct PinnedDefinitions {
    pub git: Git,
    records: BTreeMap<(String, String, String), Value>,
    schemas: BTreeMap<(String, String), Validator>,
    commits: BTreeSet<String>,
}
impl PinnedDefinitions {
    pub fn new(root: &Path) -> Result<Self, String> {
        Ok(Self {
            git: Git::new(root)?,
            records: BTreeMap::new(),
            schemas: BTreeMap::new(),
            commits: BTreeSet::new(),
        })
    }
    pub fn commit(&mut self, commit: &str) -> Check {
        if !self.commits.contains(commit) {
            self.git.exact(commit)?;
            self.commits.insert(commit.to_owned());
        }
        Ok(())
    }
    pub fn blob(&mut self, commit: &str, path: &str) -> Result<Vec<u8>, String> {
        self.commit(commit)?;
        safe_path(&self.git.root, path)?;
        self.git.read(&["show", &format!("{commit}:{path}")])
    }
    pub fn record(&mut self, reference: &Value) -> Result<Value, String> {
        let id = reference["id"]
            .as_str()
            .ok_or("Pinned reference needs an identity")?;
        let revision = reference["revision"]
            .as_u64()
            .filter(|r| *r > 0)
            .ok_or("Pinned reference needs a positive integer revision")?;
        let commit = reference["registry_commit"]
            .as_str()
            .ok_or("Pinned reference needs a registry commit")?;
        let key = (id.to_owned(), revision.to_string(), commit.to_owned());
        if let Some(record) = self.records.get(&key) {
            return Ok(record.clone());
        }
        let (prefix, digits) = id.split_once('-').ok_or("Not a versioned definition")?;
        ensure(
            digits.len() >= 3 && digits.bytes().all(|c| c.is_ascii_digit()),
            "Not a versioned definition",
        )?;
        let (kind, folder) = match prefix {
            "VAL" => ("validation", "validations"),
            "EXP" => ("experiment", "experiments"),
            "TST" => ("test", "tests"),
            _ => return Err(format!("Not a versioned definition: {id}")),
        };
        let content = self.blob(commit, &format!("validation/register/{folder}/{id}.yaml"))?;
        let mut record =
            registry::yaml::parse(std::str::from_utf8(&content).map_err(|e| e.to_string())?)?;
        if record["revision"] != revision {
            let content = self.blob(
                commit,
                &format!("validation/revisions/{folder}/{id}/r{revision}.yaml"),
            )?;
            record =
                registry::yaml::parse(std::str::from_utf8(&content).map_err(|e| e.to_string())?)?;
        }
        ensure(
            record["id"] == id && record["revision"] == revision,
            format!("Pinned definition does not match {id} r{revision}"),
        )?;
        let schema_key = (commit.to_owned(), kind.to_owned());
        if !self.schemas.contains_key(&schema_key) {
            let schema: Value = serde_json::from_slice(
                &self.blob(commit, &format!("validation/schemas/{kind}.schema.json"))?,
            )
            .map_err(|e| e.to_string())?;
            self.schemas
                .insert(schema_key.clone(), registry::schema_validator(&schema)?);
        }
        let errors = registry::schema_errors(&self.schemas[&schema_key], &record);
        ensure(errors.is_empty(), errors.join("; "))?;
        self.records.insert(key, record.clone());
        Ok(record)
    }
}

pub fn validate_record(root: &Path, kind: &str, record: &Value) -> Check {
    let errors = registry::schema_errors(&registry::schema_at(root, kind)?, record);
    ensure(errors.is_empty(), errors.join("; "))
}
pub fn load_records(root: &Path) -> Result<Records, String> {
    let mut records = Records::new();
    for &(kind, directory) in registry::CATALOGS {
        let validator = registry::schema_at(root, kind)?;
        let folder = registry::local_path(root, format!("validation/{directory}"))?;
        let mut paths = fs::read_dir(folder)
            .map_err(|e| e.to_string())?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        paths.sort();
        for path in paths {
            if path.file_name().is_some_and(|s| s == ".gitkeep")
                || registry::publication_temporary(&path)
            {
                continue;
            }
            ensure(
                path.is_file() && path.extension().is_some_and(|s| s == "yaml"),
                "Registry entries must be .yaml files",
            )?;
            let record = registry::read_yaml(&registry::local_path(root, &path)?)?;
            let errors = registry::schema_errors(&validator, &record);
            ensure(
                errors.is_empty(),
                format!("{}: {}", path.display(), errors.join("; ")),
            )?;
            let id = text(&record["id"]);
            ensure(
                path.file_stem().and_then(|s| s.to_str()) == Some(id),
                "Registry filename must match the record identity",
            )?;
            ensure(!records.contains_key(id), format!("Duplicate record: {id}"))?;
            records.insert(id.to_owned(), record);
        }
    }
    Ok(records)
}

pub fn file_digest(path: &Path) -> Result<(u64, String), String> {
    let mut input = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut digest = Sha256::new();
    let mut bytes = 0;
    loop {
        let count = input.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
        bytes += count as u64;
    }
    Ok((bytes, format!("{:x}", digest.finalize())))
}
pub fn bytes_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[derive(Default, Clone)]
pub struct ArtifactOptions {
    pub mapping: BTreeMap<String, PathBuf>,
    pub root: Option<PathBuf>,
}
pub fn read_artifact_map(path: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    let value = registry::read_yaml(&path)?;
    let object = value
        .as_object()
        .ok_or("Artifact map must map EVD IDs to local file paths")?;
    object
        .iter()
        .map(|(id, value)| {
            let value = value
                .as_str()
                .ok_or("Artifact map must map EVD IDs to local file paths")?;
            let target = path
                .parent()
                .ok_or("Artifact map has no parent")?
                .join(value);
            Ok((id.clone(), target.canonicalize().unwrap_or(target)))
        })
        .collect()
}
pub fn verify_artifact(
    root: &Path,
    evidence: &Value,
    options: &ArtifactOptions,
) -> Result<PathBuf, String> {
    let id = text(&evidence["id"]);
    let path = if let Some(path) = options.mapping.get(id) {
        path.clone()
    } else if let Some(relative) = text(&evidence["artifact"]["uri"]).strip_prefix("repo://") {
        safe_path(options.root.as_deref().unwrap_or(root), relative)?
    } else {
        return Err(format!(
            "{id}: external artifact needs an explicit local mirror in --artifact-map"
        ));
    };
    ensure(
        path.is_file(),
        format!("{id}: artifact is unavailable: {}", path.display()),
    )?;
    let (bytes, digest) = file_digest(&path)?;
    ensure(
        evidence["artifact"]["size_bytes"] == bytes && evidence["artifact"]["sha256"] == digest,
        format!("{id}: artifact size/SHA-256 mismatch"),
    )?;
    Ok(path)
}

/// Python shlex.split(..., comments=False, posix=True) for bound argv, without
/// executing a shell. Quotes concatenate; '#' is literal; empty quotes survive.
pub fn split_binding(value: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut active = false;
    let mut quote = None;
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match quote {
            Some('\'') => {
                if c == '\'' {
                    quote = None;
                } else {
                    word.push(c);
                }
            }
            Some('"') => {
                if c == '"' {
                    quote = None;
                } else if c == '\\' {
                    let next = chars.next().ok_or("No escaped character")?;
                    if next != '"' && next != '\\' {
                        word.push('\\');
                    }
                    word.push(next);
                } else {
                    word.push(c);
                }
            }
            _ => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    active = true;
                }
                '\\' => {
                    word.push(chars.next().ok_or("No escaped character")?);
                    active = true;
                }
                ' ' | '\t' | '\r' | '\n' => {
                    if active {
                        words.push(std::mem::take(&mut word));
                        active = false;
                    }
                }
                _ => {
                    word.push(c);
                    active = true;
                }
            },
        }
    }
    ensure(quote.is_none(), "No closing quotation")?;
    if active {
        words.push(word);
    }
    Ok(words)
}
