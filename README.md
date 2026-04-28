# Maestro

**The companion CLI for [Maestro Core](https://github.com/morphet81/maestro-core) — set up and manage your AI coding pipeline in minutes.**

Maestro Core is an AI coding pipeline that works at your pace: poll Jira or GitHub Issues automatically, run the full pipeline overnight (branch → implement → review → test → PR), or stay in the driver's seat and trigger each phase manually from the dashboard. The `maestro` CLI takes care of the boring part — generating config files, orchestrating Docker Compose, and running auth flows — so you can focus on what matters.

---

## What you can achieve

- **Fully automated mode** — connect Jira or GitHub Issues and Maestro polls automatically: it picks up "To Do" tickets, runs the full AI pipeline (worktree → install → implement → lint/tests → PR), and moves on to the next one.
- **Manual mode, your pace** — add any ticket or task to the dashboard yourself, refine its description with AI assistance before the agent ever sees it, then trigger each workflow phase when you're ready.
- **Mix both** — auto-pick routine tasks while manually curating the tricky ones.
- **Run multiple tickets in parallel** — configure how many workflows run concurrently; each gets its own git worktree and isolated environment.
- **Monitor everything in real time** — a live web dashboard streams terminal output per workflow, shows progress, and lets you pause, resume, retry, or inspect any run.
- **Jump into any workflow** — open a browser-based VS Code editor and web terminal, pre-configured with your project tools, pointed at the exact worktree the agent is working on.
- **Define your own pipeline steps** — TOML workflow definitions let you chain phases: implement → address PR comments → merge base branch. Steps depend on each other; trigger them from the dashboard.
- **Work without a ticketing system** — paste any description via the dashboard and Maestro treats it as a workflow. No Jira account required.

---

## Why Maestro?

| | IDE assistant (Copilot, Cursor inline) | Maestro |
|---|---|---|
| **Where it runs** | Inside your editor, on your machine | Inside Docker, on any machine or server |
| **Supervision required** | Yes — you approve each step | Optional — fully autonomous or manual-trigger, your choice |
| **Ticketing integration** | None | Jira, GitHub Issues, or standalone |
| **Pipeline definition** | Single prompt | Multi-step TOML: implement, review, test, PR |
| **Concurrent work** | One task at a time | Multiple tickets in parallel |
| **Security boundary** | Full internet access from agent | Egress firewall — only approved hosts reachable |
| **Team deployment** | Per-developer only | Self-host on a server; shared dashboard |
| **Persistence** | Session ends when you close your editor | Survives container restarts; paused workflows resume |

---

## Quick Start

### 1. Install the CLI

**Homebrew (macOS / Linux — recommended):**
```bash
brew install morphet81/tools/maestro
```

<details>
<summary>Manual install</summary>

Download the latest binary for your platform from [Releases](https://github.com/morphet81/maestro/releases/latest).

**macOS (Apple Silicon):**
```bash
curl -L -o maestro https://github.com/morphet81/maestro/releases/latest/download/maestro-darwin-arm64
chmod +x maestro
sudo mv maestro /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -L -o maestro https://github.com/morphet81/maestro/releases/latest/download/maestro-darwin-amd64
chmod +x maestro
sudo mv maestro /usr/local/bin/
```

**Linux (amd64):**
```bash
curl -L -o maestro https://github.com/morphet81/maestro/releases/latest/download/maestro-linux-amd64
chmod +x maestro
sudo mv maestro /usr/local/bin/
```

**Linux (arm64):**
```bash
curl -L -o maestro https://github.com/morphet81/maestro/releases/latest/download/maestro-linux-arm64
chmod +x maestro
sudo mv maestro /usr/local/bin/
```

**Windows:**

Download [`maestro-windows-amd64.exe`](https://github.com/morphet81/maestro/releases/latest/download/maestro-windows-amd64.exe) and add it to your `PATH`.

</details>

### 2. Prerequisites

You need **Docker** or **Podman** installed. The CLI auto-detects which one you have (including aliases).

- [Docker Desktop](https://docs.docker.com/get-docker/)
- [Podman](https://podman.io/getting-started/installation)

Pull the Maestro Core container image:

```bash
docker pull ghcr.io/morphet81/maestro:latest
```

> **Private registry authentication:** If the image is private, authenticate first:
> ```bash
> gh auth refresh -h github.com -s read:packages
> gh auth token | docker login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin
> ```

### 3. Set up your project

Create a directory for your project and run the interactive setup wizard:

```bash
mkdir my-project && cd my-project
maestro setup
```

The wizard walks you through every configuration option (repo URL, ticketing system, AI provider, model, etc.) and generates all required files:

```
my-project/
  maestro.yml                      # Docker Compose orchestration
  .maestro/
    config.toml                    # project configuration
    maestro.env                    # secrets and API tokens
    workflows/                     # pipeline step definitions
      implement_ticket.toml
      merge_base.toml
      address_pr_comments.toml
```

### 4. Authenticate

```bash
maestro auth
```

This runs the first-time authentication flow inside the container:
1. **GitHub CLI** (`gh`) — required for creating PRs
2. **Claude Code** or **Cursor Agent** — your AI provider
3. **Repository clone** — clones your project into the container's workspace

### 5. Start Maestro

```bash
maestro start
```

Open **http://localhost:8080** in your browser.

If you configured Jira or GitHub Issues, Maestro starts polling automatically. Otherwise, click **+** to paste a description and kick off a workflow manually.

### Other commands

```bash
maestro stop       # stop Maestro services
maestro restart    # restart Maestro services
```

> **Multi-project isolation:** Docker Compose automatically prefixes all volumes with the directory name (e.g., `my-app_claude-auth`, `my-app_workspace`). To run Maestro on multiple projects simultaneously, use separate directories — each one gets fully isolated auth, workspace, and caches with no configuration needed.

---

## Security

> **⚠ Maestro runs AI agents autonomously and unattended.** Before going live, make sure the mitigations below are in place. A misconfigured setup can result in unreviewed code being pushed to protected branches or sensitive data being over-shared with the AI model.

**Security model:** Maestro does not maintain an engine-level allowlist for `gh` or `acli` calls. Security is delegated entirely to the token permissions you configure — scope your tokens to the minimum required.

### Branch protection (required)

Agents push branches and open PRs — they never commit directly to `main` or your release branches. Enforce this at the Git host level so it holds even if the agent misbehaves:

- **GitHub:** enable branch protection rules on `main` (and any other long-lived branches): require at least one human approving review before merge, enable status checks, and disable direct pushes.
- **GitLab:** use protected branches with "Maintainer" merge access and require approval rules.

Without branch protection, a prompt-injection attack embedded in a ticket description could instruct the agent to force-push or merge without review.

### Scoped GitHub token (required)

Use a **fine-grained personal access token** (PAT) scoped to the target repository instead of a classic token or your personal `gh` session. Grant only what Maestro needs:

| Permission    | Access       | Used for                                                                                         |
|---------------|--------------|--------------------------------------------------------------------------------------------------|
| Contents      | Read & write | `git push` (branch push before `gh pr create`)                                                   |
| Pull requests | Read & write | `gh pr create`, `gh pr edit --add-reviewer`, PR merge polling                                    |
| Metadata      | Read         | Required base permission for all fine-grained tokens                                             |
| Issues        | Read & write | Only if `ticketing_system = "github"` — Maestro polls issues and patches descriptions            |

To use a PAT, pick one of two approaches:

- **During `maestro auth`:** when prompted by the `gh` interactive login, paste the token.
- **Via `maestro.env`:** add `GH_TOKEN=<your-token>` — `gh` picks this up automatically, no interactive login needed.

### Scoped Jira tokens (required when using Jira)

Use a dedicated Jira service account or a scoped API token, not your personal admin credentials:

- Grant only **Browse Projects**, **Create Issues** (for comment/transition), and **Assign Issues** on the target project(s).
- Rotate the token if Maestro's container or its volumes are ever compromised.

### Prompt injection

Ticket descriptions (Jira or GitHub Issues) are embedded in AI prompts. Treat them like user-supplied content: a malicious ticket could attempt to override agent instructions. Branch protection and scoped tokens are your main defence — they limit what a hijacked agent session can actually do.

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
| **Claude Code** (default) | `provider = "claude"` | Authenticated during `maestro auth` or via `CLAUDE_CODE_OAUTH_TOKEN` |
| **Cursor Agent** | `provider = "cursor"` | Authenticate via `CURSOR_API_KEY` in `maestro.env` |

### Workflow Definitions

Maestro Core dynamically discovers `*.toml` files from the `workflows/` directory. The setup wizard generates three ready-to-use workflow definitions:

| File | Trigger | Description |
|------|---------|-------------|
| `implement_ticket.toml` | New ticket / manual | Full pipeline: implement → review → commit → lint/test → PR |
| `merge_base.toml` | After implement | Merges the base branch into the current feature branch |
| `address_pr_comments.toml` | After implement | Fixes PR review comments and re-runs lint/tests |

Each definition uses `[[steps]]` with a `prompt` (ticket context auto-injected) or `commands`. Chain definitions with `depends_on` — "merge base" and "address PR comments" only become available after "implement ticket" completes.

**Example step:**
```toml
name = "Implement Ticket"

[[steps]]
name = "Implement ticket"
prompt = """
Follow instructions provided below:
{description}
...
"""
repeat = 1

[[steps.skills]]
name = "address-ticket"
args = ["--no-jira", "--headless"]
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

# GitHub token (alternative to interactive gh login)
GH_TOKEN=github_pat_...

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

| Preset | Stack | Run Commands |
|--------|-------|-------------|
| [`examples/react-vite/`](examples/react-vite/) | React + Vite | Dev Server, Storybook, Preview Build |
| [`examples/rust/`](examples/rust/) | Rust | Run Server, cargo watch tests |
| [`examples/ruby-rails/`](examples/ruby-rails/) | Ruby on Rails | Rails Server, Console, Sidekiq |

Each preset is self-contained — copy the entire folder and edit `config.toml`.

---

## Manual Setup (without the CLI)

If you prefer not to use the `maestro` CLI, you can set up everything manually.

### 1. Pick a preset and copy it

```bash
cp -r examples/react-vite/ my-project && cd my-project
```

### 2. Edit .maestro/config.toml

Configure at minimum:

```toml
[git]
repo_url = "https://github.com/your-org/your-repo.git"

[commands]
install = "npm install"    # or pip install, cargo build, etc.
```

All configuration files live in the `.maestro/` subdirectory:
```
my-project/
  maestro.yml
  .maestro/
    config.toml                    # project configuration
    maestro.env                    # secrets and API tokens (optional)
    workflows/                     # pipeline step definitions
      implement_ticket.toml
      merge_base.toml
      address_pr_comments.toml
```

### 3. First-time setup

**Docker:**
```bash
docker compose -f maestro.yml run --rm -it maestro setup
```

**Podman:**
```bash
touch .maestro/maestro.env
P=$(basename "$(pwd)")

podman run --rm -it \
  --security-opt=label=disable \
  -v "$(pwd)/.maestro/config.toml":/etc/maestro/config.toml:ro \
  -v "$(pwd)/.maestro/workflows":/etc/maestro/workflows:ro \
  -v "$(pwd)/.maestro/maestro.env":/etc/maestro/env:ro \
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
  ghcr.io/morphet81/maestro:latest setup
```

The `P=...` variable prefixes volume names with your directory name so each project
is isolated — matching what Docker Compose does automatically.

### 4. Start Maestro

**Docker:**
```bash
docker compose -f maestro.yml up -d
```

**Podman:**
```bash
podman compose -f maestro.yml up -d
```

Open **http://localhost:8080** in your browser.

---

## Troubleshooting

### npm install fails during setup

Your npm registry is blocked by egress rules. Add the registry domain to `[network] extra_egress_hosts` in `config.toml`.

### Claude Code auth not found after restart

Auth is stored in Docker volumes. If volumes were deleted, re-run:
```bash
maestro auth
```

### Cursor agent login fails

Rebuild the Maestro Core image — you may be on an outdated layer:
```bash
docker compose -f maestro.yml build --no-cache
```

Or set `CURSOR_API_KEY` in `maestro.env` to skip interactive auth.

### `maestro start` stalls after "Egress rules applied"

Auth preflight is running. For Cursor, set `CURSOR_API_KEY` in `maestro.env` to skip interactive auth checks.

### Podman on Linux with SELinux

Add `:z` or `:Z` to volume mounts, or set `security_opt: [label=disable]` in `maestro.yml`.

---

## Source & License

This repository contains the Maestro CLI utility, licensed under [MIT](LICENSE).

The Maestro Core application is open source under [AGPL v3](https://github.com/morphet81/maestro-core/blob/main/LICENSE).
Source code is available at [github.com/morphet81/maestro-core](https://github.com/morphet81/maestro-core).
