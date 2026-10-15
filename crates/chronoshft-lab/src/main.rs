use chronoshft_lab::cli::{self, USAGE_ERROR};
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Result<Vec<String>, _> = std::env::args_os()
        .skip(1)
        .map(|arg| arg.into_string())
        .collect();
    let args = match args {
        Ok(args) => args,
        Err(_) => {
            let _ = writeln!(io::stderr(), "Arguments must be valid Unicode.");
            return ExitCode::from(USAGE_ERROR);
        }
    };
    match cli::run(&args, &mut io::stdout().lock(), &mut io::stderr().lock()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let _ = writeln!(io::stderr(), "Could not write CLI output: {error}");
            ExitCode::from(1)
        }
    }
}
