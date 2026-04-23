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
    /// Force Docker runtime (skip Podman detection)
    #[arg(long, global = true, conflicts_with = "podman")]
    docker: bool,

    /// Force Podman runtime (skip Docker detection)
    #[arg(long, global = true, conflicts_with = "docker")]
    podman: bool,

    /// Use locally built image instead of pulling from the registry
    #[arg(long, global = true)]
    local: bool,

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

    let pref = if cli.docker {
        runtime::RuntimePreference::Docker
    } else if cli.podman {
        runtime::RuntimePreference::Podman
    } else {
        runtime::RuntimePreference::Auto
    };

    match cli.command {
        Commands::Setup => commands::setup::run()?,
        Commands::Auth => {
            preflight_check()?;
            let rt = runtime::detect(pref)?;
            commands::auth::run(&rt, cli.local)?;
        }
        Commands::Start => {
            preflight_check()?;
            let rt = runtime::detect(pref)?;
            commands::start::run(&rt, cli.local)?;
        }
        Commands::Stop => {
            preflight_check()?;
            let rt = runtime::detect(pref)?;
            commands::stop::run(&rt)?;
        }
        Commands::Restart => {
            preflight_check()?;
            let rt = runtime::detect(pref)?;
            commands::stop::run(&rt)?;
            commands::start::run(&rt, cli.local)?;
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

/// Find the Maestro compose file (`maestro.yml`).
pub fn find_compose_file(cwd: &Path) -> Option<PathBuf> {
    let maestro_yml = cwd.join("maestro.yml");
    if maestro_yml.exists() {
        return Some(maestro_yml);
    }
    None
}

fn preflight_check() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mdir = cwd.join(MAESTRO_DIR);

    if !mdir.join("config.toml").exists() {
        anyhow::bail!(".maestro/config.toml not found. Run `maestro setup` first.");
    }
    // Create maestro.yml if missing
    let compose_path = cwd.join("maestro.yml");
    if !compose_path.exists() {
        std::fs::write(&compose_path, templates::DOCKER_COMPOSE)?;
    }

    // Create maestro.env if missing (non-fatal)
    let env_path = mdir.join("maestro.env");
    if !env_path.exists() && mdir.exists() {
        std::fs::write(&env_path, templates::MAESTRO_ENV)?;
    }

    Ok(())
}
