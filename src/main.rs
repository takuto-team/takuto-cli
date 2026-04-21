mod commands;
mod config;
mod runtime;
mod templates;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Term};

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

fn preflight_check() -> Result<()> {
    let cwd = std::env::current_dir()?;

    if !cwd.join("config.toml").exists() {
        anyhow::bail!("config.toml not found. Run `maestro setup` first.");
    }
    if !cwd.join("docker-compose.yml").exists() {
        anyhow::bail!("docker-compose.yml not found. Run `maestro setup` first.");
    }

    // Create maestro.env if missing (non-fatal)
    let env_path = cwd.join("maestro.env");
    if !env_path.exists() {
        std::fs::write(&env_path, templates::MAESTRO_ENV)?;
    }

    Ok(())
}
