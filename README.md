# Maestro

**Automated workflow orchestration for AI coding agents.**

Maestro polls your ticketing system (Jira, GitHub Issues, or manual input) for work, then runs a configurable pipeline for each ticket: create a branch, install dependencies, run AI agent steps (Claude Code or Cursor Agent), review, test, and open a PR — all in isolated Docker containers with a real-time web dashboard.

## Quick Start

### 1. Pull the image

```bash
docker pull ghcr.io/morphet81/maestro-releases:latest
```

**Apple Silicon (M1/M2/M3):** The image is built for `linux/amd64`. Docker Desktop runs it via Rosetta emulation — pull with the explicit platform flag:

```bash
docker pull --platform linux/amd64 ghcr.io/morphet81/maestro-releases:latest
```

> **Private registry authentication:** If the image is private, authenticate first:
> ```bash
> gh auth refresh -h github.com -s read:packages
> gh auth token | docker login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin
> ```

### 2. Set up your project

Pick a preset that matches your stack and copy it:

```bash
# React + Vite
cp -r examples/react-vite/ my-maestro && cd my-maestro

# Rust
cp -r examples/rust/ my-maestro && cd my-maestro

# Ruby on Rails
cp -r examples/ruby-rails/ my-maestro && cd my-maestro
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
    platform: linux/amd64    # Required for Apple Silicon (M1/M2/M3)
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

> **Multi-project isolation:** Docker Compose automatically prefixes all volumes with
> the directory name (e.g., `my-app_claude-auth`, `my-app_workspace`). To run Maestro
> on multiple projects simultaneously, use separate directories — each one gets fully
> isolated auth, workspace, and caches with no configuration needed.

**Docker:**
```bash
docker compose run --rm -it --network=host maestro setup
```

**Podman:**
```bash
touch maestro.env    # create if missing (optional, for API tokens)
P=$(basename "$(pwd)")

podman run --rm -it \
  --network=host \
  --security-opt=label=disable \
  -v "$(pwd)/config.toml":/etc/maestro/config.toml:ro \
  -v "$(pwd)/workflows":/etc/maestro/workflows:ro \
  -v "$(pwd)/maestro.env":/etc/maestro/env:ro \
  -v "${P}_claude-auth":/home/maestro/.claude \
  -v "${P}_cursor-auth":/home/maestro/.cursor \
  -v "${P}_gh-auth":/home/maestro/.config/gh \
  -v "${P}_workspace":/workspace \
  -v "${P}_npm-cache":/home/maestro/.npm \
  -v "${P}_mise-data":/home/maestro/.local/share/mise \
  -v "${P}_mise-cache":/home/maestro/.cache/mise \
  -e MAESTRO_CONFIG=/etc/maestro/config.toml \
  -e MAESTRO_HOME=/home/maestro \
  -e NODE_OPTIONS=--dns-result-order=ipv4first \
  ghcr.io/morphet81/maestro-releases:latest setup
```

The `P=...` variable prefixes volume names with your directory name so each project
is isolated — matching what Docker Compose does automatically.

This walks you through authenticating:
1. **GitHub CLI** (`gh`) — required for creating PRs
2. **Claude Code** or **Cursor Agent** — your AI provider
3. **Repository clone** — clones your project into the container's workspace

### 6. Start Maestro

**Docker:**
```bash
docker compose up -d
```

**Podman:**
```bash
podman compose up -d
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

Each preset is self-contained — copy the entire folder and edit `config.toml`:

| Preset | Stack | Run Commands |
|--------|-------|-------------|
| [`examples/react-vite/`](examples/react-vite/) | React + Vite | Dev Server, Storybook, Preview Build |
| [`examples/rust/`](examples/rust/) | Rust | Run Server, cargo watch tests |
| [`examples/ruby-rails/`](examples/ruby-rails/) | Ruby on Rails | Rails Server, Console, Sidekiq |

Each preset includes: `config.toml`, `docker-compose.yml`, `maestro.env`, and `workflows/` (ticket, review, merge-base steps).

---

## License

Proprietary. All rights reserved.
