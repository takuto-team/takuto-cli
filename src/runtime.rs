use anyhow::{bail, Result};
use console::style;
use std::process::Command;

#[derive(Debug)]
pub enum ComposeVariant {
    Plugin,
    Standalone,
}

#[derive(Debug)]
pub enum Runtime {
    Docker { compose: ComposeVariant },
    Podman { has_compose: bool },
}

impl Runtime {
    /// Returns the base command for compose operations (e.g., "docker compose" or "podman compose").
    /// Returns None for Podman without compose.
    pub fn compose_command(&self) -> Option<Vec<String>> {
        match self {
            Runtime::Docker { compose } => match compose {
                ComposeVariant::Plugin => Some(vec!["docker".into(), "compose".into()]),
                ComposeVariant::Standalone => Some(vec!["docker-compose".into()]),
            },
            Runtime::Podman { has_compose } => {
                if *has_compose {
                    Some(vec!["podman".into(), "compose".into()])
                } else {
                    None
                }
            }
        }
    }

    /// Returns the raw container runtime binary name ("docker" or "podman").
    #[allow(dead_code)]
    pub fn runtime_binary(&self) -> &str {
        match self {
            Runtime::Docker { .. } => "docker",
            Runtime::Podman { .. } => "podman",
        }
    }

    #[allow(dead_code)]
    pub fn is_podman(&self) -> bool {
        matches!(self, Runtime::Podman { .. })
    }
}

/// Detect the container runtime (Docker or Podman) and compose availability.
pub fn detect() -> Result<Runtime> {
    // Check docker
    let docker_available = which::which("docker").is_ok();
    let podman_available = which::which("podman").is_ok();

    let docker_is_real = if docker_available {
        is_real_docker("docker")
    } else {
        false
    };

    let podman_is_real = if podman_available {
        is_real_podman("podman")
    } else {
        false
    };

    // docker binary exists but is actually podman (alias)
    let docker_is_podman_alias = docker_available && !docker_is_real;

    // Determine effective runtime
    if docker_is_real {
        // Real Docker — check compose
        let compose = detect_docker_compose()?;
        return Ok(Runtime::Docker { compose });
    }

    if podman_is_real || docker_is_podman_alias {
        // Podman (either native or aliased as docker)
        let has_compose = check_podman_compose();
        return Ok(Runtime::Podman { has_compose });
    }

    // Nothing found
    bail!(
        "No container runtime found.\n\n\
         Install one of the following:\n\
         {} Docker Desktop: https://docs.docker.com/get-docker/\n\
         {} Podman:         https://podman.io/getting-started/installation",
        style("•").cyan(),
        style("•").cyan(),
    );
}

/// Check if a binary named "docker" is real Docker (not podman aliased).
fn is_real_docker(bin: &str) -> bool {
    let output = Command::new(bin).arg("--version").output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
            let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
            let combined = format!("{stdout} {stderr}");
            // podman's version output typically contains "podman"
            !combined.contains("podman")
        }
        Err(_) => false,
    }
}

/// Check if a binary named "podman" is real Podman (not docker aliased).
fn is_real_podman(bin: &str) -> bool {
    let output = Command::new(bin).arg("--version").output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
            stdout.contains("podman")
        }
        Err(_) => false,
    }
}

/// Detect Docker Compose variant (plugin or standalone).
fn detect_docker_compose() -> Result<ComposeVariant> {
    // Try plugin first: docker compose version
    let plugin = Command::new("docker")
        .args(["compose", "version"])
        .output();

    if let Ok(out) = plugin {
        if out.status.success() {
            return Ok(ComposeVariant::Plugin);
        }
    }

    // Try standalone: docker-compose --version
    if which::which("docker-compose").is_ok() {
        let standalone = Command::new("docker-compose").arg("--version").output();
        if let Ok(out) = standalone {
            if out.status.success() {
                return Ok(ComposeVariant::Standalone);
            }
        }
    }

    bail!(
        "Docker is installed but Docker Compose is not available.\n\
         Install Docker Compose: https://docs.docker.com/compose/install/"
    );
}

/// Check if podman compose is available.
fn check_podman_compose() -> bool {
    // Try: podman compose version
    let result = Command::new("podman")
        .args(["compose", "version"])
        .output();

    matches!(result, Ok(out) if out.status.success())
}
