//! Native EXP-001 single-case runner and actual Rust toolchain observation.
use crate::runtime::{ensure, safe_path};
use serde_json::Value;
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::Command,
};
pub const CASES: &[(&str, &str)] = &[
    ("fifo_exactly_once", "TST-001"),
    ("capacity_before_write", "TST-002"),
    ("wraparound_partial_drains", "TST-003"),
    ("capacity_configuration", "TST-004"),
];
/// Build separately so compiler/launch/list failures are never assertion failures.
/// The Cargo test executable must list the exact named case before it can run.
pub fn run_case(
    root: &Path,
    case: &str,
    out: &mut impl Write,
    err: &mut impl Write,
) -> io::Result<u8> {
    let result = (|| -> Result<u8, String> {
        let identity = CASES
            .iter()
            .find(|(name, _)| *name == case)
            .map(|(_, id)| *id)
            .ok_or("Unknown EXP-001 case")?;
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let report_dir = std::env::var_os("CHRONOSHFT_REPORT_DIR")
            .unwrap_or_else(|| "target/validation-reports".into());
        let report = safe_path(
            &root,
            Path::new(&report_dir).join(format!("{identity}.txt")),
        )?;
        match fs::remove_file(report) {
            Ok(()) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
        let args = [
            "test",
            "--offline",
            "--locked",
            "--manifest-path",
            "Cargo.toml",
            "-p",
            "exp-001-spsc-queue",
            "--test",
            "queue_contracts",
            "--no-run",
            "--message-format=json",
        ];
        writeln!(out, "Build: cargo {}", args.join(" ")).map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;
        let build = Command::new("cargo")
            .args(args)
            .current_dir(&root)
            .output()
            .map_err(|e| e.to_string())?;
        err.write_all(&build.stderr).map_err(|e| e.to_string())?;
        let stdout = std::str::from_utf8(&build.stdout).map_err(|e| e.to_string())?;
        let mut executables = Vec::new();
        for line in stdout.lines() {
            let item: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
            if item["reason"] == "compiler-message" {
                write!(
                    err,
                    "{}",
                    item["message"]["rendered"].as_str().unwrap_or("")
                )
                .map_err(|e| e.to_string())?;
            }
            if item["reason"] == "compiler-artifact" && item["target"]["name"] == "queue_contracts"
            {
                if let Some(path) = item["executable"].as_str() {
                    executables.push(path.to_owned());
                }
            }
        }
        ensure(
            build.status.success() && executables.len() == 1,
            "build failed or test executable is ambiguous",
        )?;
        let executable = &executables[0];
        let list = Command::new(executable)
            .arg("--list")
            .current_dir(&root)
            .output()
            .map_err(|e| e.to_string())?;
        ensure(
            list.status.success()
                && String::from_utf8_lossy(&list.stdout)
                    .lines()
                    .any(|line| line == format!("{case}: test")),
            "requested test is missing",
        )?;
        writeln!(out, "Execute: {executable} {case} --exact --nocapture")
            .map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;
        let result = Command::new(executable)
            .args([case, "--exact", "--nocapture"])
            .current_dir(&root)
            .status()
            .map_err(|e| e.to_string())?;
        Ok(match result.code() {
            Some(0) => 0,
            Some(101) => 101,
            _ => 2,
        })
    })();
    match result {
        Ok(code) => Ok(code),
        Err(error) => {
            writeln!(err, "Tool error: {error}")?;
            Ok(2)
        }
    }
}
/// Print identities observed now; never invent versions from a manifest.
pub fn toolchain(out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    for tool in ["rustc", "cargo"] {
        writeln!(out, "{tool} --version --verbose")?;
        out.flush()?;
        match Command::new(tool).args(["--version", "--verbose"]).output() {
            Ok(result) => {
                out.write_all(&result.stdout)?;
                err.write_all(&result.stderr)?;
                if !result.status.success() {
                    writeln!(err, "Toolchain probe failed: {tool}")?;
                    return Ok(1);
                }
            }
            Err(error) => {
                writeln!(err, "Toolchain probe could not start {tool}: {error}")?;
                return Ok(1);
            }
        }
    }
    Ok(0)
}
