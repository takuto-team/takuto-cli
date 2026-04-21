use anyhow::{bail, Context, Result};
use console::style;
use std::process::Command;

use crate::runtime::Runtime;

pub fn run(rt: &Runtime) -> Result<()> {
    let compose = rt
        .compose_command()
        .ok_or_else(|| anyhow::anyhow!(
            "Podman Compose is not installed.\n\
             Install it with: pip install podman-compose\n\
             Or use Docker instead: https://docs.docker.com/get-docker/"
        ))?;

    println!(
        "\n  {} Stopping Maestro services...\n",
        style("→").cyan().bold()
    );

    let status = Command::new(&compose[0])
        .args(&compose[1..])
        .args(["down"])
        .status()
        .context("Failed to stop services")?;

    if !status.success() {
        bail!("Failed to stop services (exit {})", status);
    }

    println!(
        "\n  {} Maestro stopped.\n",
        style("✓").green().bold(),
    );
    Ok(())
}
