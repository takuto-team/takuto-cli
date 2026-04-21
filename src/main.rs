mod commands;
mod config;
mod runtime;
mod templates;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Term};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "maestro", about = "Manage Maestro orchestration containers")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive configuration wizard
    Setup,
    /// Run first-time authentication flow
    Auth,
    /// Start Maestro services
    Start,
    /// Stop Maestro services
    Stop,
    /// Restart Maestro services
    Restart,
}

/// The `.maestro` subdirectory where config files live.
pub const MAESTRO_DIR: &str = ".maestro";

fn main() {
    if let Err(e) = run() {
        let term = Term::stderr();
        let _ = term.write_line(&format!("{} {:#}", style("Error:").red().bold(), e));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => commands::setup::run()?,
        Commands::Auth => {
            preflight_check()?;
            let rt = runtime::detect()?;
            commands::auth::run(&rt)?;
        }
        Commands::Start => {
            preflight_check()?;
            let rt = runtime::detect()?;
            commands::start::run(&rt)?;
        }
        Commands::Stop => {
            preflight_check()?;
            let rt = runtime::detect()?;
            commands::stop::run(&rt)?;
        }
        Commands::Restart => {
            preflight_check()?;
            let rt = runtime::detect()?;
            commands::stop::run(&rt)?;
            commands::start::run(&rt)?;
        }
    }

    Ok(())
}

/// Returns the path to the `.maestro` config directory within CWD.
pub fn maestro_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(MAESTRO_DIR)
}

/// Find the compose file: `maestro.yml` takes priority over `docker-compose.yml`.
pub fn find_compose_file(cwd: &Path) -> Option<PathBuf> {
    let maestro_yml = cwd.join("maestro.yml");
    if maestro_yml.exists() {
        return Some(maestro_yml);
    }
    let docker_compose = cwd.join("docker-compose.yml");
    if docker_compose.exists() {
        return Some(docker_compose);
    }
    None
}

fn preflight_check() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mdir = cwd.join(MAESTRO_DIR);

    if !mdir.join("config.toml").exists() {
        anyhow::bail!(".maestro/config.toml not found. Run `maestro setup` first.");
    }
    if find_compose_file(&cwd).is_none() {
        anyhow::bail!("No maestro.yml or docker-compose.yml found. Run `maestro setup` first.");
    }

    // Create maestro.env if missing (non-fatal)
    let env_path = mdir.join("maestro.env");
    if !env_path.exists() && mdir.exists() {
        std::fs::write(&env_path, templates::MAESTRO_ENV)?;
    }

    Ok(())
}
