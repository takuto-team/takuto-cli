pub const WORKFLOW_IMPLEMENT_TICKET: &str =
    include_str!("../examples/react-vite/.maestro/workflows/implement_ticket.toml");
pub const WORKFLOW_MERGE_BASE: &str =
    include_str!("../examples/react-vite/.maestro/workflows/merge_base.toml");
pub const WORKFLOW_ADDRESS_PR_COMMENTS: &str =
    include_str!("../examples/react-vite/.maestro/workflows/address_pr_comments.toml");
pub const MAESTRO_ENV: &str = include_str!("../examples/react-vite/.maestro/maestro.env");

/// Docker Compose template for the CLI.
/// Config files are read from the `.maestro/` subdirectory.
pub const DOCKER_COMPOSE: &str = r#"# Maestro — Docker Compose with workflow isolation (DinD sidecar)
#
# Usage:
#   maestro start                # start services
#   maestro auth                 # first-time auth
#   maestro stop                 # stop services
#
# Or manually:
#   docker compose -f maestro.yml up -d
#   docker compose -f maestro.yml run --rm -it maestro setup
#
# Multi-user: on first boot the dashboard prompts you to create the initial
# admin account. There are no dashboard credentials in config.toml anymore.

services:
  # ── Maestro application ──────────────────────────────────────────────────────
  maestro:
    container_name: maestro
    image: ${MAESTRO_IMAGE:-ghcr.io/morphet81/maestro:latest}
    ports:
      - "8080:8080"
    cap_add:
      - NET_ADMIN
    volumes:
      # Configuration (required) — mounted read-write so the dashboard's
      # Configuration screens can persist changes back to the file.
      - ./.maestro/config.toml:/etc/maestro/config.toml:rw
      # Custom workflow definitions (optional) — *.toml discovered at startup
      - ./.maestro/workflows:/etc/maestro/workflows:ro
      # Environment variables / secrets (optional)
      - ./.maestro/maestro.env:/etc/maestro/env:ro
      # Persistent state: snapshots, maestro.db (users/sessions), secret.key
      - maestro-data:/home/maestro/.maestro
      # Admin-provisioned tools ([provisioning].install_commands)
      - maestro-tools:/opt/maestro-tools/bin
      - claude-auth:/home/maestro/.claude
      - cursor-auth:/home/maestro/.cursor
      - agents-data:/home/maestro/.agents
      - gh-auth:/home/maestro/.config/gh
      - acli-auth:/home/maestro/.config/acli
      - fcli-auth:/home/maestro/.config/fcli
      # Project repositories cloned via the dashboard "Setup a New Project" flow
      - workspaces:/workspaces
      # Legacy single-workspace mount (kept for backward compatibility)
      - workspace:/workspace
      # Caches
      - npm-cache:/home/maestro/.npm
      - mise-data:/home/maestro/.local/share/mise
      - mise-cache:/home/maestro/.cache/mise
      - aws-config:/home/maestro/.aws
      - playwright-cache:/home/maestro/.cache/ms-playwright
    environment:
      - MAESTRO_CONFIG=/etc/maestro/config.toml
      - MAESTRO_HOME=/home/maestro
      - MAESTRO_DATA_DIR=/home/maestro/.maestro
      - CURSOR_CONFIG_DIR=/home/maestro/.cursor
      # External database (optional): overrides [database].connection.
      # Leave unset to use the local SQLite default at {data_dir}/maestro.db.
      # - MAESTRO_DATABASE_CONNECTION=postgres://maestro:pw@db.example:5432/maestro
      # DinD connection
      - DOCKER_HOST=tcp://dind:2375
      - MAESTRO_DIND_PORT_OFFSET=100
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
    depends_on:
      dind:
        condition: service_healthy

  # ── Docker-in-Docker sidecar (workflow isolation) ────────────────────────────
  dind:
    container_name: maestro-dind
    image: docker:27-dind
    privileged: true
    ports:
      # Worker/editor ports — offset by 100 to avoid conflicts
      - "9200-9300:9100-9200"
    environment:
      DOCKER_TLS_CERTDIR: ""
    volumes:
      - workspaces:/workspaces
      - workspace:/workspace
      - dind-storage:/var/lib/docker
      # Auth + tools volumes shared with worker containers
      - maestro-tools:/shared-auth/maestro-tools
      - claude-auth:/shared-auth/claude
      - cursor-auth:/shared-auth/cursor
      - agents-data:/shared-auth/agents
      - gh-auth:/shared-auth/gh
      - acli-auth:/shared-auth/acli
      - fcli-auth:/shared-auth/fcli
      - npm-cache:/shared-auth/npm
      - mise-data:/shared-auth/mise-data
      - mise-cache:/shared-auth/mise-cache
      - aws-config:/shared-auth/aws
      - playwright-cache:/shared-auth/playwright-cache
      - vscode-data:/shared-auth/vscode
      # Config for worker egress rules
      - ./.maestro/config.toml:/etc/maestro/config.toml:ro
      - ./.maestro/workflows:/etc/maestro/workflows:ro
      - ./.maestro/maestro.env:/etc/maestro/env:ro
    healthcheck:
      test: ["CMD", "docker", "info"]
      interval: 5s
      timeout: 3s
      retries: 10
      start_period: 5s

volumes:
  maestro-data:
  maestro-tools:
  claude-auth:
  cursor-auth:
  agents-data:
  gh-auth:
  acli-auth:
  fcli-auth:
  workspaces:
  workspace:
  npm-cache:
  mise-data:
  mise-cache:
  aws-config:
  playwright-cache:
  dind-storage:
  vscode-data:
"#;
