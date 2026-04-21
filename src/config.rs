use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MaestroConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run_commands: Vec<RunCommand>,
    #[serde(default)]
    pub general: General,
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub commands: CommandsConfig,
    #[serde(default)]
    pub web: Web,
    #[serde(default)]
    pub agent: Agent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira: Option<Jira>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docker: Option<DockerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<Editor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal: Option<Terminal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<GitHubApp>,
}

impl Default for MaestroConfig {
    fn default() -> Self {
        Self {
            run_commands: Vec::new(),
            general: General::default(),
            git: Git::default(),
            commands: CommandsConfig::default(),
            web: Web::default(),
            agent: Agent::default(),
            jira: None,
            docker: None,
            editor: Some(Editor::default()),
            terminal: None,
            network: Some(Network::default()),
            github: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunCommand {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
    #[serde(default = "default_ticketing_system")]
    pub ticketing_system: String,
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
    #[serde(default = "default_ticket_workflow")]
    pub ticket_workflow_steps_file: String,
    #[serde(default = "default_review_workflow")]
    pub review_workflow_steps_file: String,
    #[serde(default = "default_merge_base_workflow")]
    pub merge_base_workflow_steps_file: String,
}

impl Default for General {
    fn default() -> Self {
        Self {
            ticketing_system: default_ticketing_system(),
            poll_interval_secs: 60,
            pr_merge_poll_interval_secs: 60,
            max_concurrent_workflows: 1,
            max_active_workflows: 0,
            log_level: default_log_level(),
            ticket_workflow_steps_file: default_ticket_workflow(),
            review_workflow_steps_file: default_review_workflow(),
            merge_base_workflow_steps_file: default_merge_base_workflow(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Git {
    #[serde(default = "default_main")]
    pub base_branch: String,
    #[serde(default = "default_origin")]
    pub remote: String,
    #[serde(default)]
    pub repo_url: String,
    #[serde(default = "default_workspace")]
    pub repo_path: String,
}

impl Default for Git {
    fn default() -> Self {
        Self {
            base_branch: default_main(),
            remote: default_origin(),
            repo_url: String::new(),
            repo_path: default_workspace(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandsConfig {
    #[serde(default = "default_install")]
    pub install: String,
    #[serde(default)]
    pub pre_workflow: Vec<String>,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            install: default_install(),
            pre_workflow: vec![
                "npx -y skills add morphet81/cheat-sheets -a claude-code -a cursor --yes --skill '*' --global".to_string(),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Web {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub dashboard_username: String,
    #[serde(default)]
    pub dashboard_password: String,
}

impl Default for Web {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: 8080,
            dashboard_username: String::new(),
            dashboard_password: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_timeout")]
    pub step_timeout_secs: u64,
    #[serde(default)]
    pub model: String,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            step_timeout_secs: 1800,
            model: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jira {
    #[serde(default)]
    pub project_keys: Vec<String>,
    #[serde(default = "default_jira_item_types")]
    pub item_types: Vec<String>,
    #[serde(default = "default_done_status")]
    pub done_status: String,
    #[serde(default)]
    pub site: String,
    #[serde(default)]
    pub email: String,
}

impl Default for Jira {
    fn default() -> Self {
        Self {
            project_keys: Vec::new(),
            item_types: vec!["Task".to_string(), "Bug".to_string()],
            done_status: "Done".to_string(),
            site: String::new(),
            email: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerConfig {
    #[serde(default)]
    pub build_commands: Vec<String>,
    #[serde(default)]
    pub compose_up_commands: Vec<String>,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            build_commands: Vec::new(),
            compose_up_commands: Vec::new(),
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    #[serde(default)]
    pub extra_egress_hosts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_all_https: Option<bool>,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            extra_egress_hosts: Vec::new(),
            allow_all_https: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubApp {
    pub app_id: u64,
    pub app_installation_id: u64,
    pub app_private_key_path: String,
}

// Default value helpers
fn default_ticketing_system() -> String { "none".to_string() }
fn default_60() -> u64 { 60 }
fn default_1_u32() -> u32 { 1 }
fn default_log_level() -> String { "info".to_string() }
fn default_ticket_workflow() -> String { "workflows/ticket.toml".to_string() }
fn default_review_workflow() -> String { "workflows/review.toml".to_string() }
fn default_merge_base_workflow() -> String { "workflows/merge_base.toml".to_string() }
fn default_main() -> String { "main".to_string() }
fn default_origin() -> String { "origin".to_string() }
fn default_workspace() -> String { "/workspace".to_string() }
fn default_install() -> String { "npm ci".to_string() }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8080 }
fn default_provider() -> String { "claude".to_string() }
fn default_timeout() -> u64 { 1800 }
fn default_jira_item_types() -> Vec<String> { vec!["Task".to_string(), "Bug".to_string()] }
fn default_done_status() -> String { "Done".to_string() }
fn default_dynamic_ports() -> u32 { 10 }
