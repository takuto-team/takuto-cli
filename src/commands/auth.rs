use anyhow::{bail, Context, Result};
use console::style;
use std::process::Command;

use crate::runtime::Runtime;
use crate::MAESTRO_DIR;

const IMAGE: &str = "ghcr.io/morphet81/maestro:latest";

/// Normalize a directory name into a Compose project name.
/// Docker/Podman Compose lowercase the directory name and keep only
/// `[a-z0-9-_]`.  We replicate that so raw `podman run` volumes
/// match the names Compose would create.
fn compose_project_name() -> Result<String> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let dir = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "maestro".to_string());

    let normalized: String = dir
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();

    Ok(if normalized.is_empty() {
        "maestro".to_string()
    } else {
        normalized
    })
}

pub fn run(rt: &Runtime) -> Result<()> {
    println!(
        "\n  {} Running authentication flow...\n",
        style("→").cyan().bold()
    );

    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let compose_file = crate::find_compose_file(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No maestro.yml found. Run `maestro setup` first."))?;

    let status = match rt {
        Runtime::Docker { compose } => {
            let file_arg = compose_file.to_string_lossy().to_string();
            let mut cmd = match compose {
                crate::runtime::ComposeVariant::Plugin => {
                    let mut c = Command::new("docker");
                    c.args([
                        "compose", "-f", &file_arg, "run", "--rm", "-it", "maestro", "setup",
                    ]);
                    c
                }
                crate::runtime::ComposeVariant::Standalone => {
                    let mut c = Command::new("docker-compose");
                    c.args(["-f", &file_arg, "run", "--rm", "-it", "maestro", "setup"]);
                    c
                }
            };
            cmd.status().context("Failed to run docker compose")?
        }
        Runtime::Podman { .. } => {
            // Podman compose doesn't support -it, use raw podman run
            let mdir = cwd.join(MAESTRO_DIR);
            let p = compose_project_name()?;

            let mut cmd = Command::new("podman");
            cmd.args(["run", "--rm", "-it"]);
            cmd.args(["--security-opt=label=disable"]);

            // Config mounts from .maestro/ (read-only)
            cmd.args([
                "-v",
                &format!(
                    "{}:/etc/maestro/config.toml:ro",
                    mdir.join("config.toml").display()
                ),
            ]);
            cmd.args([
                "-v",
                &format!(
                    "{}:/etc/maestro/workflows:ro",
                    mdir.join("workflows").display()
                ),
            ]);
            cmd.args([
                "-v",
                &format!("{}:/etc/maestro/env:ro", mdir.join("maestro.env").display()),
            ]);

            // Named volumes (project-isolated, matching Compose naming)
            cmd.args(["-v", &format!("{p}_claude-auth:/home/maestro/.claude")]);
            cmd.args(["-v", &format!("{p}_cursor-auth:/home/maestro/.cursor")]);
            cmd.args(["-v", &format!("{p}_gh-auth:/home/maestro/.config/gh")]);
            cmd.args(["-v", &format!("{p}_workspace:/workspace")]);
            cmd.args(["-v", &format!("{p}_npm-cache:/home/maestro/.npm")]);
            cmd.args([
                "-v",
                &format!("{p}_mise-data:/home/maestro/.local/share/mise"),
            ]);
            cmd.args(["-v", &format!("{p}_mise-cache:/home/maestro/.cache/mise")]);

            // Environment
            cmd.args(["-e", "MAESTRO_CONFIG=/etc/maestro/config.toml"]);
            cmd.args(["-e", "MAESTRO_HOME=/home/maestro"]);
            cmd.args(["-e", "NODE_OPTIONS=--dns-result-order=ipv4first"]);

            // Image + command
            cmd.args([IMAGE, "setup"]);

            cmd.status().context("Failed to run podman")?
        }
    };

    if !status.success() {
        bail!("Authentication flow exited with status {}", status);
    }

    println!(
        "\n  {} Authentication complete!\n",
        style("✓").green().bold()
    );
    Ok(())
}
