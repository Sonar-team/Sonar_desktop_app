use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use crate::{setup::print_banner, utils::lookup_default_device};

const SMOKE_ARG: &str = "--sonar-smoke-test";
const SMOKE_LOG_ENV: &str = "SONAR_SMOKE_LOG_PATH";

pub const VALIDATION_LOG: &str = "SONAR_STARTUP_VALIDATION=OK";

pub fn is_requested() -> bool {
    env::args().any(|arg| arg == SMOKE_ARG)
}

pub fn run() -> i32 {
    match run_inner() {
        Ok(()) => 0,
        Err(err) => {
            let _ = report_failure(&err);
            1
        }
    }
}

fn run_inner() -> Result<(), String> {
    let mut logger = SmokeLogger::new().map_err(|err| err.to_string())?;

    logger.line("SONAR_STARTUP_VALIDATION=BEGIN")?;
    logger.line(&print_banner())?;

    let device = lookup_default_device()?;
    logger.line(&format!("Using device {}", device.name))?;
    logger.line(&format!("SONAR_SMOKE_DEVICE={}", device.name))?;
    logger.line(&format!(
        "SONAR_SMOKE_TARGET={} {}",
        env::consts::OS,
        env::consts::ARCH
    ))?;
    logger.line(VALIDATION_LOG)?;

    Ok(())
}

fn report_failure(err: &str) -> io::Result<()> {
    let message = format!("SONAR_STARTUP_VALIDATION=FAILED {err}");
    eprintln!("{message}");

    if let Some(path) = smoke_log_path() {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{message}")?;
    }

    Ok(())
}

fn smoke_log_path() -> Option<PathBuf> {
    env::var_os(SMOKE_LOG_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

struct SmokeLogger {
    file: Option<File>,
}

impl SmokeLogger {
    fn new() -> io::Result<Self> {
        let file = match smoke_log_path() {
            Some(path) => Some(File::create(path)?),
            None => None,
        };

        Ok(Self { file })
    }

    fn line(&mut self, message: &str) -> Result<(), String> {
        println!("{message}");

        if let Some(file) = self.file.as_mut() {
            writeln!(file, "{message}").map_err(|err| err.to_string())?;
        }

        Ok(())
    }
}
