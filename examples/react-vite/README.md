# React + Vite Preset

Ready-to-use Maestro configuration for React projects using Vite.

## What's included

- `config.toml` pre-set for a React/Vite project (Claude provider, GitHub Issues)
- `maestro.yml` Docker Compose with the DinD workflow-isolation sidecar
- `workflows/` — `implement_ticket`, `merge_base`, `address_pr_comments`
- Editor-extension suggestions (Prettier, ESLint, Tailwind) and dynamic port
  forwarding for dev servers (Vite on 5173, Storybook on 6006, etc.)

## Setup

```bash
cp -r . /path/to/your/maestro-project && cd /path/to/your/maestro-project
```

Edit the values marked with `←` in `.maestro/config.toml` (branch, ticketing
system, Jira details). Then `maestro auth` and `maestro start`.

## Configured from the dashboard (not in config.toml)

- **Install / worktree-init commands** (e.g. `npm ci`) → Configuration → Worktree Settings
- **Run-command buttons** (Dev Server, Storybook, Preview Build) → Configuration → Worktree Settings
- **Dashboard login** → create the admin account on the first-boot setup page
- **Repository to work on** → dashboard "Setup a New Project" button
