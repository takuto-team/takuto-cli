use anyhow::{bail, Context, Result};
use console::style;
use std::process::Command;

use crate::runtime::Runtime;
use crate::TAKUTO_DIR;

/// Normalize a directory name into a Compose project name.
/// Docker/Podman Compose lowercase the directory name and keep only
/// `[a-z0-9-_]`.  We replicate that so raw `podman run` volumes
/// match the names Compose would create.
fn compose_project_name() -> Result<String> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let dir = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "takuto".to_string());

    let normalized: String = dir
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();

    Ok(if normalized.is_empty() {
        "takuto".to_string()
    } else {
        normalized
    })
}

pub fn run(rt: &Runtime, local: bool) -> Result<()> {
    println!(
        "\n  {} Running authentication flow...\n",
        style("→").cyan().bold()
    );

    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let compose_file = crate::find_compose_file(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No takuto.yml found. Run `takuto setup` first."))?;

    let status = match rt {
        Runtime::Docker { compose } => {
            let file_arg = compose_file.to_string_lossy().to_string();
            let mut cmd = match compose {
                crate::runtime::ComposeVariant::Plugin => {
                    let mut c = Command::new("docker");
                    let mut args = vec!["compose", "-f", &file_arg, "run", "--rm", "-it"];
                    if local {
                        args.push("--pull=never");
                    }
                    args.extend(["takuto", "setup"]);
                    c.args(&args);
                    c
                }
                crate::runtime::ComposeVariant::Standalone => {
                    let mut c = Command::new("docker-compose");
                    let mut args = vec!["-f", &file_arg, "run", "--rm", "-it"];
                    if local {
                        args.push("--pull=never");
                    }
                    args.extend(["takuto", "setup"]);
                    c.args(&args);
                    c
                }
            };
            if local {
                cmd.env("TAKUTO_IMAGE", rt.image());
            }
            cmd.status().context("Failed to run docker compose")?
        }
        Runtime::Podman { .. } => {
            // Podman compose doesn't support -it, use raw podman run
            let mdir = cwd.join(TAKUTO_DIR);
            let p = compose_project_name()?;

            let mut cmd = Command::new("podman");
            cmd.args(["run", "--rm", "-it"]);
            if local {
                cmd.args(["--pull=never"]);
            }
            cmd.args(["--security-opt=label=disable"]);

            // Config mounts from .takuto/ (config is read-write so the
            // setup flow can persist changes back to the file)
            cmd.args([
                "-v",
                &format!(
                    "{}:/etc/takuto/config.toml:rw",
                    mdir.join("config.toml").display()
                ),
            ]);
            cmd.args([
                "-v",
                &format!(
                    "{}:/etc/takuto/workflows:ro",
                    mdir.join("workflows").display()
                ),
            ]);
            cmd.args([
                "-v",
                &format!("{}:/etc/takuto/env:ro", mdir.join("takuto.env").display()),
            ]);

            // Named volumes (project-isolated, matching Compose naming)
            cmd.args(["-v", &format!("{p}_takuto-data:/home/takuto/.takuto")]);
            cmd.args(["-v", &format!("{p}_claude-auth:/home/takuto/.claude")]);
            cmd.args(["-v", &format!("{p}_cursor-auth:/home/takuto/.cursor")]);
            cmd.args(["-v", &format!("{p}_agents-data:/home/takuto/.agents")]);
            cmd.args(["-v", &format!("{p}_gh-auth:/home/takuto/.config/gh")]);
            cmd.args(["-v", &format!("{p}_acli-auth:/home/takuto/.config/acli")]);
            cmd.args(["-v", &format!("{p}_fcli-auth:/home/takuto/.config/fcli")]);
            cmd.args(["-v", &format!("{p}_workspaces:/workspaces")]);
            cmd.args(["-v", &format!("{p}_workspace:/workspace")]);
            cmd.args(["-v", &format!("{p}_npm-cache:/home/takuto/.npm")]);
            cmd.args([
                "-v",
                &format!("{p}_mise-data:/home/takuto/.local/share/mise"),
            ]);
            cmd.args(["-v", &format!("{p}_mise-cache:/home/takuto/.cache/mise")]);

            // Environment
            cmd.args(["-e", "TAKUTO_CONFIG=/etc/takuto/config.toml"]);
            cmd.args(["-e", "TAKUTO_HOME=/home/takuto"]);
            cmd.args(["-e", "TAKUTO_DATA_DIR=/home/takuto/.takuto"]);
            cmd.args(["-e", "CURSOR_CONFIG_DIR=/home/takuto/.cursor"]);
            cmd.args(["-e", "NODE_OPTIONS=--dns-result-order=ipv4first"]);

            // Image + command
            cmd.args([rt.image(), "setup"]);

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
