mod commands;
mod config;
mod dbwire;
mod runtime;
mod templates;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Term};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "takuto", about = "Manage Takuto orchestration containers")]
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
    /// Start Takuto services
    Start,
    /// Stop Takuto services
    Stop,
    /// Restart Takuto services
    Restart,
}

/// The `.takuto` subdirectory where config files live.
pub const TAKUTO_DIR: &str = ".takuto";

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

/// Returns the path to the `.takuto` config directory within CWD.
pub fn takuto_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(TAKUTO_DIR)
}

/// Find the Takuto compose file (`takuto.yml`).
pub fn find_compose_file(cwd: &Path) -> Option<PathBuf> {
    let takuto_yml = cwd.join("takuto.yml");
    if takuto_yml.exists() {
        return Some(takuto_yml);
    }
    None
}

fn preflight_check() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mdir = cwd.join(TAKUTO_DIR);

    if !mdir.join("config.toml").exists() {
        anyhow::bail!(".takuto/config.toml not found. Run `takuto setup` first.");
    }
    // Create takuto.yml if missing
    let compose_path = cwd.join("takuto.yml");
    if !compose_path.exists() {
        std::fs::write(&compose_path, templates::DOCKER_COMPOSE)?;
    }

    // Create takuto.env if missing (non-fatal)
    let env_path = mdir.join("takuto.env");
    if !env_path.exists() && mdir.exists() {
        std::fs::write(&env_path, templates::TAKUTO_ENV)?;
    }

    Ok(())
}
