# Rust Preset

Ready-to-use Maestro configuration for Rust projects.

## What's included

- `config.toml` pre-set for a Rust project (Claude provider, GitHub Issues)
- `maestro.yml` Docker Compose with the DinD workflow-isolation sidecar
- `workflows/` — `implement_ticket`, `merge_base`, `address_pr_comments`
- Rust toolchain (cargo, rustc, rustfmt, clippy) is pre-installed in the Maestro image

## Setup

```bash
cp -r . /path/to/your/maestro-project && cd /path/to/your/maestro-project
```

Edit the values marked with `←` in `.maestro/config.toml` (branch, ticketing
system, Jira details). Then `maestro auth` and `maestro start`.

## Configured from the dashboard (not in config.toml)

- **Install / worktree-init commands** (e.g. `cargo build`) → Configuration → Worktree Settings
- **Run-command buttons** (Run Server, `cargo watch -x test`) → Configuration → Worktree Settings
- **Dashboard login** → create the admin account on the first-boot setup page
- **Repository to work on** → dashboard "Setup a New Project" button
