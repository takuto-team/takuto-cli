use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, Select};
use std::fs;
use std::path::Path;

use crate::config::*;
use crate::templates;

pub fn run() -> Result<()> {
    println!(
        "\n{}",
        style("  Maestro Setup Wizard  ").bold().on_cyan().black()
    );
    println!();

    // Load existing config or start fresh
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join("config.toml");
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        println!(
            "  {} Loading existing config.toml\n",
            style("✓").green().bold()
        );
        toml::from_str::<MaestroConfig>(&content).unwrap_or_default()
    } else {
        MaestroConfig::default()
    };

    // ── Git ──────────────────────────────────────────────────────────────
    section_header("Git");

    config.git.repo_url = Input::new()
        .with_prompt("Repository URL")
        .default(if config.git.repo_url.is_empty() {
            "https://github.com/your-org/your-repo.git".to_string()
        } else {
            config.git.repo_url.clone()
        })
        .interact_text()?;

    config.git.base_branch = Input::new()
        .with_prompt("Base branch")
        .default(config.git.base_branch.clone())
        .interact_text()?;

    config.git.remote = Input::new()
        .with_prompt("Git remote")
        .default(config.git.remote.clone())
        .interact_text()?;

    // ── General ──────────────────────────────────────────────────────────
    section_header("General");

    let ticketing_options = &["none", "github", "jira"];
    let current_ticketing_idx = ticketing_options
        .iter()
        .position(|&s| s == config.general.ticketing_system)
        .unwrap_or(0);

    let ticketing_idx = Select::new()
        .with_prompt("Ticketing system")
        .items(ticketing_options)
        .default(current_ticketing_idx)
        .interact()?;
    config.general.ticketing_system = ticketing_options[ticketing_idx].to_string();

    config.general.poll_interval_secs = Input::new()
        .with_prompt("Poll interval (seconds)")
        .default(config.general.poll_interval_secs)
        .interact_text()?;

    config.general.max_concurrent_workflows = Input::new()
        .with_prompt("Max concurrent workflows")
        .default(config.general.max_concurrent_workflows)
        .interact_text()?;

    config.general.max_active_workflows = Input::new()
        .with_prompt("Max active workflows (0 = unlimited)")
        .default(config.general.max_active_workflows)
        .interact_text()?;

    let log_options = &["info", "debug", "warn", "error", "trace"];
    let current_log_idx = log_options
        .iter()
        .position(|&s| s == config.general.log_level)
        .unwrap_or(0);
    let log_idx = Select::new()
        .with_prompt("Log level")
        .items(log_options)
        .default(current_log_idx)
        .interact()?;
    config.general.log_level = log_options[log_idx].to_string();

    // ── Agent ────────────────────────────────────────────────────────────
    section_header("AI Agent");

    let provider_options = &["claude", "cursor"];
    let current_provider_idx = provider_options
        .iter()
        .position(|&s| s == config.agent.provider)
        .unwrap_or(0);
    let provider_idx = Select::new()
        .with_prompt("Agent provider")
        .items(provider_options)
        .default(current_provider_idx)
        .interact()?;
    config.agent.provider = provider_options[provider_idx].to_string();

    {
        let mut model_options: Vec<&str> = match config.agent.provider.as_str() {
            "claude" => vec![
                "(default)",
                "claude-opus-4-7",
                "claude-opus-4-6",
                "claude-sonnet-4-6",
                "claude-haiku-4-5-20251001",
                "claude-sonnet-4-5-20250929",
                "Custom...",
            ],
            "cursor" => vec![
                "(default)",
                "claude-opus-4-7",
                "claude-sonnet-4-6",
                "gpt-5.4",
                "gpt-5",
                "gpt-5-mini",
                "gemini-3.1-pro",
                "gemini-3-flash",
                "Custom...",
            ],
            _ => vec!["(default)", "Custom..."],
        };

        // If current model is set and not already in the list, insert it after (default)
        let current = config.agent.model.as_str();
        let current_in_list = current.is_empty()
            || model_options.iter().any(|&o| o == current);
        if !current_in_list {
            model_options.insert(1, Box::leak(current.to_string().into_boxed_str()));
        }

        let default_idx = if current.is_empty() {
            0
        } else {
            model_options.iter().position(|&o| o == current).unwrap_or(0)
        };

        let model_idx = Select::new()
            .with_prompt("Model")
            .items(&model_options)
            .default(default_idx)
            .interact()?;

        let selected = model_options[model_idx];
        config.agent.model = if selected == "(default)" {
            String::new()
        } else if selected == "Custom..." {
            Input::new()
                .with_prompt("Enter model name")
                .default(config.agent.model.clone())
                .allow_empty(true)
                .interact_text()?
        } else {
            selected.to_string()
        };
    }

    config.agent.step_timeout_secs = Input::new()
        .with_prompt("Step timeout (seconds)")
        .default(config.agent.step_timeout_secs)
        .interact_text()?;

    // ── Commands ─────────────────────────────────────────────────────────
    section_header("Commands");

    config.commands.install = Input::new()
        .with_prompt("Install command")
        .default(config.commands.install.clone())
        .interact_text()?;

    // ── Web Dashboard ────────────────────────────────────────────────────
    section_header("Web Dashboard");

    config.web.port = Input::new()
        .with_prompt("Dashboard port")
        .default(config.web.port)
        .interact_text()?;

    config.web.dashboard_username = Input::new()
        .with_prompt("Dashboard username (empty = no auth)")
        .default(config.web.dashboard_username.clone())
        .allow_empty(true)
        .interact_text()?;

    if !config.web.dashboard_username.is_empty() {
        config.web.dashboard_password = Input::new()
            .with_prompt("Dashboard password")
            .default(config.web.dashboard_password.clone())
            .interact_text()?;
    } else {
        config.web.dashboard_password = String::new();
    }

    // ── Jira ─────────────────────────────────────────────────────────────
    if config.general.ticketing_system == "jira" {
        section_header("Jira");

        let jira = config.jira.get_or_insert_with(Jira::default);

        jira.site = Input::new()
            .with_prompt("Jira site URL")
            .default(jira.site.clone())
            .interact_text()?;

        jira.email = Input::new()
            .with_prompt("Jira email")
            .default(jira.email.clone())
            .interact_text()?;

        let keys_str: String = Input::new()
            .with_prompt("Project keys (comma-separated)")
            .default(jira.project_keys.join(", "))
            .interact_text()?;
        jira.project_keys = keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        jira.done_status = Input::new()
            .with_prompt("Done status")
            .default(jira.done_status.clone())
            .interact_text()?;
    } else {
        config.jira = None;
    }

    // ── Editor ───────────────────────────────────────────────────────────
    if Confirm::new()
        .with_prompt("Configure editor settings?")
        .default(false)
        .interact()?
    {
        section_header("Editor");
        let editor = config.editor.get_or_insert_with(Editor::default);

        editor.dynamic_ports = Input::new()
            .with_prompt("Dynamic port mappings")
            .default(editor.dynamic_ports)
            .interact_text()?;

        let ports_str: String = Input::new()
            .with_prompt("Pre-mapped ports (comma-separated, empty to skip)")
            .default(
                editor
                    .ports
                    .as_ref()
                    .map(|p| {
                        p.iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default(),
            )
            .allow_empty(true)
            .interact_text()?;

        editor.ports = if ports_str.is_empty() {
            None
        } else {
            Some(
                ports_str
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u16>().ok())
                    .collect(),
            )
        };
    }

    // ── Network ──────────────────────────────────────────────────────────
    if Confirm::new()
        .with_prompt("Configure network settings?")
        .default(false)
        .interact()?
    {
        section_header("Network");
        let network = config.network.get_or_insert_with(Network::default);

        let hosts_str: String = Input::new()
            .with_prompt("Extra egress hosts (comma-separated, empty to skip)")
            .default(network.extra_egress_hosts.join(", "))
            .allow_empty(true)
            .interact_text()?;

        network.extra_egress_hosts = hosts_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let allow_all = Confirm::new()
            .with_prompt("Allow all HTTPS egress?")
            .default(network.allow_all_https.unwrap_or(false))
            .interact()?;
        network.allow_all_https = if allow_all { Some(true) } else { None };
    }

    // ── Run Commands ─────────────────────────────────────────────────────
    section_header("Run Commands");
    if !config.run_commands.is_empty() {
        println!("  Current run commands:");
        for (i, cmd) in config.run_commands.iter().enumerate() {
            println!("    {}. {} → {}", i + 1, style(&cmd.name).bold(), cmd.command);
        }
        println!();
    }

    loop {
        if !Confirm::new()
            .with_prompt("Add a run command?")
            .default(config.run_commands.is_empty())
            .interact()?
        {
            break;
        }

        let name: String = Input::new()
            .with_prompt("  Command name")
            .interact_text()?;
        let command: String = Input::new()
            .with_prompt("  Shell command")
            .interact_text()?;
        config.run_commands.push(RunCommand { name, command });
    }

    // ── Write files ──────────────────────────────────────────────────────
    println!();
    section_header("Writing files");

    // config.toml
    let toml_str = toml::to_string_pretty(&config)?;
    fs::write(&config_path, &toml_str)?;
    println!("  {} config.toml", style("wrote").green());

    // docker-compose.yml
    write_if_missing(&cwd, "docker-compose.yml", templates::DOCKER_COMPOSE)?;

    // maestro.env
    write_if_missing(&cwd, "maestro.env", templates::MAESTRO_ENV)?;

    // workflows/
    let workflows_dir = cwd.join("workflows");
    if !workflows_dir.exists() {
        fs::create_dir_all(&workflows_dir)?;
    }
    write_if_missing(&workflows_dir, "ticket.toml", templates::WORKFLOW_TICKET)?;
    write_if_missing(&workflows_dir, "review.toml", templates::WORKFLOW_REVIEW)?;
    write_if_missing(
        &workflows_dir,
        "merge_base.toml",
        templates::WORKFLOW_MERGE_BASE,
    )?;

    println!(
        "\n  {} Setup complete! Next steps:\n\
         \n    1. Run {} to authenticate with GitHub and your AI provider\
         \n    2. Run {} to start Maestro\
         \n    3. Open {} in your browser\n",
        style("✓").green().bold(),
        style("maestro auth").cyan().bold(),
        style("maestro start").cyan().bold(),
        style(format!("http://localhost:{}", config.web.port))
            .cyan()
            .underlined(),
    );

    Ok(())
}

fn section_header(name: &str) {
    println!("  {} {}", style("─").dim(), style(name).bold().yellow());
}

fn write_if_missing(dir: &Path, filename: &str, content: &str) -> Result<()> {
    let path = dir.join(filename);
    if path.exists() {
        println!(
            "  {} {} (already exists)",
            style("skip").dim(),
            filename
        );
    } else {
        fs::write(&path, content)?;
        println!("  {} {}", style("wrote").green(), filename);
    }
    Ok(())
}
