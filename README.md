# Maestro

**The companion CLI for [Maestro Core](https://github.com/morphet81/maestro-core) — set up and manage your AI coding pipeline in minutes.**

Maestro Core is an AI coding pipeline that works at your pace: poll Jira or GitHub Issues automatically, run the full pipeline overnight (branch → implement → review → test → PR), or stay in the driver's seat and trigger each phase manually from the dashboard. The `maestro` CLI takes care of the boring part — generating config files, orchestrating Docker Compose, and running auth flows — so you can focus on what matters.

---

## What you can achieve

- **Fully automated mode** — connect Jira or GitHub Issues and Maestro polls automatically: it picks up "To Do" tickets, runs the full AI pipeline (worktree → install → implement → lint/tests → PR), and moves on to the next one.
- **Manual mode, your pace** — add any ticket or task to the dashboard yourself, refine its description with AI assistance before the agent ever sees it, then trigger each workflow phase when you're ready.
- **Mix both** — auto-pick routine tasks while manually curating the tricky ones.
- **Run multiple tickets in parallel** — configure how many workflows run concurrently; each gets its own git worktree and isolated environment.
- **Work as a team** — Maestro is multi-user: each person signs in to a shared dashboard and sees only their own workflows. Self-host on a server and point everyone at the same instance.
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
| **Team model** | Per-developer only | Multi-user dashboard; shared, self-hosted instance |
| **Security boundary** | Full internet access from agent | Egress firewall — only approved hosts reachable |
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

The wizard asks about ticketing system, AI provider/model, branch, ports, and (optionally) an external database, then generates all required files:

```
my-project/
  maestro.yml                      # Docker Compose orchestration (+ DinD sidecar)
  .maestro/
    config.toml                    # bootstrap configuration (mounted read-write)
    maestro.env                    # secrets and API tokens
    workflows/                     # pipeline definitions (auto-discovered)
      implement_ticket.toml
      merge_base.toml
      address_pr_comments.toml
```

> The wizard no longer asks for a repository URL or dashboard credentials — you
> clone repositories and create your admin account from the dashboard itself
> (see steps 5–6). See [What moved to the dashboard](#what-moved-to-the-dashboard).

### 4. Authenticate

```bash
maestro auth
```

This runs the first-time authentication flow inside the container:
1. **GitHub CLI** (`gh`) — required for creating PRs
2. **Atlassian CLI** (`acli`) — only if you chose Jira
3. **Claude Code** / **Cursor Agent** / **Codex** / **OpenCode** — your AI provider

### 5. Start Maestro

```bash
maestro start
```

Open **http://localhost:8080** in your browser.

### 6. Create your admin account and add a repository

Maestro is **multi-user**. On first boot the dashboard shows a setup page —
the account you create there becomes the initial **admin**. Then:

- Click **"Setup a New Project"** to clone the repository you want Maestro to work on.
- If you configured Jira or GitHub Issues, polling starts automatically. Otherwise click **+** to paste a description and kick off a workflow manually.

### Other commands

```bash
maestro stop       # stop Maestro services
maestro restart    # restart Maestro services
```

`--docker` / `--podman` force a runtime; `--local` uses a locally built image instead of pulling.

> **Multi-project isolation:** Docker Compose automatically prefixes all volumes with the directory name (e.g., `my-app_claude-auth`, `my-app_workspaces`). To run Maestro for multiple projects simultaneously, use separate directories — each one gets fully isolated auth, workspaces, database, and caches with no configuration needed.

---

## What moved to the dashboard

Maestro Core now stores per-user and per-workspace settings in a database and
edits them from the dashboard's **Configuration** screens. These are **no
longer in `config.toml`** — the CLI doesn't generate them, and if an old config
still contains them they are ignored (with a startup warning):

| Used to be in `config.toml` | Now lives in |
|---|---|
| Dashboard login (`[web] dashboard_username` / `dashboard_password`) | Multi-user accounts in the database — create the admin on first boot |
| Install / worktree-init commands (`[commands]`) | **Configuration → Worktree Settings** (per user + workspace) |
| Run/stop dev-server buttons (`[[run_commands]]`) | **Configuration → Worktree Settings** |
| Repository URL (`[git] repo_url`) | Dashboard **"Setup a New Project"** button (multiple repos supported) |
| Per-provider model / endpoint | **Configuration → AI Settings** (`[agent.providers.<name>]` seeds the default) |
| Polling filters & cadence | **Configuration → Item Polling** |
| Workflow step file paths (`*_workflow_steps_file`) | Auto-discovery of every `*.toml` in `workflows/` |

`config.toml` is mounted **read-write** so the dashboard can persist changes
back to it — the file stays the source of truth for bootstrap settings.

---

## Multi-user model

Maestro is multi-user, single-tenant. Everyone shares one instance (and its
Jira/GitHub/AI credentials) but each person has their own dashboard view.

- **First boot:** when the database has zero users, the dashboard shows a setup page. The account you create becomes the initial **admin**.
- **Roles:** `admin` can manage users and shared state (config, polling, workspace switch, repo clone); `user` sees and acts only on workflows they created.
- **Sessions:** username + password (argon2-hashed in `maestro.db`). Idle TTL 24 h, absolute TTL 30 days. After 5 failed logins in 10 minutes the account locks (an admin unlocks it). One-time recovery codes are issued at account creation.
- **Poller ownership:** workflows created automatically by the Jira/GitHub poller are owned by `[general] poller_owner_username` (defaults to the first admin).

User management lives at **Configuration → Users** (admin-only). Full details are in Maestro Core's [README](https://github.com/morphet81/maestro-core#multi-user-model).

---

## External database

By default Maestro stores users, sessions, and snapshots in a local SQLite file
inside the `maestro-data` volume — zero configuration. For team or
multi-instance deployments you can point it at an external **PostgreSQL**,
**MySQL**, or **MariaDB**.

Two ways to configure it:

**In `.maestro/config.toml`:**
```toml
[database]
connection = "postgres://maestro:s3cret@db.example:5432/maestro"
# fail_fast = true          # abort startup if the DB is unreachable (default true)
# import_from_sqlite = true # one-shot copy of an existing maestro.db on first boot (default true)
```

**Or via `.maestro/maestro.env`** (takes precedence; keeps the secret out of `config.toml`):
```bash
export MAESTRO_DATABASE_CONNECTION="postgres://maestro:s3cret@db.example:5432/maestro"
```

Supported schemes: `sqlite://…`, `postgres://…` (and `postgresql://…`),
`mysql://…` (covers MariaDB). On first boot against an empty external database,
Maestro copies an existing local `maestro.db` over and then skips the import on
subsequent restarts.

> The database backend is **restart-only** — changing it requires `maestro restart`.
> Make sure the database host is reachable from the container: add it to
> `[network] extra_egress_hosts` (the egress firewall blocks unknown hosts), or
> run the database as a service on the same Docker network.

---

## Security

> **⚠ Maestro runs AI agents autonomously and unattended.** Before going live, make sure the mitigations below are in place. A misconfigured setup can result in unreviewed code being pushed to protected branches or sensitive data being over-shared with the AI model.

**Security model:** Maestro does not maintain an engine-level allowlist for `gh` or `acli` calls by default. Security is delegated to the token permissions you configure — scope your tokens to the minimum required.

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
- **Via `maestro.env`:** add `export GH_TOKEN=<your-token>` — `gh` picks this up automatically, no interactive login needed.

### Scoped Jira tokens (required when using Jira)

Use a dedicated Jira service account or a scoped API token, not your personal admin credentials:

- Grant only **Browse Projects**, **Create Issues** (for comment/transition), and **Assign Issues** on the target project(s).
- Rotate the token if Maestro's container or its volumes are ever compromised.

### Dashboard authentication

The dashboard is protected by **multi-user authentication** that can't be
disabled: every instance requires an admin account created on first boot, and
sessions are argon2-backed with idle/absolute TTLs, account lockout, and per-IP
rate limiting on login. When exposing the dashboard beyond localhost, terminate
TLS in front of it and set `[web] cors_origins` (and, if needed, `cookie_secure`).

### Prompt injection

Ticket descriptions (Jira or GitHub Issues) are embedded in AI prompts. Treat them like user-supplied content: a malicious ticket could attempt to override agent instructions. Branch protection and scoped tokens are your main defence — they limit what a hijacked agent session can actually do. Maestro also adds explicit untrusted-content framing and optional `[jira]` byte caps.

---

## Configuration Guide

`config.toml` holds **bootstrap** settings (needed before the dashboard and
database exist). Everything else is edited from the dashboard. The canonical
per-key reference lives in Maestro Core's
[`docs/configuration.md`](https://github.com/morphet81/maestro-core/blob/main/docs/configuration.md).

### Ticketing System

| Mode | Config | Description |
|------|--------|-------------|
| **None** (default) | `ticketing_system = "none"` | Start workflows manually from the dashboard |
| **GitHub Issues** | `ticketing_system = "github"` | Polls GitHub Issues; the repo is detected from the cloned project's git remote |
| **Jira** | `ticketing_system = "jira"` | Polls Jira for To Do tickets (requires `acli` auth and `[jira] site` / `email`) |

### AI Provider

`provider` in `[agent]` selects the tool; per-provider model and endpoint
details go in `[agent.providers.<name>]` (and are editable from
**Configuration → AI Settings**).

| Provider | Config | Setup |
|----------|--------|-------|
| **Claude Code** (default) | `provider = "claude"` | OAuth during `maestro auth`, or `ANTHROPIC_API_KEY` / `CLAUDE_CODE_OAUTH_TOKEN` in `maestro.env` |
| **Cursor Agent** | `provider = "cursor"` | Interactive login during `maestro auth`, or `CURSOR_API_KEY` in `maestro.env` |
| **Codex** | `provider = "codex"` | OpenAI-compatible; configure model/endpoint under `[agent.providers.codex]` |
| **OpenCode** | `provider = "opencode"` | Self-hosted / OpenAI-compatible (LM Studio, Ollama, vLLM…); set `model` and `base_url` under `[agent.providers.opencode]` |

```toml
[agent]
provider = "claude"
step_timeout_secs = 1800

[agent.providers.claude]
# model = "claude-sonnet-4-6"   # empty/unset = automatic selection
```

> Running a model server on your **host machine** for OpenCode? Docker may block
> the worker containers from reaching `host.docker.internal`. Maestro Core ships
> a small bridge sidecar for that case — see its
> [self-hosted model docs](https://github.com/morphet81/maestro-core/blob/main/docs/troubleshooting-self-hosted-models.md).

### Workflow Definitions

Maestro Core discovers **every** `*.toml` file in the `workflows/` directory at
startup — there are no per-file config keys anymore. The wizard generates three
ready-to-use definitions:

| File | Trigger | Description |
|------|---------|-------------|
| `implement_ticket.toml` | New ticket / manual | Full pipeline: implement → review → commit → lint/test → PR |
| `merge_base.toml` | After implement | Merges the base branch into the current feature branch |
| `address_pr_comments.toml` | After implement | Fixes PR review comments and re-runs lint/tests |

Each definition has a top-level `name`, optional `depends_on`, and `[[steps]]`
entries with a `prompt` (ticket context auto-injected), `commands`, or `skills`.
Chain definitions with `depends_on` — "merge base" and "address PR comments"
only become available after "implement ticket" completes.

```toml
name = "Implement Ticket"

[[steps]]
name = "Implement ticket"
prompt = """
Follow the instructions in the system prompt.
...
"""
when = "ticketing"        # "always" | "ticketing" | "no_ticketing"
repeat = 1

[[steps.skills]]
name = "create-pr"
args = ["--no-draft"]
```

### Worktree settings & run commands (dashboard)

Install / worktree-init commands and the run/stop dev-server buttons that appear
on workflow cards are **per-user and per-workspace** — configure them in
**Configuration → Worktree Settings**. They are not in `config.toml`.

### GitHub App (optional)

For bot-attributed commits and PRs instead of your personal account:

```toml
[github]
app_id = 123456
app_installation_id = 78901234
# Either an inline PEM key…
app_private_key = """
-----BEGIN RSA PRIVATE KEY-----
...
-----END RSA PRIVATE KEY-----
"""
# …or a path to a PEM file:
# app_private_key_path = "/etc/maestro/github-app-key.pem"
```

Required App permissions: contents (write), pull_requests (write), metadata (read).

### Provisioning (extra CLI tools)

Install tools that aren't baked into the image (e.g. `kubectl`, `terraform`, a
pinned `claude` version) into a shared volume on every start. The install is
SHA-gated, so unchanged lists are a no-op:

```toml
[provisioning]
install_commands = [
  '[ -f "$MAESTRO_TOOLS_BIN/kubectl" ] || (curl -fsSLo "$MAESTRO_TOOLS_BIN/kubectl" https://dl.k8s.io/release/v1.31.0/bin/linux/amd64/kubectl && chmod +x "$MAESTRO_TOOLS_BIN/kubectl")',
]
```

See Maestro Core's [`docs/extending-maestro.md`](https://github.com/morphet81/maestro-core/blob/main/docs/extending-maestro.md) for the full model.

### Environment Variables

Secrets and API tokens go in `maestro.env` (mounted at `/etc/maestro/env`). Only `export VAR=value` lines are honoured:

```bash
# Claude Code (skip interactive login)
export CLAUDE_CODE_OAUTH_TOKEN=sk-ant-...
export ANTHROPIC_API_KEY=sk-ant-...

# Cursor Agent
export CURSOR_API_KEY=...

# GitHub token (alternative to interactive gh login)
export GH_TOKEN=github_pat_...

# Custom Anthropic proxy / gateway
export ANTHROPIC_BASE_URL=https://custom-proxy.example.com/claude

# External database (alternative to [database].connection)
export MAESTRO_DATABASE_CONNECTION=postgres://maestro:pw@db.example:5432/maestro

# Figma integration
export FIGMA_API_TOKEN=...
```

---

## Dashboard Features

- **Multi-user sign-in** with per-user workflow isolation and an admin **Users** screen
- **"Setup a New Project"** — clone and switch between multiple repositories
- **Real-time workflow cards** with progress segments and live terminal output
- **Ticket description editor** with Markdown preview, Mermaid diagrams, and AI improvement
- **Browser-based VS Code editor** and **web terminal** per workflow
- **Port forwarding** — auto-detected dev server ports shown as clickable buttons
- **Run commands** — custom shell commands on completed workflows (Worktree Settings)
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

| Preset | Stack | Notes |
|--------|-------|-------------|
| [`examples/react-vite/`](examples/react-vite/) | React + Vite | Dynamic port forwarding for Vite/Storybook |
| [`examples/rust/`](examples/rust/) | Rust | Toolchain pre-installed in the image |
| [`examples/ruby-rails/`](examples/ruby-rails/) | Ruby on Rails | Ruby via mise |

Each preset is self-contained — copy the entire folder and edit `config.toml`.
Install commands and run-command buttons are configured from the dashboard
(**Configuration → Worktree Settings**), not the preset.

---

## Manual Setup (without the CLI)

If you prefer not to use the `maestro` CLI, you can set up everything manually.

### 1. Pick a preset and copy it

```bash
cp -r examples/react-vite/ my-project && cd my-project
```

### 2. Edit .maestro/config.toml

Configure at minimum the ticketing system and your base branch:

```toml
[general]
ticketing_system = "none"   # or "github" / "jira"

[git]
base_branch = "main"
```

Repositories are cloned from the dashboard, and install commands live in
**Configuration → Worktree Settings** — so there's nothing else required to boot.

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
  -v "$(pwd)/.maestro/config.toml":/etc/maestro/config.toml:rw \
  -v "$(pwd)/.maestro/workflows":/etc/maestro/workflows:ro \
  -v "$(pwd)/.maestro/maestro.env":/etc/maestro/env:ro \
  -v "${P}_maestro-data":/home/maestro/.maestro \
  -v "${P}_claude-auth":/home/maestro/.claude \
  -v "${P}_cursor-auth":/home/maestro/.cursor \
  -v "${P}_agents-data":/home/maestro/.agents \
  -v "${P}_gh-auth":/home/maestro/.config/gh \
  -v "${P}_acli-auth":/home/maestro/.config/acli \
  -v "${P}_fcli-auth":/home/maestro/.config/fcli \
  -v "${P}_workspaces":/workspaces \
  -v "${P}_workspace":/workspace \
  -v "${P}_npm-cache":/home/maestro/.npm \
  -v "${P}_mise-data":/home/maestro/.local/share/mise \
  -v "${P}_mise-cache":/home/maestro/.cache/mise \
  -e MAESTRO_CONFIG=/etc/maestro/config.toml \
  -e MAESTRO_HOME=/home/maestro \
  -e MAESTRO_DATA_DIR=/home/maestro/.maestro \
  -e CURSOR_CONFIG_DIR=/home/maestro/.cursor \
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

Open **http://localhost:8080**, create your admin account on the first-boot page, then clone a repository via **"Setup a New Project"**.

---

## Troubleshooting

### npm install fails during setup

Your npm registry is blocked by egress rules. Add the registry domain to `[network] extra_egress_hosts` in `config.toml`.

### Claude Code auth not found after restart

Auth is stored in Docker volumes. If volumes were deleted, re-run:
```bash
maestro auth
```

### Can't reach an external database

The egress firewall blocks unknown hosts. Add the database host to
`[network] extra_egress_hosts`, or run it as a service on the same Docker
network. Changing the backend requires `maestro restart`.

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
