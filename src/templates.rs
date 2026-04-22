pub const WORKFLOW_TICKET: &str = include_str!("../examples/react-vite/.maestro/workflows/ticket.toml");
pub const WORKFLOW_REVIEW: &str = include_str!("../examples/react-vite/.maestro/workflows/review.toml");
pub const WORKFLOW_MERGE_BASE: &str =
    include_str!("../examples/react-vite/.maestro/workflows/merge_base.toml");
pub const MAESTRO_ENV: &str = include_str!("../examples/react-vite/.maestro/maestro.env");

/// Docker Compose template for the CLI.
/// Config files are read from `.maestro/` subdirectory.
pub const DOCKER_COMPOSE: &str = r#"# Maestro — Docker Compose with workflow isolation (DinD sidecar)
#
# Usage:
#   maestro start                # start services
#   maestro auth                 # first-time auth
#   maestro stop                 # stop services
#
# Or manually:
#   docker compose -f maestro.yml up -d
#   docker compose -f maestro.yml run --rm -it --network=host maestro setup

services:
  # ── Maestro application ──────────────────────────────────────────────────────
  maestro:
    container_name: maestro
    image: ghcr.io/morphet81/maestro:latest
    ports:
      - "8080:8080"
    cap_add:
      - NET_ADMIN
    volumes:
      # Configuration (required) — from .maestro/
      - ./.maestro/config.toml:/etc/maestro/config.toml:ro
      # Custom workflow steps (optional)
      - ./.maestro/workflows:/etc/maestro/workflows:ro
      # Environment variables / secrets (optional)
      - ./.maestro/maestro.env:/etc/maestro/env:ro
      # Persistent state
      - maestro-data:/home/maestro/.maestro
      - claude-auth:/home/maestro/.claude
      - cursor-auth:/home/maestro/.cursor
      - gh-auth:/home/maestro/.config/gh
      - acli-auth:/home/maestro/.config/acli
      # Workspace — your project is cloned here during setup
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
      - workspace:/workspace
      - dind-storage:/var/lib/docker
      # Auth volumes shared with worker containers
      - claude-auth:/shared-auth/claude
      - cursor-auth:/shared-auth/cursor
      - gh-auth:/shared-auth/gh
      - acli-auth:/shared-auth/acli
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
  claude-auth:
  cursor-auth:
  gh-auth:
  acli-auth:
  workspace:
  npm-cache:
  mise-data:
  mise-cache:
  aws-config:
  playwright-cache:
  dind-storage:
  vscode-data:
"#;
