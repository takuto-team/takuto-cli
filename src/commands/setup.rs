use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, Select};
use std::fs;
use std::path::Path;

use crate::config::*;
use crate::templates;
use crate::MAESTRO_DIR;

pub fn run() -> Result<()> {
    println!(
        "\n{}",
        style("  Maestro Setup Wizard  ").bold().on_cyan().black()
    );
    println!();

    // Ensure .maestro directory exists
    let cwd = std::env::current_dir()?;
    let mdir = cwd.join(MAESTRO_DIR);
    if !mdir.exists() {
        fs::create_dir_all(&mdir)?;
    }

    // Load existing config or start fresh
    let config_path = mdir.join("config.toml");
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        println!(
            "  {} Loading existing .maestro/config.toml\n",
            style("✓").green().bold()
        );
        toml::from_str::<MaestroConfig>(&content).unwrap_or_default()
    } else {
        MaestroConfig::default()
    };

    // ── Git ──────────────────────────────────────────────────────────────
    // Note: repositories are cloned from the dashboard ("Setup a New Project")
    // — there is no repo URL in config.toml anymore.
    section_header("Git");

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

    config.general.auto_polling = Confirm::new()
        .with_prompt("Enable auto polling on startup?")
        .default(config.general.auto_polling)
        .interact()?;

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

    // ── AI Agent ───────────────────────────────────────────────────────────
    section_header("AI Agent");

    let provider_options = &["claude", "cursor", "codex", "opencode"];
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

    // Model, endpoint, and other per-provider details (`[agent.providers.<name>]`)
    // are managed from the dashboard's Configuration → AI Settings — the wizard
    // only sets the default provider so `maestro auth` knows which CLI to log in.

    config.agent.step_timeout_secs = Input::new()
        .with_prompt("Step timeout (seconds)")
        .default(config.agent.step_timeout_secs)
        .interact_text()?;

    // ── Web Dashboard ────────────────────────────────────────────────────
    // Authentication is multi-user: the initial admin account is created on
    // the dashboard's first-boot setup page, not configured here.
    section_header("Web Dashboard");

    config.web.port = Input::new()
        .with_prompt("Dashboard port")
        .default(config.web.port)
        .interact_text()?;

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

    // ── Database ───────────────────────────────────────────────────────────
    // By default Maestro stores users/sessions in a local SQLite file. Point
    // it at an external Postgres/MySQL/MariaDB here (or via the
    // MAESTRO_DATABASE_CONNECTION env var in maestro.env).
    if Confirm::new()
        .with_prompt("Use an external database (Postgres / MySQL / MariaDB)?")
        .default(config.database.is_some())
        .interact()?
    {
        section_header("Database");
        let existing_conn = config
            .database
            .as_ref()
            .map(|d| d.connection.clone())
            .unwrap_or_default();
        let conn: String = Input::new()
            .with_prompt("Connection URL (postgres://… | mysql://… | sqlite://…)")
            .default(existing_conn)
            .allow_empty(true)
            .interact_text()?;
        config.database = if conn.is_empty() {
            None
        } else {
            Some(Database {
                connection: conn,
                max_connections: None,
                acquire_timeout_secs: None,
                idle_timeout_secs: None,
                fail_fast: None,
                import_from_sqlite: None,
            })
        };
    } else {
        config.database = None;
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

    // ── Write files ──────────────────────────────────────────────────────
    println!();
    section_header("Writing files");

    // .maestro/config.toml
    let toml_str = toml::to_string_pretty(&config)?;
    fs::write(&config_path, &toml_str)?;
    println!("  {} .maestro/config.toml", style("wrote").green());

    // maestro.yml (at project root)
    write_if_missing(&cwd, "maestro.yml", templates::DOCKER_COMPOSE)?;

    // .maestro/maestro.env
    write_if_missing(&mdir, "maestro.env", templates::MAESTRO_ENV)?;

    // .maestro/workflows/
    let workflows_dir = mdir.join("workflows");
    if !workflows_dir.exists() {
        fs::create_dir_all(&workflows_dir)?;
    }
    write_if_missing(
        &workflows_dir,
        "implement_ticket.toml",
        templates::WORKFLOW_IMPLEMENT_TICKET,
    )?;
    write_if_missing(
        &workflows_dir,
        "merge_base.toml",
        templates::WORKFLOW_MERGE_BASE,
    )?;
    write_if_missing(
        &workflows_dir,
        "address_pr_comments.toml",
        templates::WORKFLOW_ADDRESS_PR_COMMENTS,
    )?;

    println!(
        "\n  {} Setup complete! Next steps:\n\
         \n    1. Run {} to authenticate with GitHub and your AI provider\
         \n    2. Run {} to start Maestro\
         \n    3. Open {} and create your admin account on the first-boot page\
         \n    4. Clone your repository from the dashboard's {} button\n",
        style("✓").green().bold(),
        style("maestro auth").cyan().bold(),
        style("maestro start").cyan().bold(),
        style(format!("http://localhost:{}", config.web.port))
            .cyan()
            .underlined(),
        style("Setup a New Project").bold(),
    );

    Ok(())
}

fn section_header(name: &str) {
    println!("  {} {}", style("─").dim(), style(name).bold().yellow());
}

fn write_if_missing(dir: &Path, filename: &str, content: &str) -> Result<()> {
    let path = dir.join(filename);
    if path.exists() {
        println!("  {} {} (already exists)", style("skip").dim(), filename);
    } else {
        fs::write(&path, content)?;
        println!("  {} {}", style("wrote").green(), filename);
    }
    Ok(())
}
