//! Shared fail-fast local/CI gate. MkDocs remains an external locked Python tool.
use std::{
    io::{self, Write},
    path::Path,
    process::Command,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub label: &'static str,
    pub argv: Vec<String>,
}
/// Explicit argument arrays; no shell interpolation and no automatic repair,
/// capture, decision publication or deployment operation is part of this gate.
pub fn plan(root: &Path, executable: &Path) -> Vec<Step> {
    let native = |label, args: &[&str]| {
        let mut argv = vec![executable.to_string_lossy().into_owned()];
        argv.extend(args.iter().map(|s| s.to_string()));
        argv.extend(["--root".into(), root.to_string_lossy().into_owned()]);
        Step { label, argv }
    };
    vec![
        native(
            "Registry schemas, references and mandatory coverage",
            &["registry", "check", "--require-tests"],
        ),
        native("Git history and immutable ledgers", &["history", "check"]),
        native(
            "Pinned execution, artifact integrity and decision evidence",
            &["evidence", "verify"],
        ),
        native(
            "Generated Lab pages and navigation",
            &["docs", "generate", "--check"],
        ),
        Step {
            label: "Native registry, runtime, review, documentation and CI regressions",
            argv: ["cargo", "test", "--offline", "--locked", "--workspace"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        },
        Step {
            label: "Strict MkDocs build",
            argv: [
                "uv", "run", "--locked", "--extra", "docs", "python", "-m", "mkdocs", "build",
                "--strict",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        },
        native("Rendered local links and anchors", &["docs", "links"]),
    ]
}
/// Injectable process boundary lets native tests exercise failure at every stage
/// without launching a recursive Cargo test run or a publishing process.
pub fn run_with(
    root: &Path,
    executable: &Path,
    out: &mut impl Write,
    err: &mut impl Write,
    mut runner: impl FnMut(&Step, &Path) -> io::Result<u8>,
) -> io::Result<u8> {
    let root = match root.canonicalize() {
        Ok(root) if root.is_dir() => root,
        Ok(_) => {
            writeln!(err, "Gate root must be a repository directory")?;
            return Ok(1);
        }
        Err(e) => {
            writeln!(err, "Cannot open gate root: {e}")?;
            return Ok(1);
        }
    };
    let steps = plan(&root, executable);
    for (index, step) in steps.iter().enumerate() {
        writeln!(out, "[{}/{}] {}", index + 1, steps.len(), step.label)?;
        out.flush()?;
        match runner(step, &root) {
            Ok(0) => (),
            Ok(code) => {
                writeln!(
                    err,
                    "{}: failed; remaining checks were not run.",
                    step.label
                )?;
                return Ok(code);
            }
            Err(e) => {
                writeln!(err, "{}: could not start check: {e}", step.label)?;
                return Ok(1);
            }
        }
    }
    writeln!(out,"Documentation and registry gates passed. Experiment execution and validation acceptance are separate gates.")?;
    Ok(0)
}
pub fn run(root: &Path, out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    let executable = std::env::current_exe()?;
    run_with(root, &executable, out, err, |step, cwd| {
        let status = Command::new(&step.argv[0])
            .args(&step.argv[1..])
            .current_dir(cwd)
            .status()?;
        Ok(status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .unwrap_or(1))
    })
}
