//! Registry-derived Markdown/navigation and read-only rendered-site verification.
pub mod links;
mod prose;
pub mod render;

use crate::{registry, runtime::safe_path};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// Preflight all paths and inputs before changing any generated file. Authored
/// pages and configuration outside the navigation markers remain untouched.
pub fn outputs(root: &Path) -> Result<BTreeMap<PathBuf, String>, Vec<String>> {
    let report = registry::validate(
        root,
        registry::Options {
            require_tests: true,
            require_coverage: true,
        },
    );
    if !report.errors.is_empty() {
        return Err(report.errors);
    }
    let result = (|| -> Result<_, String> {
        let mut outputs = render::documents(&report.records);
        let experiments = safe_path(root, format!("{}/experiments", render::LAB))?;
        if experiments.exists() {
            for entry in fs::read_dir(experiments).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let relative =
                    PathBuf::from(format!("{}/experiments", render::LAB)).join(entry.file_name());
                if relative.extension().is_some_and(|e| e == "md")
                    && !outputs.contains_key(&relative)
                {
                    return Err(format!(
                        "Unregistered experiment page: {}",
                        relative.display()
                    ));
                }
            }
        }
        let config =
            fs::read_to_string(safe_path(root, "mkdocs.yml")?).map_err(|e| e.to_string())?;
        outputs.insert(
            "mkdocs.yml".into(),
            render::replace_navigation(&config, &render::navigation(&report.records))?,
        );
        for authored in [
            "methodology.md",
            "automation-and-ci.md",
            "execution-and-evidence.md",
        ] {
            if !safe_path(root, format!("{}/{authored}", render::LAB))?.is_file() {
                return Err(format!("The authored Lab page {authored} is missing."));
            }
        }
        for path in outputs.keys() {
            let target = safe_path(root, path)?;
            if target.exists() && !target.is_file() {
                return Err(format!(
                    "Generated target is not a file: {}",
                    path.display()
                ));
            }
        }
        Ok(outputs)
    })();
    result.map_err(|e| vec![e])
}
/// `check` never writes; normal generation writes only stale generated targets.
pub fn generate(root: &Path, check: bool) -> Result<usize, Vec<String>> {
    let outputs = outputs(root)?;
    let mut errors = Vec::new();
    for (relative, content) in &outputs {
        let path = safe_path(root, relative).map_err(|e| vec![e])?;
        let existing = match fs::read_to_string(&path) {
            Ok(s) => Some(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(vec![format!("{}: {e}", relative.display())]),
        };
        if existing.as_ref() == Some(content) {
            continue;
        }
        if check {
            errors.push(format!(
                "{} is missing or stale; regenerate validation documentation.",
                relative.display()
            ));
        } else {
            let result = (|| -> std::io::Result<()> {
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(&path, content)
            })();
            if let Err(e) = result {
                return Err(vec![format!("{}: {e}", relative.display())]);
            }
        }
    }
    if errors.is_empty() {
        Ok(outputs.len() - 1)
    } else {
        Err(errors)
    }
}
