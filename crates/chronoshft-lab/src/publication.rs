//! Exclusive append-only publication, with verification before the first write.
use crate::{
    evidence, registry,
    runtime::{self, *},
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

/// JSON is a YAML subset: quote scalars unambiguously without a second YAML
/// serializer. Historical YAML bytes are never reserialized or overwritten.
pub fn yaml_text(value: &Value) -> Result<Vec<u8>, String> {
    let mut data = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    data.push(b'\n');
    Ok(data)
}
fn immutable(path: &Path, write: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Missing publication directory"))?;
    fs::create_dir_all(parent)?;
    let mut temp = tempfile::Builder::new()
        .prefix(".publish-")
        .tempfile_in(parent)?;
    write(temp.as_file_mut())?;
    temp.as_file_mut().flush()?;
    temp.as_file().sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temp.as_file()
            .set_permissions(fs::Permissions::from_mode(0o444))?;
    }
    #[cfg(not(unix))]
    {
        let mut permissions = temp.as_file().metadata()?.permissions();
        permissions.set_readonly(true);
        temp.as_file().set_permissions(permissions)?;
    }
    fs::hard_link(temp.path(), path)?;
    // Persist the directory entry as well as the content on Unix.
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}
pub fn immutable_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    immutable(path, |file| file.write_all(bytes))
}
pub fn immutable_copy(source: &Path, destination: &Path) -> io::Result<()> {
    immutable(destination, |file| {
        let mut source = File::open(source)?;
        io::copy(&mut source, file)?;
        Ok(())
    })
}
/// Verify the complete candidate observation first. Evidence is published before
/// the run. An interrupted prefix is recoverable with identical supplied data.
pub fn publish_run(
    root: &Path,
    run: &Value,
    evidence: &[Value],
    options: &ArtifactOptions,
    resume: bool,
) -> Check {
    runtime::validate_record(root, "execution", run)?;
    for item in evidence {
        runtime::validate_record(root, "evidence", item)?;
    }
    let records = runtime::load_records(root)?;
    let run_id = text(&run["id"]);
    let new_ids: BTreeSet<_> = evidence.iter().map(|e| text(&e["id"])).collect();
    ensure(
        new_ids.len() == evidence.len(),
        "RUN/EVD IDs cannot be overwritten or reused",
    )?;
    ensure(
        resume
            || (!records.contains_key(run_id) && new_ids.iter().all(|i| !records.contains_key(*i))),
        "RUN/EVD IDs cannot be overwritten or reused",
    )?;
    ensure(
        new_ids.is_subset(&strings(&run["evidence_ids"])),
        "Unlinked evidence must not be published with the run",
    )?;
    for item in evidence.iter().chain(std::iter::once(run)) {
        if let Some(existing) = records.get(text(&item["id"])) {
            ensure(
                existing == item,
                "Resume cannot change an already published record",
            )?;
        }
    }
    let mut candidate = records.clone();
    for item in evidence {
        candidate.insert(text(&item["id"]).into(), item.clone());
    }
    candidate.insert(run_id.into(), run.clone());
    evidence::verify_run(
        root,
        run,
        &mut PinnedDefinitions::new(root)?,
        &candidate,
        options,
    )?;
    // Preflight every target before publishing even the first manifest.
    let mut staged = Vec::new();
    for (item, directory) in evidence
        .iter()
        .map(|e| (e, "evidence-manifests"))
        .chain(std::iter::once((run, "executions")))
    {
        let id = text(&item["id"]);
        let path = safe_path(root, format!("validation/{directory}/{id}.yaml"))?;
        if !records.contains_key(id) {
            staged.push((path, yaml_text(item)?));
        }
    }
    for (path, data) in staged {
        immutable_write(&path,&data).map_err(|e|format!("Exclusive publication {} failed: {e}; retain staged records and use --resume only for identical content",path.display()))?;
    }
    Ok(())
}
/// Import supplied manual/staged observations; this never executes a command.
pub fn import(
    root: &Path,
    run_path: &Path,
    evidence_directory: &Path,
    options: &ArtifactOptions,
    resume: bool,
) -> Result<Value, String> {
    let run = registry::read_yaml(run_path)?;
    let mut paths = fs::read_dir(evidence_directory)
        .map_err(|e| e.to_string())?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    paths.retain(|p| p.extension().is_some_and(|e| e == "yaml"));
    paths.sort();
    let items = paths
        .iter()
        .map(|p| registry::read_yaml(p))
        .collect::<Result<Vec<_>, _>>()?;
    publish_run(root, &run, &items, options, resume)?;
    Ok(run)
}
