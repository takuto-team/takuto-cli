# Ruby on Rails Preset

Ready-to-use Maestro configuration for Ruby on Rails projects.

## What's included

- `bundle install` as the install command
- **Run commands**: Rails Server, Rails Console, Sidekiq
- Claude Code skills from `morphet81/cheat-sheets` (installed before each workflow)
- Ruby 3.3 auto-installed via mise on first editor open (cached across restarts)
- Rails server binds to `0.0.0.0` for port forwarding to work

## Setup

```bash
cp config.toml /path/to/your/maestro/config.toml
```

Edit the values marked with `←` (repo URL, branch, ticketing system).

## Notes

- If your project uses a `.ruby-version` or `.tool-versions` file, mise will use that version instead of 3.3.
- For projects with a database, add your database host to `[network] extra_egress_hosts`.
