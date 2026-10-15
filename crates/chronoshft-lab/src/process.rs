//! File-backed child supervision; no shell, bounded waits, process-group cleanup.
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub struct Interrupt {
    pub flag: Arc<AtomicBool>,
    #[cfg(unix)]
    handlers: Vec<signal_hook::SigId>,
}
impl Interrupt {
    pub fn new() -> Result<Self, String> {
        let flag = Arc::new(AtomicBool::new(false));
        #[cfg(unix)]
        {
            let mut handlers = Vec::new();
            for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
                match signal_hook::flag::register(signal, flag.clone()) {
                    Ok(id) => handlers.push(id),
                    Err(e) => {
                        for id in handlers {
                            signal_hook::low_level::unregister(id);
                        }
                        return Err(e.to_string());
                    }
                }
            }
            Ok(Self { flag, handlers })
        }
        #[cfg(not(unix))]
        {
            Ok(Self { flag })
        }
    }
}
impl Drop for Interrupt {
    fn drop(&mut self) {
        #[cfg(unix)]
        for id in &self.handlers {
            signal_hook::low_level::unregister(*id);
        }
    }
}
struct Group {
    child: Child,
    armed: bool,
}
impl Group {
    fn kill(&mut self) -> Result<(), String> {
        #[cfg(unix)]
        {
            use nix::{
                errno::Errno,
                sys::signal::{killpg, Signal},
                unistd::Pid,
            };
            let pid = i32::try_from(self.child.id()).map_err(|e| e.to_string())?;
            match killpg(Pid::from_raw(pid), Signal::SIGKILL) {
                Ok(()) | Err(Errno::ESRCH) => Ok(()),
                Err(e) => Err(format!("Cannot terminate child process group: {e}")),
            }
        }
        #[cfg(not(unix))]
        {
            self.child.kill().map_err(|e| e.to_string())
        }
    }
}
impl Drop for Group {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.kill();
            let _ = self.child.wait();
        }
    }
}
#[derive(Debug)]
pub struct Observation {
    pub code: Option<i32>,
    pub error: bool,
    pub aborted: bool,
    pub duration: f64,
    pub details: String,
}
fn status_code(status: std::process::ExitStatus) -> Option<i32> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.code().or_else(|| status.signal().map(|s| -s))
    }
    #[cfg(not(unix))]
    {
        status.code()
    }
}
/// Ordinary child descendants inherit the group. Kill surviving descendants even
/// when the direct child exits normally, before logs are hashed/finalized.
pub fn run(
    argv: &[String],
    cwd: &Path,
    stdout: &Path,
    stderr: &Path,
    timeout: Duration,
    environment: &BTreeMap<String, String>,
    interrupted: &AtomicBool,
) -> Result<Observation, String> {
    fs::create_dir_all(stdout.parent().ok_or("Missing log directory")?)
        .map_err(|e| e.to_string())?;
    let out = File::create(stdout).map_err(|e| e.to_string())?;
    let mut err = File::create(stderr).map_err(|e| e.to_string())?;
    let start = Instant::now();
    let mut result = Observation {
        code: None,
        error: false,
        aborted: false,
        duration: 0.,
        details: String::new(),
    };
    if interrupted.load(Ordering::SeqCst) {
        result.error = true;
        result.aborted = true;
        result.details = "Execution interrupted".into();
    } else {
        let execution = (|| -> Result<(), String> {
            if argv.is_empty() {
                return Err("Empty command".into());
            }
            #[cfg(not(unix))]
            {
                return Err(
                    "Native capture requires Unix process-group supervision on this platform"
                        .into(),
                );
            }
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let mut command = Command::new(&argv[0]);
                command
                    .args(&argv[1..])
                    .current_dir(cwd)
                    .stdin(Stdio::null())
                    .stdout(out)
                    .stderr(err.try_clone().map_err(|e| e.to_string())?)
                    .env_clear()
                    .envs(environment)
                    .process_group(0);
                let mut child = Group {
                    child: command.spawn().map_err(|e| e.to_string())?,
                    armed: true,
                };
                loop {
                    if interrupted.load(Ordering::SeqCst) {
                        result.aborted = true;
                        result.error = true;
                        result.details = "Execution interrupted".into();
                        break;
                    }
                    if let Some(status) = child.child.try_wait().map_err(|e| e.to_string())? {
                        result.code = status_code(status);
                        break;
                    }
                    if start.elapsed() >= timeout {
                        result.error = true;
                        result.details = "Execution timed out".into();
                        break;
                    }
                    std::thread::sleep(
                        Duration::from_millis(10).min(timeout.saturating_sub(start.elapsed())),
                    );
                }
                child.kill()?;
                child.child.wait().map_err(|e| e.to_string())?;
                child.armed = false;
                Ok(())
            }
        })();
        if let Err(error) = execution {
            result.error = true;
            result.details = error;
        }
    }
    if result.error {
        writeln!(err, "{}", result.details).map_err(|e| e.to_string())?;
    }
    result.duration = start.elapsed().as_secs_f64();
    Ok(result)
}
