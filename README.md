# Maestro

**Automated workflow orchestration for AI coding agents.**

Maestro polls your ticketing system (Jira, GitHub Issues, or manual input) for work, then runs a configurable pipeline for each ticket: create a branch, install dependencies, run AI agent steps (Claude Code or Cursor Agent), review, test, and open a PR — all in isolated Docker containers with a real-time web dashboard.

## Quick Start

### 1. Pull the image

```bash
docker pull ghcr.io/morphet81/maestro-releases:latest
```

### 2. Set up your project

```bash
mkdir maestro && cd maestro

# Download example configs
docker run --rm ghcr.io/morphet81/maestro-releases:latest cat /etc/maestro/examples/config.toml.example > config.toml
docker run --rm ghcr.io/morphet81/maestro-releases:latest cat /etc/maestro/examples/maestro.env.example > maestro.env

# Create workflow directory
mkdir workflows
```

Or copy the examples from this repository:

```bash
cp examples/config.toml config.toml
cp examples/maestro.env maestro.env
cp -r examples/workflows/ workflows/
```

### 3. Edit your config

Open `config.toml` and configure at minimum:

```toml
[git]
repo_url = "https://github.com/your-org/your-repo.git"

[commands]
install = "npm install"    # or pip install, cargo build, etc.
```

### 4. Create docker-compose.yml

```yaml
services:
  maestro:
    image: ghcr.io/morphet81/maestro-releases:latest
    container_name: maestro
    ports:
      - "8080:8080"
    cap_add:
      - NET_ADMIN
    volumes:
      - ./config.toml:/etc/maestro/config.toml:ro
      - ./workflows:/etc/maestro/workflows:ro
      - ./maestro.env:/etc/maestro/env:ro
      - maestro-data:/home/maestro/.maestro
      - claude-auth:/home/maestro/.claude
      - gh-auth:/home/maestro/.config/gh
      - workspace:/workspace
      - npm-cache:/home/maestro/.npm
      - mise-data:/home/maestro/.local/share/mise
      - mise-cache:/home/maestro/.cache/mise
    environment:
      - MAESTRO_CONFIG=/etc/maestro/config.toml
      - MAESTRO_HOME=/home/maestro
      - MAESTRO_DATA_DIR=/home/maestro/.maestro
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

volumes:
  maestro-data:
  claude-auth:
  gh-auth:
  workspace:
  npm-cache:
  mise-data:
  mise-cache:
```

### 5. First-time setup

```bash
docker compose run --rm -it --network=host maestro setup
```

This walks you through authenticating:
1. **GitHub CLI** (`gh`) — required for creating PRs
2. **Claude Code** or **Cursor Agent** — your AI provider
3. **Repository clone** — clones your project into the container's workspace

### 6. Start Maestro

```bash
docker compose up -d
```

Open **http://localhost:8080** in your browser.

---

## Configuration Guide

### Ticketing System

Maestro supports three modes:

| Mode | Config | Description |
|------|--------|-------------|
| **None** (default) | `ticketing_system = "none"` | Start workflows manually from the dashboard |
| **GitHub Issues** | `ticketing_system = "github"` | Polls GitHub Issues from your repo |
| **Jira** | `ticketing_system = "jira"` | Polls Jira for To Do tickets (requires Atlassian CLI auth) |

### AI Provider

| Provider | Config | Setup |
|----------|--------|-------|
| **Claude Code** (default) | `provider = "claude"` | Authenticated during `make setup` or via `CLAUDE_CODE_OAUTH_TOKEN` |
| **Cursor Agent** | `provider = "cursor"` | Set `cursor_cli = "agent"` and authenticate via `CURSOR_API_KEY` |

### Workflow Steps

Maestro runs a configurable sequence of steps for each ticket. Steps can be:

- **Agent steps** — AI sessions with prompts (Claude Code or Cursor Agent)
- **Command steps** — shell commands (e.g., `npm test`, `cargo clippy`)

See [`examples/workflows/`](examples/workflows/) for complete examples.

#### Inline steps (in config.toml)

```toml
[[agent_steps]]
name = "Implement"
prompt = "Implement the feature described in: {description}"

[[agent_steps]]
name = "Test"
commands = ["npm test"]
```

#### External step files

```toml
[general]
ticket_workflow_steps_file = "workflows/ticket.toml"
review_workflow_steps_file = "workflows/review.toml"
merge_base_workflow_steps_file = "workflows/merge_base.toml"
```

### Run Commands

Define custom commands that appear as buttons on completed workflow cards:

```toml
[[run_commands]]
name = "Dev Server"
command = "cd app && npm run dev"

[[run_commands]]
name = "Storybook"
command = "npx storybook dev -p 6006"
```

Containers run with automatic port detection — when a dev server starts, a port-forward button appears on the dashboard card.

### Dashboard Authentication

```toml
[web]
dashboard_username = "admin"
dashboard_password = "your-secure-password"
```

Leave both empty to disable authentication.

### GitHub App (optional)

For bot-attributed commits and PRs instead of your personal account:

```toml
[github]
app_id = 123456
app_installation_id = 78901234
app_private_key_path = "/etc/maestro/github-app-key.pem"
```

### Environment Variables

Secrets and API tokens go in `maestro.env` (mounted at `/etc/maestro/env`):

```bash
# Claude Code (skip interactive login)
CLAUDE_CODE_OAUTH_TOKEN=sk-ant-...

# Cursor Agent
CURSOR_API_KEY=...

# Figma integration
FIGMA_ACCESS_TOKEN=...

# Custom proxy
ANTHROPIC_BASE_URL=https://custom-proxy.example.com/claude
```

---

## Docker-in-Docker (Workflow Isolation)

For isolated workflow execution (recommended for production), add a DinD sidecar. See [`examples/docker-compose.dind.yml`](examples/docker-compose.dind.yml).

```bash
docker compose -f docker-compose.yml -f docker-compose.dind.yml up -d
```

---

## Dashboard Features

- **Real-time workflow cards** with progress segments and live terminal output
- **Ticket description editor** with Markdown preview, Mermaid diagrams, and AI improvement
- **Browser-based VS Code editor** and **web terminal** per workflow
- **Port forwarding** — auto-detected dev server ports shown as clickable buttons
- **Run commands** — custom shell commands on completed workflows
- **PWA** — installable progressive web app

---

## Prompt Placeholders

Available in agent step prompts and command step commands:

| Placeholder | Description |
|-------------|-------------|
| `{description}` | Ticket/issue description text |
| `{ticket_key}` | Ticket identifier (e.g., `PROJ-123`, `GH-42`) |
| `{ticket_summary}` | Ticket title |
| `{ticket_context}` | Formatted summary with all ticket fields |
| `{ticket_type}` | Type label (Bug, Story, Task) |
| `{acceptance_criteria}` | Acceptance criteria field |
| `{base_branch}` | Target branch (e.g., `main`) |
| `{pr_url}` | PR URL (available in review/merge-base steps) |

---

## Examples

This repository includes ready-to-use example configurations:

| File | Description |
|------|-------------|
| [`examples/config.toml`](examples/config.toml) | Full annotated configuration |
| [`examples/maestro.env`](examples/maestro.env) | Environment variables template |
| [`examples/docker-compose.yml`](examples/docker-compose.yml) | Basic Docker Compose setup |
| [`examples/docker-compose.dind.yml`](examples/docker-compose.dind.yml) | DinD sidecar for workflow isolation |
| [`examples/workflows/ticket.toml`](examples/workflows/ticket.toml) | Main ticket pipeline (implement → review → test → PR) |
| [`examples/workflows/review.toml`](examples/workflows/review.toml) | PR review comment handler |
| [`examples/workflows/merge_base.toml`](examples/workflows/merge_base.toml) | Base branch merge workflow |

---

## License

Proprietary. All rights reserved.
