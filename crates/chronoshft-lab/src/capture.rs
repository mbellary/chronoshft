//! Predeclared, Git-pinned automated observations and retained artifact bundles.
use crate::{
    evidence,
    process::{self, Interrupt},
    publication::{self, immutable_copy, immutable_write, yaml_text},
    registry,
    runtime::*,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const SOURCE_PATHS: &[&str] = &[
    ".",
    ":(exclude)validation/executions/**",
    ":(exclude)validation/evidence-manifests/**",
    ":(exclude)validation/decisions/**",
    ":(exclude)validation/artifacts/**",
];
pub fn utc_now() -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    let date = chrono::DateTime::from_timestamp(
        i64::try_from(now.as_secs()).map_err(|e| e.to_string())?,
        now.subsec_micros() * 1000,
    )
    .ok_or("System timestamp out of range")?;
    Ok(date.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string())
}
pub fn shell_join(args: &[Value]) -> String {
    args.iter()
        .map(|v| {
            let s = text(v);
            if !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_@%+=:,./-".contains(&b))
            {
                s.to_owned()
            } else {
                format!("'{}'", s.replace('\'', "'\"'\"'"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn argv(v: &Value) -> Vec<String> {
    array(v).iter().map(|s| text(s).to_owned()).collect()
}
fn write_value(path: &Path, v: &Value) -> Check {
    fs::write(path, yaml_text(v)?).map_err(|e| e.to_string())
}
fn source_status(git: &Git, preflight: bool) -> Result<bool, String> {
    let mut args = if preflight {
        vec!["status", "--porcelain", "--untracked-files=all", "--"]
    } else {
        vec!["diff", "HEAD", "--"]
    };
    args.extend(SOURCE_PATHS);
    Ok(!git.read(&args)?.is_empty())
}
fn cpu() -> String {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.to_lowercase().starts_with("model name"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
        })
        .unwrap_or_else(|| std::env::consts::ARCH.to_string())
}
fn os() -> String {
    format!(
        "{} {} {}",
        std::env::consts::OS,
        fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_default()
            .trim(),
        std::env::consts::ARCH
    )
}
fn environment(plan: &Value) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();
    for name in [
        "PATH",
        "HOME",
        "SYSTEMROOT",
        "WINDIR",
        "TMPDIR",
        "TEMP",
        "TMP",
        "CARGO_HOME",
        "RUSTUP_HOME",
    ] {
        if let Ok(v) = std::env::var(name) {
            vars.insert(name.into(), v);
        }
    }
    vars.insert("LC_ALL".into(), "C".into());
    vars.insert("TZ".into(), "UTC".into());
    if let Some(map) = plan["environment_variables"].as_object() {
        for (name, value) in map {
            vars.insert(name.clone(), text(value).into());
        }
    }
    vars
}
struct Retention<'a> {
    root: &'a Path,
    plan: &'a Value,
    implementation: &'a str,
    commit: &'a str,
    evidence: Vec<Value>,
}
impl Retention<'_> {
    fn retain(
        &mut self,
        path: &Path,
        kind: &str,
        scope: &str,
        validations: &[Value],
    ) -> Result<String, String> {
        let (size, digest) = file_digest(path)?;
        let repository = self.plan["retention"] == "repository"
            && self.plan["max_repository_artifact_bytes"]
                .as_u64()
                .is_some_and(|max| size <= max);
        let relative = if repository {
            let relative = PathBuf::from(format!("validation/artifacts/sha256/{digest}"));
            let target = safe_path(self.root, &relative)?;
            if !target.exists() {
                match immutable_copy(path, &target) {
                    Ok(()) => (),
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                    Err(e) => return Err(e.to_string()),
                }
            }
            ensure(
                file_digest(&target)? == (size, digest.clone()),
                "Existing or concurrent content-addressed artifact differs from its digest",
            )?;
            relative
        } else {
            path.strip_prefix(self.root)
                .map_err(|_| "Retained artifact escapes repository")?
                .to_path_buf()
        };
        let run_id = text(&self.plan["run_id"]);
        let identity = format!(
            "EVD-{}-{:03}",
            run_id.strip_prefix("RUN-").unwrap_or(run_id),
            self.evidence.len() + 1
        );
        let media_type = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => "application/json",
            Some("log" | "yaml") => "text/plain",
            _ => "application/octet-stream",
        };
        self.evidence.push(json!({"schema_version":1,"id":identity,"kind":kind,"provenance":{"execution_id":run_id,"description":scope,"source_version":self.implementation,"retrieved_at":utc_now()?},"artifact":{"uri":format!("repo://{}",relative.to_string_lossy().replace('\\',"/")),"sha256":digest,"size_bytes":size,"media_type":media_type,"retention_location":if repository {"Repository content-addressed artifact"}else{"Run bundle; publish and supply a local mirror for verification"},"access":if repository {"Repository checkout"}else{"Retained run bundle or explicit artifact mirror"}},"supports":validations.iter().map(|v|json!({"validation":reference(text(&v["id"]),&v["revision"],self.commit),"scope":scope})).collect::<Vec<_>>(),"supersedes":null}));
        let mut permissions = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions).map_err(|e| e.to_string())?;
        Ok(identity)
    }
}
/// Capture a supplied plan. Preconditions are checked before reserving a RUN ID;
/// once reserved, its bundle is retained even on incomplete publication.
pub fn capture(root: &Path, plan: &Value, shared_ci: bool) -> Result<Value, String> {
    let interrupt = Interrupt::new()?;
    capture_with_interrupt(root, plan, shared_ci, &interrupt.flag)
}
/// Injectable cancellation for deterministic native tests; production callers use
/// `capture`, which installs SIGINT/SIGTERM handlers for the full observation.
pub fn capture_with_interrupt(
    root: &Path,
    plan: &Value,
    shared_ci: bool,
    interrupt: &std::sync::atomic::AtomicBool,
) -> Result<Value, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let git = Git::new(&root)?;
    let mut plan = plan.clone();
    ensure(plan.is_object(), "Capture plan must be a mapping")?;
    let checkout = git.resolve("HEAD")?;
    if plan["implementation_commit"] == "HEAD" {
        plan["implementation_commit"] = json!(checkout);
    }
    if plan["experiment"]["registry_commit"] == "HEAD" {
        plan["experiment"]["registry_commit"] = json!(checkout);
    }
    validate_record(&root, "capture-plan", &plan)?;
    let mut resolver = PinnedDefinitions::new(&root)?;
    let experiment = resolver.record(&plan["experiment"])?;
    let commit = text(&plan["experiment"]["registry_commit"]).to_owned();
    let implementation = text(&plan["implementation_commit"]).to_owned();
    resolver.commit(&implementation)?;
    ensure(
        git.resolve("HEAD")? == implementation,
        "Implementation commit must be the current checkout HEAD",
    )?;
    ensure(!source_status(&git,true)?,"Capture requires clean committed source; commit framework/bindings/inputs before a real run")?;
    ensure(
        matches!(
            text(&experiment["lifecycle"]["state"]),
            "ready" | "running" | "reviewing" | "completed"
        ),
        "Pinned experiment is not ready for execution",
    )?;
    let cwd = safe_path(&root, text(&plan["working_directory"]))?;
    ensure(cwd.is_dir(), "Declared working directory does not exist")?;
    let tests = array(&experiment["tests"])
        .iter()
        .map(|t| resolver.record(&reference(text(&t["id"]), &t["revision"], &commit)))
        .collect::<Result<Vec<_>, _>>()?;
    ensure(
        tests
            .iter()
            .all(|t| !flag(&t["mandatory"]) || t["execution_binding"]["state"] == "bound"),
        "All mandatory methods must be bound before experiment execution",
    )?;
    let selected: BTreeMap<_, _> = array(&plan["tests"])
        .iter()
        .map(|t| (text(&t["id"]).to_owned(), t.clone()))
        .collect();
    let expected: BTreeMap<_, _> = tests.iter().map(|t| (text(&t["id"]), t)).collect();
    ensure(
        selected.len() == array(&plan["tests"]).len()
            && selected.keys().all(|id| expected.contains_key(id.as_str())),
        "Capture plan has duplicate or foreign test identities",
    )?;
    for (identity, item) in &selected {
        let test = expected[identity.as_str()];
        ensure(
            !shared_ci || test["classification"] != "performance",
            "Shared CI cannot execute performance acceptance tests; use a controlled environment",
        )?;
        ensure(test["execution_type"]=="automated"&&test["revision"]==item["revision"],"Capture plans select exact automated test revisions; use execution import for manual observations")?;
        ensure(
            test["execution_binding"]["state"] == "bound"
                && item["command"]
                    == json!(split_binding(text(
                        &test["execution_binding"]["entrypoint"]
                    ))?),
            format!("{identity}: command does not match the pinned binding"),
        )?;
        resolver.blob(
            &implementation,
            text(&test["execution_binding"]["implementation_path"]),
        )?;
        for artifact in array(&item["artifacts"]) {
            safe_path(&cwd, text(&artifact["path"]))?;
        }
    }
    if shared_ci {
        plan["deviations"].as_array_mut().unwrap().push(json!(
            "Shared CI environment; not controlled performance acceptance evidence"
        ));
    }
    let mut inputs = Vec::new();
    for item in array(&plan["inputs"]) {
        let path = text(&item["path"]);
        let data = fs::read(safe_path(&root, path)?).map_err(|e| e.to_string())?;
        ensure(
            data == resolver.blob(&implementation, path)?,
            "Inputs must match their committed source bytes",
        )?;
        inputs.push(json!({"name":item["name"],"sha256":bytes_digest(&data),"reference":format!("repo://{path}")}));
    }
    let mut lockfiles = Vec::new();
    for relative in array(&plan["dependency_lockfiles"]) {
        let path = text(relative);
        let data = fs::read(safe_path(&root, path)?).map_err(|e| e.to_string())?;
        ensure(
            data == resolver.blob(&implementation, path)?,
            "Lockfile differs from its committed source",
        )?;
        lockfiles.push(json!({"path":path,"sha256":bytes_digest(&data)}));
    }
    let timeout = plan["timeout_seconds"]
        .as_f64()
        .and_then(|n| Duration::try_from_secs_f64(n).ok())
        .ok_or("Timeout is not a finite representable duration")?;
    ensure(!timeout.is_zero(), "Timeout must be positive")?;
    let run_id = text(&plan["run_id"]);
    ensure(
        !safe_path(&root, format!("validation/executions/{run_id}.yaml"))?.exists(),
        "Run IDs cannot be reused",
    )?;
    let bundle = safe_path(&root, format!("evidence/{run_id}"))?;
    fs::create_dir_all(bundle.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::create_dir(&bundle)
        .map_err(|e| format!("Run bundle cannot be reserved (never reuse its ID): {e}"))?;
    write_value(
        &bundle.join("capture-state.json"),
        &json!({"state":"capturing","run_id":run_id}),
    )?;
    immutable_write(&bundle.join("plan.yaml"), &yaml_text(&plan)?).map_err(|e| e.to_string())?;
    let started = utc_now()?;
    let validations = array(&experiment["validates"]);
    let environment = environment(&plan);
    let mut retention = Retention {
        root: &root,
        plan: &plan,
        implementation: &implementation,
        commit: &commit,
        evidence: Vec::new(),
    };
    retention.retain(
        &bundle.join("plan.yaml"),
        "other",
        "Exact predeclared capture plan; not a technical conclusion",
        validations,
    )?;
    let version_out = bundle.join("toolchain/stdout.log");
    let version_err = bundle.join("toolchain/stderr.log");
    let version = process::run(
        &argv(&plan["toolchain_command"]),
        &root,
        &version_out,
        &version_err,
        timeout,
        &environment,
        interrupt,
    )?;
    retention.retain(
        &version_out,
        "other",
        "Actual toolchain command output",
        validations,
    )?;
    retention.retain(
        &version_err,
        "other",
        "Actual toolchain command error output",
        validations,
    )?;
    let mut aborted = version.aborted;
    let mut tool_errors = Vec::new();
    if version.error || version.code != Some(0) {
        tool_errors.push(format!(
            "Toolchain probe failed: {}",
            if version.details.is_empty() {
                version
                    .code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "None".into())
            } else {
                version.details
            }
        ));
    }
    let toolchain = format!(
        "{}{}",
        String::from_utf8_lossy(&fs::read(&version_out).map_err(|e| e.to_string())?),
        String::from_utf8_lossy(&fs::read(&version_err).map_err(|e| e.to_string())?)
    )
    .trim()
    .to_owned();
    let observed_environment = json!({"toolchain":if toolchain.is_empty(){"Toolchain probe produced no version output"}else{&toolchain},"os":os(),"cpu":cpu(),"configuration":format!("{}\nProcess environment: {}",text(&plan["configuration"]),serde_json::to_string(&environment).map_err(|e|e.to_string())?),"deviations":plan["deviations"]});
    let mut results = Vec::new();
    let mut source_changed = source_status(&git, false)?
        || git.resolve("HEAD")? != implementation
        || inputs.iter().any(|i| {
            safe_path(&root, text(&i["reference"]).trim_start_matches("repo://"))
                .and_then(|p| file_digest(&p))
                .map(|(_, d)| d != text(&i["sha256"]))
                .unwrap_or(true)
        });
    if source_changed {
        tool_errors.push("Tracked source or a pinned input changed during execution".into());
    }
    let mut stop = !tool_errors.is_empty() || aborted || source_changed;
    for test in &tests {
        let identity = text(&test["id"]);
        let selected = selected.get(identity);
        let mut case = json!({"test":reference(identity,&test["revision"],&commit),"outcome":"skipped","details":"Not selected in this capture plan","command":selected.map(|i|i["command"].clone()).unwrap_or(json!([])),"exit_code":null,"duration_seconds":0,"evidence_ids":[]});
        if selected.is_none() || stop {
            if stop {
                case["details"] = json!(
                    "Not launched after interruption, infrastructure error or source/input change"
                );
            }
            results.push(case);
            continue;
        }
        let item = selected.unwrap();
        let out = bundle.join(identity).join("stdout.log");
        let err = bundle.join(identity).join("stderr.log");
        let observed = process::run(
            &argv(&item["command"]),
            &cwd,
            &out,
            &err,
            timeout,
            &environment,
            interrupt,
        )?;
        aborted |= observed.aborted;
        let outcome = if observed.error {
            "error"
        } else if observed.code == Some(0) {
            "passed"
        } else if observed
            .code
            .is_some_and(|c| array(&item["failure_exit_codes"]).contains(&json!(c)))
        {
            "failed"
        } else {
            "error"
        };
        case["outcome"] = json!(outcome);
        case["details"] = json!(if observed.details.is_empty() {
            format!(
                "Bound process exited with code {}",
                observed
                    .code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "None".into())
            )
        } else {
            observed.details
        });
        case["exit_code"] = json!(observed.code);
        case["duration_seconds"] = json!(observed.duration);
        let mut case_evidence = vec![
            retention.retain(
                &out,
                "test_report",
                &format!("{identity} standard output; observation, not automatic closure evidence"),
                array(&test["validations"]),
            )?,
            retention.retain(
                &err,
                "test_report",
                &format!("{identity} standard error"),
                array(&test["validations"]),
            )?,
        ];
        for (index, artifact) in array(&item["artifacts"]).iter().enumerate() {
            let retained = (|| -> Result<String, String> {
                let source = safe_path(&cwd, text(&artifact["path"]))?;
                let dest = bundle
                    .join(identity)
                    .join(format!("artifact-{:03}", index + 1));
                immutable_copy(&source, &dest).map_err(|e| e.to_string())?;
                retention.retain(
                    &dest,
                    text(&artifact["kind"]),
                    &format!("{identity} declared output {}", text(&artifact["path"])),
                    array(&test["validations"]),
                )
            })();
            match retained {
                Ok(id) => case_evidence.push(id),
                Err(e) => {
                    case["outcome"] = json!("error");
                    case["details"] = json!(format!(
                        "{}; required artifact unavailable: {e}",
                        text(&case["details"])
                    ));
                }
            }
        }
        case["evidence_ids"] = json!(case_evidence);
        let input_changed = inputs.iter().any(|i| {
            let path = text(&i["reference"]).strip_prefix("repo://").unwrap();
            safe_path(&root, path)
                .and_then(|p| file_digest(&p))
                .map(|(_, d)| d != text(&i["sha256"]))
                .unwrap_or(true)
        });
        source_changed |=
            source_status(&git, false)? || input_changed || git.resolve("HEAD")? != implementation;
        if source_changed {
            tool_errors.push("Tracked source or a pinned input changed during execution".into());
        }
        if case["outcome"] == "error" {
            tool_errors.push(format!("{identity}: {}", text(&case["details"])));
        }
        results.push(case);
        stop = source_changed || aborted;
    }
    // A probe may itself change source. Recheck even if no case was launched.
    let input_changed = inputs.iter().any(|i| {
        safe_path(&root, text(&i["reference"]).trim_start_matches("repo://"))
            .and_then(|p| file_digest(&p))
            .map(|(_, digest)| digest != text(&i["sha256"]))
            .unwrap_or(true)
    });
    let final_changed =
        source_status(&git, false)? || input_changed || git.resolve("HEAD")? != implementation;
    if final_changed && !source_changed {
        source_changed = true;
        tool_errors.push("Tracked source or a pinned input changed during execution".into());
    }
    aborted |= interrupt.load(std::sync::atomic::Ordering::SeqCst);
    let ended = utc_now()?;
    let outcome = evidence::aggregate(&results, aborted, source_changed, !tool_errors.is_empty());
    let mut procedure = vec![
        format!("Working directory: {}", text(&plan["working_directory"])),
        format!(
            "Toolchain probe: {}",
            shell_join(array(&plan["toolchain_command"]))
        ),
    ];
    procedure.extend(
        results
            .iter()
            .filter(|r| r["outcome"] != "skipped")
            .map(|r| shell_join(array(&r["command"]))),
    );
    let report_id = format!(
        "EVD-{}-{:03}",
        run_id.trim_start_matches("RUN-"),
        retention.evidence.len() + 1
    );
    let mut evidence_ids: Vec<_> = retention
        .evidence
        .iter()
        .map(|e| text(&e["id"]).to_owned())
        .collect();
    evidence_ids.push(report_id.clone());
    let messages = |state: &str| {
        results
            .iter()
            .filter(|r| r["outcome"] == state)
            .map(|r| format!("{}: {}", text(&r["test"]["id"]), text(&r["details"])))
            .collect::<Vec<_>>()
    };
    let run = json!({"schema_version":1,"id":run_id,"started_at":started,"ended_at":ended,"experiment":plan["experiment"],"working_directory":plan["working_directory"],"tests":results,"outcome":outcome,"procedure":procedure,"source":{"implementation_commit":implementation,"registry_commit":commit,"dirty":false,"changed_during_run":source_changed,"patch_evidence_id":null,"dependency_lockfiles":lockfiles},"environment":observed_environment,"inputs":inputs,"random_seeds":plan["random_seeds"],"instrumentation":plan["instrumentation"],"evidence_ids":evidence_ids,"failures":messages("failed"),"skips":messages("skipped"),"tool_errors":tool_errors,"supersedes":null});
    immutable_write(
        &bundle.join("results.json"),
        &yaml_text(&json!({"run":run,"capture_plan":plan}))?,
    )
    .map_err(|e| e.to_string())?;
    let retained = retention.retain(
        &bundle.join("results.json"),
        "test_report",
        "Structured outcomes and reproduction metadata; review required",
        validations,
    )?;
    ensure(
        retained == report_id,
        "Structured report identity differs from the final manifest",
    )?;
    immutable_write(
        &bundle.join(format!("manifests/{run_id}.yaml")),
        &yaml_text(&run)?,
    )
    .map_err(|e| e.to_string())?;
    for item in &retention.evidence {
        immutable_write(
            &bundle.join(format!("manifests/evidence/{}.yaml", text(&item["id"]))),
            &yaml_text(item)?,
        )
        .map_err(|e| e.to_string())?;
    }
    publication::publish_run(
        &root,
        &run,
        &retention.evidence,
        &ArtifactOptions::default(),
        false,
    )?;
    write_value(
        &bundle.join("capture-state.json"),
        &json!({"state":"published","run_id":run_id,"outcome":outcome}),
    )?;
    Ok(run)
}
/// Validate the optional committed-plan boundary before applying the identity
/// override. Local external plans are explicit operator inputs, never implicit.
pub fn from_file(
    root: &Path,
    path: &Path,
    run_id: Option<&str>,
    committed: bool,
    shared_ci: bool,
) -> Result<Value, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    if committed {
        let relative = path
            .strip_prefix(root.join("validation/execution-plans"))
            .map_err(|_| "CI requires a committed plan under validation/execution-plans")?;
        ensure(
            !relative.as_os_str().is_empty(),
            "Committed plan must be a file",
        )?;
        let relative = path
            .strip_prefix(&root)
            .unwrap()
            .to_str()
            .ok_or("Non-Unicode plan path")?;
        ensure(
            {
                let git = Git::new(&root)?;
                git.blob(&git.resolve("HEAD")?, relative)?
            } == fs::read(&path).map_err(|e| e.to_string())?,
            "The plan differs from the committed source",
        )?;
    }
    let mut plan = registry::read_yaml(&path)?;
    ensure(plan.is_object(), "Capture plan must be a mapping")?;
    if let Some(id) = run_id {
        plan["run_id"] = json!(id);
    }
    capture(&root, &plan, shared_ci)
}
