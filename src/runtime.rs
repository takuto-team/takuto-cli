use anyhow::{bail, Result};
use console::style;
use std::process::Command;

/// Caller preference for which container runtime to use.
pub enum RuntimePreference {
    /// Auto-detect (current behaviour).
    Auto,
    /// Force Docker; error if not available.
    Docker,
    /// Force Podman; error if not available.
    Podman,
}

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
    /// Returns the base compose command with `-f <compose_file>` included.
    /// Returns None for Podman without compose.
    pub fn compose_command(&self, compose_file: &std::path::Path) -> Option<Vec<String>> {
        let file_arg = compose_file.to_string_lossy().to_string();
        match self {
            Runtime::Docker { compose } => match compose {
                ComposeVariant::Plugin => {
                    Some(vec!["docker".into(), "compose".into(), "-f".into(), file_arg])
                }
                ComposeVariant::Standalone => {
                    Some(vec!["docker-compose".into(), "-f".into(), file_arg])
                }
            },
            Runtime::Podman { has_compose } => {
                if *has_compose {
                    Some(vec!["podman".into(), "compose".into(), "-f".into(), file_arg])
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

    /// Returns the image reference to use.
    ///
    /// Always the published GHCR image — the `--local` flag controls
    /// *pull behaviour* (never pull), not the image name.
    pub fn image(&self) -> &'static str {
        "ghcr.io/morphet81/maestro:latest"
    }
}

/// Detect the container runtime (Docker or Podman) and compose availability.
///
/// Pass a `RuntimePreference` to force a specific runtime or use auto-detection.
pub fn detect(pref: RuntimePreference) -> Result<Runtime> {
    match pref {
        RuntimePreference::Auto => detect_auto(),
        RuntimePreference::Docker => detect_forced_docker(),
        RuntimePreference::Podman => detect_forced_podman(),
    }
}

fn detect_auto() -> Result<Runtime> {
    let docker_available = which::which("docker").is_ok();
    let podman_available = which::which("podman").is_ok();

    let docker_is_real = docker_available && is_real_docker("docker");
    let podman_is_real = podman_available && is_real_podman("podman");
    let docker_is_podman_alias = docker_available && !docker_is_real;

    if docker_is_real {
        let compose = detect_docker_compose()?;
        return Ok(Runtime::Docker { compose });
    }

    if podman_is_real || docker_is_podman_alias {
        let has_compose = check_podman_compose();
        return Ok(Runtime::Podman { has_compose });
    }

    bail!(
        "No container runtime found.\n\n\
         Install one of the following:\n\
         {} Docker Desktop: https://docs.docker.com/get-docker/\n\
         {} Podman:         https://podman.io/getting-started/installation",
        style("•").cyan(),
        style("•").cyan(),
    );
}

fn detect_forced_docker() -> Result<Runtime> {
    if which::which("docker").is_err() {
        bail!(
            "Docker not found.\n\
             Install Docker Desktop: https://docs.docker.com/get-docker/"
        );
    }
    if !is_real_docker("docker") {
        bail!(
            "The `docker` binary appears to be a Podman alias.\n\
             Install real Docker or use --podman instead."
        );
    }
    let compose = detect_docker_compose()?;
    Ok(Runtime::Docker { compose })
}

fn detect_forced_podman() -> Result<Runtime> {
    // Native podman binary
    if which::which("podman").is_ok() && is_real_podman("podman") {
        let has_compose = check_podman_compose();
        return Ok(Runtime::Podman { has_compose });
    }
    // docker binary that is actually a podman alias
    if which::which("docker").is_ok() && !is_real_docker("docker") {
        let has_compose = check_podman_compose();
        return Ok(Runtime::Podman { has_compose });
    }
    bail!(
        "Podman not found.\n\
         Install Podman: https://podman.io/getting-started/installation"
    );
}

/// Check if a binary named "docker" is real Docker (not podman aliased).
fn is_real_docker(bin: &str) -> bool {
    // 1. Check --version output for "podman"
    let output = Command::new(bin).arg("--version").output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
            let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
            let combined = format!("{stdout} {stderr}");
            if combined.contains("podman") {
                return false;
            }
        }
        Err(_) => return false,
    }

    // 2. Check if the binary resolves to a path containing "podman"
    //    (catches symlinks like /usr/local/bin/docker -> podman)
    if let Ok(path) = which::which(bin) {
        if let Ok(resolved) = path.canonicalize() {
            if resolved.to_string_lossy().to_lowercase().contains("podman") {
                return false;
            }
        }
    }

    // 3. Check `docker info` — podman's output mentions "podman" even when
    //    the binary is wrapped as "docker"
    if let Ok(out) = Command::new(bin).arg("info").output() {
        let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
        if stdout.contains("podman") {
            return false;
        }
    }

    true
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
