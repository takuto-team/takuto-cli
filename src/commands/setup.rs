use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, Select};
use std::fs;
use std::path::Path;

use crate::config::*;
use crate::templates;
use crate::TAKUTO_DIR;

pub fn run() -> Result<()> {
    println!(
        "\n{}",
        style("  Takuto Setup Wizard  ").bold().on_cyan().black()
    );
    println!();

    // Ensure .takuto directory exists
    let cwd = std::env::current_dir()?;
    let mdir = cwd.join(TAKUTO_DIR);
    if !mdir.exists() {
        fs::create_dir_all(&mdir)?;
    }

    // Load existing config or start fresh
    let config_path = mdir.join("config.toml");
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        println!(
            "  {} Loading existing .takuto/config.toml\n",
            style("✓").green().bold()
        );
        toml::from_str::<TakutoConfig>(&content).unwrap_or_default()
    } else {
        TakutoConfig::default()
    };

    // Everything else — Git defaults, ticketing/Jira, AI provider & model,
    // editor and network settings, the admin account — is now configured from
    // takuto-core's web setup wizard. The CLI only bootstraps the two values
    // needed before the dashboard can boot: the port to serve it on and the
    // database it persists to.

    // ── Web Dashboard ────────────────────────────────────────────────────
    section_header("Web Dashboard");

    config.web.port = Input::new()
        .with_prompt("Dashboard port")
        .default(config.web.port)
        .interact_text()?;

    // ── Database ───────────────────────────────────────────────────────────
    // By default Takuto stores users/sessions in a local SQLite file. Point
    // it at an external Postgres/MySQL/MariaDB here (or via the
    // TAKUTO_DATABASE_CONNECTION env var in takuto.env).
    if Confirm::new()
        .with_prompt("Use an external database (Postgres / MySQL / MariaDB)?")
        .default(config.database.is_some())
        .interact()?
    {
        section_header("Database");

        // Where the DB lives changes how it must be reached from inside Takuto's
        // (DinD-shared) network namespace. A container on this machine can be
        // auto-wired from the host-facing URL the user already knows; a remote or
        // host-native database needs a URL that is reachable as-is.
        let kinds = [
            "A database container running on this machine (auto-wire it)",
            "A remote or host-native database (I'll provide a reachable URL)",
        ];
        let prev_local = config
            .database
            .as_ref()
            .and_then(|d| d.local_container)
            .unwrap_or(false);
        let kind = Select::new()
            .with_prompt("Where is your database?")
            .items(&kinds)
            .default(if prev_local { 0 } else { 1 })
            .interact()?;
        let local_container = kind == 0;

        let existing_conn = config
            .database
            .as_ref()
            .map(|d| d.connection.clone())
            .unwrap_or_default();
        let prompt = if local_container {
            "Connection URL as you'd use it from your host (e.g. postgres://user:pw@localhost:5433/takuto)"
        } else {
            "Connection URL (postgres://… | mysql://… | sqlite://…)"
        };
        let conn: String = Input::new()
            .with_prompt(prompt)
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
                local_container: if local_container { Some(true) } else { None },
            })
        };

        if local_container && config.database.is_some() {
            println!(
                "  {} On `takuto start`, Takuto will find the container behind that port, \
                 attach it to its network as `{}`, and rewrite the connection accordingly.",
                style("ℹ").cyan().bold(),
                crate::dbwire::ALIAS,
            );
        }
    } else {
        config.database = None;
    }

    // ── Network ──────────────────────────────────────────────────────────
    // Egress is restricted by default. Authorizing all HTTPS URLs lets the
    // sandbox reach any host over HTTPS; other egress rules are managed from
    // the dashboard.
    let allow_all_https = Confirm::new()
        .with_prompt("Authorize all HTTPS URLs in egress rules?")
        .default(
            config
                .network
                .as_ref()
                .and_then(|n| n.allow_all_https)
                .unwrap_or(true),
        )
        .interact()?;
    if allow_all_https {
        config
            .network
            .get_or_insert_with(Network::default)
            .allow_all_https = Some(true);
    } else if let Some(network) = config.network.as_mut() {
        network.allow_all_https = None;
    }

    // Whitelist additional egress domains beyond the defaults.
    if Confirm::new()
        .with_prompt("Whitelist additional egress domains?")
        .default(
            config
                .network
                .as_ref()
                .map(|n| !n.extra_egress_hosts.is_empty())
                .unwrap_or(false),
        )
        .interact()?
    {
        let existing = config
            .network
            .as_ref()
            .map(|n| n.extra_egress_hosts.join(", "))
            .unwrap_or_default();
        let hosts_str: String = Input::new()
            .with_prompt("Egress domains (comma-separated)")
            .default(existing)
            .allow_empty(true)
            .interact_text()?;
        config
            .network
            .get_or_insert_with(Network::default)
            .extra_egress_hosts = hosts_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    } else if let Some(network) = config.network.as_mut() {
        network.extra_egress_hosts.clear();
    }

    // ── Write files ──────────────────────────────────────────────────────
    println!();
    section_header("Writing files");

    // .takuto/config.toml
    let toml_str = toml::to_string_pretty(&config)?;
    fs::write(&config_path, &toml_str)?;
    println!("  {} .takuto/config.toml", style("wrote").green());

    // takuto.yml (at project root)
    write_if_missing(&cwd, "takuto.yml", templates::DOCKER_COMPOSE)?;

    // .takuto/takuto.env
    write_if_missing(&mdir, "takuto.env", templates::TAKUTO_ENV)?;

    // .takuto/workflows/
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
         \n    1. Run {} to launch Takuto\
         \n    2. Open {} and follow the setup wizard\n",
        style("✓").green().bold(),
        style("takuto start").cyan().bold(),
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
        println!("  {} {} (already exists)", style("skip").dim(), filename);
    } else {
        fs::write(&path, content)?;
        println!("  {} {}", style("wrote").green(), filename);
    }
    Ok(())
}
