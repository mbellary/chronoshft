//! Native validation/documentation routing; later migration operations fail closed.
use crate::migration::{BASELINE, STAGES};
use std::io::{self, Write};

pub const SUCCESS: u8 = 0;
pub const USAGE_ERROR: u8 = 2;

const HELP: &str = "ChronosHFT Technical Validation Lab — Rust migration Stage 8

Usage: chronoshft-lab <command>

Available now:
  --help, -h                 Show this help
  --version, -V              Show the tooling version
  migration status           Show migration progress and compatibility baseline
  history check              Check revision history and immutable ledgers
    --root PATH --base-ref REF
  evidence verify            Verify pinned observations, artifacts and decisions
    --root PATH --artifact-map PATH --artifact-root PATH
  registry check             Validate schemas, references and mandatory coverage
    --root PATH              Repository root (default: compiled source repository)
    --require-coverage       Require mandatory investigation coverage
    --require-tests          Also require hypothesis, method and test coverage

  docs generate              Generate Lab views (--check is read-only)
    --root PATH --check
  docs links                 Check rendered local links and anchors
    --root PATH

  execution capture          Capture bound observations and retained evidence
    --plan PATH --root PATH --run-id ID --committed-plan --shared-ci
  execution import           Import a supplied execution and evidence
    --run PATH --evidence-dir PATH --root PATH --artifact-map PATH --artifact-root PATH --resume
  experiment run-case CASE   Build and run one declared EXP-001 case (--root PATH)
  toolchain show             Observe compiler and package-manager versions

  review prepare             Prepare an unapproved review package
    --validation VAL-ID --decision-id VDEC-ID --output PATH --root PATH
  review check               Check a supplied human decision (read-only by default)
    --candidate PATH --root PATH --artifact-map PATH --artifact-root PATH
    --register-reviewed      Explicit human-review attestation; append a new VDEC only

  check                      Run the shared local/CI gate, including native tests and MkDocs
    --root PATH

Rust validation, documentation, capture/import, experiment, reviewer and shared-gate workflows are available; custom Python validation tools are retired. MkDocs, Material,
Python/uv documentation dependencies and Pages publication remain supported.
";

/// Route validation checks and generated-document updates. History/evidence
/// checks invoke read-only Git commands. Capture/import publish observations;
/// supplied human decisions register only with explicit attestation.
/// Output failures propagate so the executable cannot report false success.
pub fn run(args: &[String], out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match words.as_slice() {
        [] | ["--help" | "-h"] => {
            write!(out, "{HELP}")?;
            Ok(SUCCESS)
        }
        ["--version" | "-V"] => {
            writeln!(out, "chronoshft-lab {}", env!("CARGO_PKG_VERSION"))?;
            Ok(SUCCESS)
        }
        ["migration", "status"] => {
            for stage in STAGES {
                writeln!(
                    out,
                    "Stage {}: {} — {}",
                    stage.number, stage.title, stage.state
                )?;
            }
            writeln!(out, "Compatibility baseline: {BASELINE}")?;
            writeln!(
                out,
                "Rust validation, documentation, capture/import, experiment, reviewer and shared-gate workflows are available; custom Python validation tools are retired."
            )?;
            writeln!(out, "Documentation: MkDocs and Material remain active.")?;
            Ok(SUCCESS)
        }
        ["registry", "check", tail @ ..] => registry_check(tail, out, err),
        ["history", "check", tail @ ..] => history_check(tail, out, err),
        ["evidence", "verify", tail @ ..] => evidence_verify(tail, out, err),
        ["docs", operation @ ("generate" | "links"), tail @ ..] => {
            docs_command(operation, tail, out, err)
        }
        ["execution", operation @ ("capture" | "import"), tail @ ..] => {
            execution_command(operation, tail, out, err)
        }
        ["experiment", "run-case", tail @ ..] => experiment_case(tail, out, err),
        ["toolchain", "show"] => crate::experiment::toolchain(out, err),
        ["toolchain", "show", "--help" | "-h"] => {
            writeln!(out, "Usage: chronoshft-lab toolchain show")?;
            Ok(0)
        }
        ["review", operation @ ("prepare" | "check"), tail @ ..] => {
            review_command(operation, tail, out, err)
        }
        ["check", tail @ ..] => gate_check(tail, out, err),
        _ => {
            writeln!(
                err,
                "Unknown or incomplete command. Run chronoshft-lab --help."
            )?;
            Ok(USAGE_ERROR)
        }
    }
}

fn registry_check(args: &[&str], out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    use crate::registry::{self, Options};
    use std::path::PathBuf;
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        match args[index] {
            "--help" | "-h" => {
                writeln!(out, "Usage: chronoshft-lab registry check [--root PATH] [--require-coverage] [--require-tests]")?;
                return Ok(SUCCESS);
            }
            "--root" if index + 1 < args.len() && !args[index + 1].starts_with("--") => {
                index += 1;
                root = args[index].into();
            }
            value if value.starts_with("--root=") && value.len() > 7 => root = value[7..].into(),
            "--require-coverage" => options.require_coverage = true,
            "--require-tests" => options.require_tests = true,
            value => {
                writeln!(err, "Invalid registry option or missing value: {value}")?;
                return Ok(USAGE_ERROR);
            }
        }
        index += 1;
    }
    let report = registry::validate(&root, options);
    if !report.accepted() {
        for error in report.errors {
            writeln!(err, "{error}")?;
        }
        return Ok(1);
    }
    writeln!(
        out,
        "Validated {} records across {} registries; 28 historical findings preserved.",
        report.records.len(),
        registry::CATALOGS.len()
    )?;
    writeln!(
        out,
        "Registry consistency only; execution evidence and reviewer acceptance are separate gates."
    )?;
    Ok(SUCCESS)
}

fn verification_options(
    args: &[&str],
    allowed: &[&str],
) -> Result<Option<std::collections::BTreeMap<String, String>>, String> {
    let mut values = std::collections::BTreeMap::new();
    let mut index = 0;
    while index < args.len() {
        let value = args[index];
        if matches!(value, "--help" | "-h") {
            return Ok(None);
        }
        if let Some((key, value)) = value.split_once('=') {
            if allowed.contains(&key) && !value.is_empty() {
                values.insert(key.into(), value.into());
            } else {
                return Err(format!("Invalid option or missing value: {key}"));
            }
        } else if allowed.contains(&value)
            && index + 1 < args.len()
            && !args[index + 1].starts_with("--")
        {
            index += 1;
            values.insert(value.into(), args[index].into());
        } else {
            return Err(format!("Invalid option or missing value: {value}"));
        }
        index += 1;
    }
    Ok(Some(values))
}

fn history_check(args: &[&str], out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    let values = match verification_options(args, &["--root", "--base-ref"]) {
        Ok(Some(values)) => values,
        Ok(None) => {
            writeln!(
                out,
                "Usage: chronoshft-lab history check [--root PATH] [--base-ref REF]"
            )?;
            return Ok(0);
        }
        Err(error) => {
            writeln!(err, "{error}")?;
            return Ok(USAGE_ERROR);
        }
    };
    let root = values
        .get("--root")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let result = (|| -> Result<(Vec<String>, bool), String> {
        let git = crate::runtime::Git::new(&root)?;
        let base = if let Some(base) = values.get("--base-ref") {
            crate::history::require_base(base)?;
            Some(base.clone())
        } else {
            let event = std::env::var("GITHUB_EVENT_NAME").ok();
            let path = std::env::var_os("GITHUB_EVENT_PATH").map(std::path::PathBuf::from);
            crate::history::automatic_base(&git, event.as_deref(), path.as_deref())?
        };
        Ok((
            crate::history::check(&root, base.as_deref())?,
            base.is_some(),
        ))
    })();
    match result {
        Ok((errors, has_base)) if errors.is_empty() => {
            writeln!(
                out,
                "{}",
                if has_base {
                    "Registry history and immutable ledgers match the Git baseline."
                } else {
                    "Initial repository event: no earlier Git baseline; current schema/coverage checks still apply."
                }
            )?;
            Ok(0)
        }
        Ok((errors, _)) => {
            for error in errors {
                writeln!(err, "{error}")?;
            }
            Ok(1)
        }
        Err(error) => {
            writeln!(err, "History verification failed: {error}")?;
            Ok(1)
        }
    }
}

fn evidence_verify(args: &[&str], out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    let values = match verification_options(args, &["--root", "--artifact-map", "--artifact-root"])
    {
        Ok(Some(values)) => values,
        Ok(None) => {
            writeln!(out,"Usage: chronoshft-lab evidence verify [--root PATH] [--artifact-map PATH] [--artifact-root PATH]")?;
            return Ok(0);
        }
        Err(error) => {
            writeln!(err, "{error}")?;
            return Ok(USAGE_ERROR);
        }
    };
    let root = values
        .get("--root")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let result = (|| -> Result<crate::evidence::Verification, String> {
        let map = values
            .get("--artifact-map")
            .cloned()
            .or_else(|| std::env::var("VALIDATION_ARTIFACT_MAP").ok())
            .filter(|v| !v.is_empty());
        let options = crate::runtime::ArtifactOptions {
            mapping: if let Some(map) = map {
                crate::runtime::read_artifact_map(std::path::Path::new(&map))?
            } else {
                Default::default()
            },
            root: values.get("--artifact-root").map(std::path::PathBuf::from),
        };
        crate::evidence::verify_all(&root, &options)
    })();
    match result {
        Ok(report) if report.errors.is_empty() => {
            let count = |prefix: &str| {
                report
                    .records
                    .keys()
                    .filter(|id| id.starts_with(prefix))
                    .count()
            };
            writeln!(out,"Verified {} executions, {} artifact manifests and {} decisions. No technical approval is inferred.",count("RUN-"),count("EVD-"),count("VDEC-"))?;
            Ok(0)
        }
        Ok(report) => {
            for error in report.errors {
                writeln!(err, "{error}")?;
            }
            Ok(1)
        }
        Err(error) => {
            writeln!(err, "Evidence verification failed: {error}")?;
            Ok(1)
        }
    }
}

fn docs_command(
    operation: &str,
    args: &[&str],
    out: &mut impl Write,
    err: &mut impl Write,
) -> io::Result<u8> {
    let check = args.contains(&"--check");
    let filtered: Vec<_> = if operation == "generate" {
        args.iter().copied().filter(|s| *s != "--check").collect()
    } else {
        args.to_vec()
    };
    let values = match verification_options(&filtered, &["--root"]) {
        Ok(Some(values)) => values,
        Ok(None) => {
            writeln!(
                out,
                "Usage: chronoshft-lab docs {operation} [--root PATH]{}",
                if operation == "generate" {
                    " [--check]"
                } else {
                    ""
                }
            )?;
            return Ok(0);
        }
        Err(error) => {
            writeln!(err, "{error}")?;
            return Ok(USAGE_ERROR);
        }
    };
    let root = values
        .get("--root")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    if operation == "generate" {
        match crate::docs::generate(&root, check) {
            Ok(count) => {
                writeln!(
                    out,
                    "{} {count} validation pages and Lab navigation.",
                    if check { "Checked" } else { "Generated" }
                )?;
                Ok(0)
            }
            Err(errors) => {
                for error in errors {
                    writeln!(err, "{error}")?;
                }
                Ok(1)
            }
        }
    } else {
        match crate::docs::links::configured(&root) {
            Ok(report) if report.errors.is_empty() => {
                writeln!(
                    out,
                    "Checked {} local links and anchors across {} rendered HTML pages.",
                    report.checked, report.pages
                )?;
                Ok(0)
            }
            Ok(report) => {
                for error in report.errors {
                    writeln!(err, "{error}")?;
                }
                Ok(1)
            }
            Err(error) => {
                writeln!(err, "Unable to check rendered documentation: {error}")?;
                Ok(1)
            }
        }
    }
}

fn execution_command(
    operation: &str,
    args: &[&str],
    out: &mut impl Write,
    err: &mut impl Write,
) -> io::Result<u8> {
    let flags = if operation == "capture" {
        &["--committed-plan", "--shared-ci"][..]
    } else {
        &["--resume"][..]
    };
    let tail: Vec<_> = args
        .iter()
        .copied()
        .filter(|s| !flags.contains(s))
        .collect();
    let allowed = if operation == "capture" {
        &["--root", "--plan", "--run-id"][..]
    } else {
        &[
            "--root",
            "--run",
            "--evidence-dir",
            "--artifact-map",
            "--artifact-root",
        ][..]
    };
    // A boolean flag cannot fill an option's missing value. Check before
    // removing flags so a malformed invocation cannot consume a later path.
    for pair in args.windows(2) {
        if allowed.contains(&pair[0]) && pair[1].starts_with('-') {
            writeln!(err, "Missing value for {}", pair[0])?;
            return Ok(USAGE_ERROR);
        }
    }
    let values = match verification_options(&tail, allowed) {
        Ok(Some(values)) => values,
        Ok(None) => {
            writeln!(
                out,
                "Usage: chronoshft-lab execution {operation} {}",
                if operation == "capture" {
                    "--plan PATH [--root PATH] [--run-id ID] [--committed-plan] [--shared-ci]"
                } else {
                    "--run PATH --evidence-dir PATH [--root PATH] [--artifact-map PATH] [--artifact-root PATH] [--resume]"
                }
            )?;
            return Ok(0);
        }
        Err(error) => {
            writeln!(err, "{error}")?;
            return Ok(USAGE_ERROR);
        }
    };
    let required = if operation == "capture" {
        &["--plan"][..]
    } else {
        &["--run", "--evidence-dir"][..]
    };
    for option in required {
        if !values.contains_key(*option) {
            writeln!(err, "Missing required option: {option}")?;
            return Ok(USAGE_ERROR);
        }
    }
    let root = values
        .get("--root")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let result = if operation == "capture" {
        crate::capture::from_file(
            &root,
            std::path::Path::new(&values["--plan"]),
            values.get("--run-id").map(String::as_str),
            args.contains(&"--committed-plan"),
            args.contains(&"--shared-ci"),
        )
    } else {
        (|| -> Result<serde_json::Value, String> {
            let mapping = values
                .get("--artifact-map")
                .cloned()
                .or_else(|| std::env::var("VALIDATION_ARTIFACT_MAP").ok())
                .filter(|v| !v.is_empty());
            let options = crate::runtime::ArtifactOptions {
                mapping: if let Some(path) = mapping {
                    crate::runtime::read_artifact_map(std::path::Path::new(&path))?
                } else {
                    Default::default()
                },
                root: values.get("--artifact-root").map(std::path::PathBuf::from),
            };
            crate::publication::import(
                &root,
                std::path::Path::new(&values["--run"]),
                std::path::Path::new(&values["--evidence-dir"]),
                &options,
                args.contains(&"--resume"),
            )
        })()
    };
    match result {
        Ok(run) => {
            writeln!(
                out,
                "{} {}: {}. Evidence retained; no validation decision created.",
                if operation == "capture" {
                    "Recorded"
                } else {
                    "Registered"
                },
                crate::runtime::text(&run["id"]),
                crate::runtime::text(&run["outcome"])
            )?;
            Ok(if operation == "capture" && run["outcome"] != "passed" {
                1
            } else {
                0
            })
        }
        Err(error) => {
            writeln!(err,"Execution {operation} failed: {error}. Retain any reserved run bundle; do not reuse its RUN ID.")?;
            Ok(1)
        }
    }
}
fn experiment_case(args: &[&str], out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    if args.iter().any(|s| matches!(*s, "--help" | "-h")) {
        writeln!(
            out,
            "Usage: chronoshft-lab experiment run-case CASE [--root PATH]\nCases: {}",
            crate::experiment::CASES
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        return Ok(0);
    }
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut case = None;
    let mut index = 0;
    while index < args.len() {
        let arg = args[index];
        if arg == "--root" && index + 1 < args.len() && !args[index + 1].starts_with("--") {
            index += 1;
            root = args[index].into();
        } else if let Some(path) = arg.strip_prefix("--root=").filter(|s| !s.is_empty()) {
            root = path.into();
        } else if crate::experiment::CASES
            .iter()
            .any(|(name, _)| *name == arg)
            && case.is_none()
        {
            case = Some(arg);
        } else {
            writeln!(err, "Invalid experiment argument or missing value: {arg}")?;
            return Ok(USAGE_ERROR);
        }
        index += 1;
    }
    let Some(case) = case else {
        writeln!(err, "Missing EXP-001 case; use experiment run-case --help.")?;
        return Ok(USAGE_ERROR);
    };
    crate::experiment::run_case(&root, case, out, err)
}

fn review_command(
    operation: &str,
    args: &[&str],
    out: &mut impl Write,
    err: &mut impl Write,
) -> io::Result<u8> {
    let allowed = if operation == "prepare" {
        &["--root", "--validation", "--decision-id", "--output"][..]
    } else {
        &["--root", "--candidate", "--artifact-map", "--artifact-root"][..]
    };
    let register_reviewed = operation == "check" && args.contains(&"--register-reviewed");
    for pair in args.windows(2) {
        if allowed.contains(&pair[0]) && pair[1].starts_with('-') {
            writeln!(err, "Missing value for {}", pair[0])?;
            return Ok(USAGE_ERROR);
        }
    }
    let tail: Vec<_> = args
        .iter()
        .copied()
        .filter(|s| !(operation == "check" && *s == "--register-reviewed"))
        .collect();
    let values = match verification_options(&tail, allowed) {
        Ok(Some(values)) => values,
        Ok(None) => {
            writeln!(
                out,
                "Usage: chronoshft-lab review {operation} {}",
                if operation == "prepare" {
                    "--validation VAL-ID --decision-id VDEC-ID --output PATH [--root PATH]"
                } else {
                    "--candidate PATH [--root PATH] [--artifact-map PATH] [--artifact-root PATH] [--register-reviewed]"
                }
            )?;
            if operation == "check" {
                writeln!(out,"--register-reviewed explicitly attests that the supplied named human review happened; it appends a new decision only.")?;
            }
            return Ok(0);
        }
        Err(error) => {
            writeln!(err, "{error}")?;
            return Ok(USAGE_ERROR);
        }
    };
    for required in if operation == "prepare" {
        &["--validation", "--decision-id", "--output"][..]
    } else {
        &["--candidate"][..]
    } {
        if !values.contains_key(*required) {
            writeln!(err, "Missing required option: {required}")?;
            return Ok(USAGE_ERROR);
        }
    }
    let root = values
        .get("--root")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let result = if operation == "prepare" {
        crate::review::prepare(
            &root,
            &values["--validation"],
            &values["--decision-id"],
            std::path::Path::new(&values["--output"]),
        )
    } else {
        (|| -> Result<serde_json::Value, String> {
            let candidate =
                crate::registry::read_yaml(std::path::Path::new(&values["--candidate"]))?;
            let map = values
                .get("--artifact-map")
                .cloned()
                .or_else(|| std::env::var("VALIDATION_ARTIFACT_MAP").ok())
                .filter(|v| !v.is_empty());
            let options = crate::runtime::ArtifactOptions {
                mapping: if let Some(path) = map {
                    crate::runtime::read_artifact_map(std::path::Path::new(&path))?
                } else {
                    Default::default()
                },
                root: values.get("--artifact-root").map(std::path::PathBuf::from),
            };
            crate::review::check(&root, &candidate, &options, register_reviewed)
        })()
    };
    match result {
        Ok(_) => {
            writeln!(
                out,
                "{}",
                if operation == "prepare" {
                    "Prepared an unapproved review package outside the registry."
                } else if register_reviewed {
                    "Registered the supplied human decision. Current-decision selection and lifecycle changes remain explicit record edits."
                } else {
                    "Decision checks passed; no decision or approval was registered."
                }
            )?;
            Ok(0)
        }
        Err(error) => {
            writeln!(err, "Review failed: {error}")?;
            Ok(1)
        }
    }
}

fn gate_check(args: &[&str], out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    let values = match verification_options(args, &["--root"]) {
        Ok(Some(values)) => values,
        Ok(None) => {
            writeln!(out,"Usage: chronoshft-lab check [--root PATH]\nRun native consistency/evidence/freshness checks and Cargo regressions, then the locked strict MkDocs build and rendered links. No source repair, capture or decision publication occurs.")?;
            return Ok(0);
        }
        Err(e) => {
            writeln!(err, "{e}")?;
            return Ok(USAGE_ERROR);
        }
    };
    let root = values
        .get("--root")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    crate::gate::run(&root, out, err)
}
