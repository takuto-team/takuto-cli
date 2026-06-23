use serde::{Deserialize, Serialize};

/// In-memory model of `.takuto/config.toml`.
///
/// This mirrors the **bootstrap** surface of Takuto Core's config — the keys
/// that must exist before the database and dashboard come up. Everything that
/// Takuto Core now manages from the dashboard (worktree init commands, run/stop
/// buttons, per-provider model details, polling policy) is intentionally *not*
/// modelled here: those live in the database and are edited from
/// **Configuration → …** screens. The keys we still write are deploy-time
/// defaults the UI can override.
#[derive(Debug, Serialize, Deserialize)]
pub struct TakutoConfig {
    #[serde(default)]
    pub general: General,
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub web: Web,
    #[serde(default)]
    pub agent: Agent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira: Option<Jira>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<GitHubApp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<Database>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<Editor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal: Option<Terminal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisioning: Option<Provisioning>,
}

impl Default for TakutoConfig {
    fn default() -> Self {
        Self {
            general: General::default(),
            git: Git::default(),
            web: Web::default(),
            agent: Agent::default(),
            jira: None,
            github: None,
            database: None,
            docker: None,
            editor: Some(Editor::default()),
            terminal: None,
            network: Some(Network::default()),
            provisioning: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
    #[serde(default = "default_ticketing_system")]
    pub ticketing_system: String,
    #[serde(default = "default_auto_polling")]
    pub auto_polling: bool,
    #[serde(default = "default_60")]
    pub poll_interval_secs: u64,
    #[serde(default = "default_60")]
    pub pr_merge_poll_interval_secs: u64,
    #[serde(default = "default_1_u32")]
    pub max_concurrent_workflows: u32,
    #[serde(default)]
    pub max_active_workflows: u32,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub dry_mode: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub poller_owner_username: String,
}

impl Default for General {
    fn default() -> Self {
        Self {
            ticketing_system: default_ticketing_system(),
            auto_polling: true,
            poll_interval_secs: 60,
            pr_merge_poll_interval_secs: 60,
            max_concurrent_workflows: 1,
            max_active_workflows: 0,
            log_level: default_log_level(),
            dry_mode: false,
            poller_owner_username: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Git {
    #[serde(default = "default_main")]
    pub base_branch: String,
    #[serde(default = "default_origin")]
    pub remote: String,
    #[serde(default = "default_workspace")]
    pub repo_path: String,
}

impl Default for Git {
    fn default() -> Self {
        Self {
            base_branch: default_main(),
            remote: default_origin(),
            repo_path: default_workspace(),
        }
    }
}

/// Dashboard web server. Authentication is multi-user (accounts live in the
/// database, the initial admin is created on first boot) — there are no
/// username/password keys here anymore.
#[derive(Debug, Serialize, Deserialize)]
pub struct Web {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cors_origins: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cookie_secure: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kick_other_sessions_on_login: Option<bool>,
}

impl Default for Web {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: 8080,
            cors_origins: Vec::new(),
            cookie_secure: None,
            kick_other_sessions_on_login: None,
        }
    }
}

/// AI provider defaults. Per-provider model/endpoint details live in
/// `[agent.providers.<name>]` and are normally edited from
/// Configuration → AI Settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_timeout")]
    pub step_timeout_secs: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_conversation_across_steps: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_repeated_output_lines: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<AgentProviders>,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            step_timeout_secs: 1800,
            share_conversation_across_steps: None,
            max_repeated_output_lines: None,
            providers: None,
        }
    }
}

/// `[agent.providers.<name>]` sub-tables. Each provider keeps its own model,
/// endpoint, and CLI settings.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AgentProviders {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude: Option<ProviderCfg>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<ProviderCfg>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex: Option<ProviderCfg>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opencode: Option<ProviderCfg>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProviderCfg {
    /// Model id. Empty / unset = the provider's automatic selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// OpenAI-compatible base URL (Claude / Codex / OpenCode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Executable name or path (Cursor Agent uses this).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    /// Context window of a self-hosted model (OpenCode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_limit: Option<u32>,
    /// Max output tokens of a self-hosted model (OpenCode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jira {
    #[serde(default)]
    pub site: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub project_keys: Vec<String>,
    #[serde(default = "default_jira_item_types")]
    pub item_types: Vec<String>,
    #[serde(default = "default_done_status")]
    pub done_status: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub jql_filter: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_items_in_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket_context_max_description_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_issue_description_max_bytes: Option<u64>,
}

impl Default for Jira {
    fn default() -> Self {
        Self {
            site: String::new(),
            email: String::new(),
            project_keys: Vec::new(),
            item_types: default_jira_item_types(),
            done_status: default_done_status(),
            jql_filter: String::new(),
            linked_items_in_prompt: None,
            ticket_context_max_description_bytes: None,
            linked_issue_description_max_bytes: None,
        }
    }
}

/// Optional GitHub App authentication (bot-attributed commits/PRs). All of
/// `app_id`, `app_installation_id`, and one private-key source must be set.
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubApp {
    pub app_id: u64,
    pub app_installation_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_private_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_private_key_path: Option<String>,
}

/// External database backend. Empty / omitted = local SQLite at
/// `{data_dir}/takuto.db` (the zero-config default).
#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    /// Connection URL. `sqlite://…`, `postgres://…`, or `mysql://…` (covers
    /// MariaDB). Empty keeps the SQLite default.
    pub connection: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquire_timeout_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_timeout_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_fast: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_from_sqlite: Option<bool>,
    /// Set by `takuto setup` when the user points at a database **container
    /// running on this machine**. The connection URL is then the host-facing one
    /// (e.g. `…@localhost:5433/…`); on `takuto start` the CLI resolves the
    /// published port to its container, attaches it to Takuto's network under the
    /// `takuto_db` alias, and writes a container-facing
    /// `TAKUTO_DATABASE_CONNECTION` into `takuto.env`. See `crate::dbwire`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_container: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DockerConfig {
    #[serde(default)]
    pub build_commands: Vec<String>,
    #[serde(default)]
    pub compose_up_commands: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Editor {
    #[serde(default = "default_dynamic_ports")]
    pub dynamic_ports: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<u16>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            dynamic_ports: 10,
            ports: None,
            theme: None,
            extensions: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Terminal {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_editor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup_commands: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_commands: Option<Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Network {
    #[serde(default)]
    pub extra_egress_hosts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_all_https: Option<bool>,
}

/// Admin-installed CLI tools, layered onto the shared `takuto-tools` volume at
/// startup (SHA-gated). See Takuto Core's `docs/extending-takuto.md`.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Provisioning {
    #[serde(default)]
    pub install_commands: Vec<String>,
}

// Default value helpers
fn default_ticketing_system() -> String { "none".to_string() }
fn default_auto_polling() -> bool { true }
fn default_60() -> u64 { 60 }
fn default_1_u32() -> u32 { 1 }
fn default_log_level() -> String { "info".to_string() }
fn default_main() -> String { "main".to_string() }
fn default_origin() -> String { "origin".to_string() }
fn default_workspace() -> String { "/workspace".to_string() }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8080 }
fn default_provider() -> String { "claude".to_string() }
fn default_timeout() -> u64 { 1800 }
fn default_jira_item_types() -> Vec<String> { vec!["Task".to_string(), "Bug".to_string()] }
fn default_done_status() -> String { "Done".to_string() }
fn default_dynamic_ports() -> u32 { 10 }

fn is_false(b: &bool) -> bool { !*b }

#[cfg(test)]
mod tests {
    use super::*;

    /// A default config must serialize to valid TOML and parse back.
    #[test]
    fn default_round_trips() {
        let cfg = TakutoConfig::default();
        let s = toml::to_string_pretty(&cfg).expect("serialize");
        let back: TakutoConfig = toml::from_str(&s).expect("parse");
        assert_eq!(back.general.ticketing_system, "none");
        assert_eq!(back.web.port, 8080);
        assert_eq!(back.agent.provider, "claude");
    }

    /// A fully-populated config exercises sub-table ordering (a scalar must
    /// never be emitted after a nested table within the same parent).
    #[test]
    fn populated_round_trips() {
        let mut cfg = TakutoConfig::default();
        cfg.general.dry_mode = true;
        cfg.general.poller_owner_username = "alice".into();
        cfg.web.cors_origins = vec!["https://takuto.example.com".into()];
        cfg.web.cookie_secure = Some(true);
        cfg.agent.provider = "opencode".into();
        cfg.agent.share_conversation_across_steps = Some(true);
        cfg.agent.providers = Some(AgentProviders {
            opencode: Some(ProviderCfg {
                model: Some("qwen2.5-coder-7b-instruct".into()),
                base_url: Some("http://lm-studio:1234/v1".into()),
                context_limit: Some(32768),
                ..Default::default()
            }),
            ..Default::default()
        });
        cfg.jira = Some(Jira {
            site: "x.atlassian.net".into(),
            jql_filter: "labels = takuto".into(),
            ..Jira::default()
        });
        cfg.github = Some(GitHubApp {
            app_id: 1,
            app_installation_id: 2,
            app_private_key_path: Some("/etc/takuto/key.pem".into()),
            app_private_key: None,
        });
        cfg.database = Some(Database {
            connection: "postgres://takuto:pw@db:5432/takuto".into(),
            fail_fast: Some(true),
            max_connections: None,
            acquire_timeout_secs: None,
            idle_timeout_secs: None,
            import_from_sqlite: None,
            local_container: None,
        });
        cfg.provisioning = Some(Provisioning {
            install_commands: vec!["echo hi".into()],
        });

        let s = toml::to_string_pretty(&cfg).expect("serialize");
        let back: TakutoConfig = toml::from_str(&s).expect("parse");
        assert_eq!(back.agent.provider, "opencode");
        assert_eq!(
            back.agent
                .providers
                .and_then(|p| p.opencode)
                .and_then(|o| o.base_url),
            Some("http://lm-studio:1234/v1".into())
        );
        assert_eq!(
            back.database.map(|d| d.connection),
            Some("postgres://takuto:pw@db:5432/takuto".into())
        );
    }

    /// Every shipped example preset must parse against the current schema.
    #[test]
    fn example_presets_parse() {
        for preset in ["react-vite", "rust", "ruby-rails"] {
            let path = format!(
                "{}/examples/{}/.takuto/config.toml",
                env!("CARGO_MANIFEST_DIR"),
                preset
            );
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {path}: {e}"));
            let _: TakutoConfig = toml::from_str(&content)
                .unwrap_or_else(|e| panic!("parse {path}: {e}"));
        }
    }
}
