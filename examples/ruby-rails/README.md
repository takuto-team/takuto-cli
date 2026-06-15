# Ruby on Rails Preset

Ready-to-use Takuto configuration for Ruby on Rails projects.

## What's included

- `config.toml` pre-set for a Rails project (Claude provider, GitHub Issues)
- `takuto.yml` Docker Compose with the DinD workflow-isolation sidecar
- `workflows/` — `implement_ticket`, `merge_base`, `address_pr_comments`
- Ruby installed via mise on first editor open (cached across restarts)

## Setup

```bash
cp -r . /path/to/your/takuto-project && cd /path/to/your/takuto-project
```

Edit the values marked with `←` in `.takuto/config.toml` (branch, ticketing
system, Jira details). Then `takuto auth` and `takuto start`.

## Configured from the dashboard (not in config.toml)

- **Install / worktree-init commands** (e.g. `bundle install`) → Configuration → Worktree Settings
- **Run-command buttons** (Rails Server, Console, Sidekiq) → Configuration → Worktree Settings
- **Dashboard login** → create the admin account on the first-boot setup page
- **Repository to work on** → dashboard "Setup a New Project" button

## Notes

- If your project pins a Ruby version via `.ruby-version` or `.tool-versions`, mise uses that.
- For projects with a database, add your database host to `[network] extra_egress_hosts`,
  or run worktree-init steps from Configuration → Worktree Settings.
- Configure the Rails server to bind `0.0.0.0` so port forwarding reaches it.
