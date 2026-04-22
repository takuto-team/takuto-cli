use anyhow::{bail, Context, Result};
use console::style;
use std::process::Command;

use crate::runtime::Runtime;

pub fn run(rt: &Runtime) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let compose_file = crate::find_compose_file(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No maestro.yml found. Run `maestro setup` first."))?;

    let compose = rt
        .compose_command(&compose_file)
        .ok_or_else(|| anyhow::anyhow!(
            "Podman Compose is not installed.\n\
             Install it with: pip install podman-compose\n\
             Or use Docker instead: https://docs.docker.com/get-docker/"
        ))?;

    println!(
        "\n  {} Pulling latest images...\n",
        style("→").cyan().bold()
    );

    let pull_status = Command::new(&compose[0])
        .args(&compose[1..])
        .args(["pull"])
        .status()
        .context("Failed to pull images")?;

    if !pull_status.success() {
        println!(
            "  {} Could not pull latest images, using cached versions.\n",
            style("⚠").yellow().bold()
        );
    }

    println!(
        "  {} Starting Maestro services...\n",
        style("→").cyan().bold()
    );

    let status = Command::new(&compose[0])
        .args(&compose[1..])
        .args(["up", "-d"])
        .status()
        .context("Failed to start services")?;

    if !status.success() {
        bail!("Failed to start services (exit {})", status);
    }

    println!(
        "\n  {} Maestro is running. Dashboard: {}\n",
        style("✓").green().bold(),
        style("http://localhost:8080").cyan().underlined(),
    );
    Ok(())
}
