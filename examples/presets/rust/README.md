# Rust Preset

Ready-to-use Maestro configuration for Rust projects.

## What's included

- `cargo build` as the install command
- **Run commands**: Run Server (`cargo run --release`), Run Tests watch mode (`cargo watch -x test`)
- Claude Code skills from `morphet81/cheat-sheets` (installed before each workflow)
- Rust toolchain (cargo, rustc, rustfmt, clippy) is pre-installed in the Maestro image

## Setup

```bash
cp config.toml /path/to/your/maestro/config.toml
```

Edit the values marked with `←` (repo URL, branch, ticketing system).
