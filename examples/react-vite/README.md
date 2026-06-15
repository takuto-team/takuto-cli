# React + Vite Preset

Ready-to-use Takuto configuration for React projects using Vite.

## What's included

- `config.toml` pre-set for a React/Vite project (Claude provider, GitHub Issues)
- `takuto.yml` Docker Compose with the DinD workflow-isolation sidecar
- `workflows/` — `implement_ticket`, `merge_base`, `address_pr_comments`
- Editor-extension suggestions (Prettier, ESLint, Tailwind) and dynamic port
  forwarding for dev servers (Vite on 5173, Storybook on 6006, etc.)

## Setup

```bash
cp -r . /path/to/your/takuto-project && cd /path/to/your/takuto-project
```

Edit the values marked with `←` in `.takuto/config.toml` (branch, ticketing
system, Jira details). Then `takuto auth` and `takuto start`.

## Configured from the dashboard (not in config.toml)

- **Install / worktree-init commands** (e.g. `npm ci`) → Configuration → Worktree Settings
- **Run-command buttons** (Dev Server, Storybook, Preview Build) → Configuration → Worktree Settings
- **Dashboard login** → create the admin account on the first-boot setup page
- **Repository to work on** → dashboard "Setup a New Project" button
