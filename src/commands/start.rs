use anyhow::{bail, Context, Result};
use console::style;
use std::process::Command;

use crate::runtime::Runtime;

pub fn run(rt: &Runtime, local: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let compose_file = crate::find_compose_file(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No takuto.yml found. Run `takuto setup` first."))?;

    let compose = rt
        .compose_command(&compose_file)
        .ok_or_else(|| anyhow::anyhow!(
            "Podman Compose is not installed.\n\
             Install it with: pip install podman-compose\n\
             Or use Docker instead: https://docs.docker.com/get-docker/"
        ))?;

    let project_dir = compose_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| cwd.clone());

    // Phase A — before `up`: if a local DB container is configured, resolve it and
    // write the container-facing TAKUTO_DATABASE_CONNECTION override so the app
    // boots with a string it can actually reach. Never block start on this.
    let wired = match crate::dbwire::prepare(rt, &project_dir) {
        Ok(w) => w,
        Err(e) => {
            println!("  {} database auto-wire skipped: {e}", style("⚠").yellow().bold());
            None
        }
    };
    if let Some(w) = &wired {
        println!(
            "  {} Database: `{}` (host :{} → {}:{})",
            style("→").cyan().bold(),
            w.db_container,
            w.published_port,
            crate::dbwire::ALIAS,
            w.internal_port,
        );
    }

    if local {
        println!(
            "\n  {} Using local image ({})...\n",
            style("→").cyan().bold(),
            rt.image(),
        );
    } else {
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
    }

    println!(
        "  {} Starting Takuto services...\n",
        style("→").cyan().bold()
    );

    let mut up_cmd = Command::new(&compose[0]);
    up_cmd.args(&compose[1..]).args(["up", "-d"]);
    if local {
        up_cmd.args(["--pull=never"]);
        up_cmd.env("TAKUTO_IMAGE", rt.image());
    }
    let status = up_cmd
        .status()
        .context("Failed to start services")?;

    if !status.success() {
        bail!("Failed to start services (exit {})", status);
    }

    // Phase B — after `up`: the network now exists, so attach the DB container to
    // it under the `takuto_db` alias and restart takuto so it reconnects with the
    // alias resolvable.
    if let Some(w) = &wired {
        match crate::dbwire::attach(rt, w) {
            Ok(()) => {
                let _ = Command::new(&compose[0])
                    .args(&compose[1..])
                    .args(["restart", "takuto"])
                    .status();
                println!(
                    "  {} Linked `{}` to Takuto's network as `{}`.",
                    style("✓").green().bold(),
                    w.db_container,
                    crate::dbwire::ALIAS,
                );
            }
            Err(e) => {
                println!(
                    "  {} Could not attach `{}` to Takuto's network: {e}\n      \
                     The dashboard may fail to reach the database until it is attached.",
                    style("⚠").yellow().bold(),
                    w.db_container,
                );
            }
        }
    }

    println!(
        "\n  {} Takuto is running. Dashboard: {}\n",
        style("✓").green().bold(),
        style("http://localhost:8080").cyan().underlined(),
    );
    Ok(())
}
