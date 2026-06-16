pub const WORKFLOW_IMPLEMENT_TICKET: &str =
    include_str!("../examples/react-vite/.takuto/workflows/implement_ticket.toml");
pub const WORKFLOW_MERGE_BASE: &str =
    include_str!("../examples/react-vite/.takuto/workflows/merge_base.toml");
pub const WORKFLOW_ADDRESS_PR_COMMENTS: &str =
    include_str!("../examples/react-vite/.takuto/workflows/address_pr_comments.toml");
pub const TAKUTO_ENV: &str = include_str!("../examples/react-vite/.takuto/takuto.env");

/// Docker Compose template for the CLI.
/// Config files are read from the `.takuto/` subdirectory.
pub const DOCKER_COMPOSE: &str = r#"# Takuto — Docker Compose with workflow isolation (DinD sidecar)
#
# Usage:
#   takuto start                # start services
#   takuto auth                 # first-time auth
#   takuto stop                 # stop services
#
# Or manually:
#   docker compose -f takuto.yml up -d
#   docker compose -f takuto.yml run --rm -it takuto setup
#
# Multi-user: on first boot the dashboard prompts you to create the initial
# admin account. There are no dashboard credentials in config.toml anymore.

services:
  # ── Takuto application ──────────────────────────────────────────────────────
  takuto:
    container_name: takuto
    image: ${TAKUTO_IMAGE:-ghcr.io/takuto-team/takuto-core:latest}
    # Share the DinD network namespace so the dashboard's reverse proxy can
    # reach editor/terminal containers (bound on docker-proxy ports inside
    # DinD) over localhost — without this, opening the editor/terminal fails
    # with "upstream unavailable". The dashboard port 8080 is therefore
    # published on the `dind` service below, not here. takuto runs no egress
    # rules of its own (NET_ADMIN dropped) — workers apply egress in their
    # own network namespace.
    network_mode: "service:dind"
    volumes:
      # Configuration (required) — mounted read-write so the dashboard's
      # Configuration screens can persist changes back to the file.
      - ./.takuto/config.toml:/etc/takuto/config.toml:rw
      # Custom workflow definitions (optional) — *.toml discovered at startup
      - ./.takuto/workflows:/etc/takuto/workflows:ro
      # Environment variables / secrets (optional)
      - ./.takuto/takuto.env:/etc/takuto/env:ro
      # Persistent state: snapshots, takuto.db (users/sessions), secret.key
      - takuto-data:/home/takuto/.takuto
      # Admin-provisioned tools ([provisioning].install_commands)
      - takuto-tools:/opt/takuto-tools/bin
      - claude-auth:/home/takuto/.claude
      - cursor-auth:/home/takuto/.cursor
      - agents-data:/home/takuto/.agents
      - gh-auth:/home/takuto/.config/gh
      - acli-auth:/home/takuto/.config/acli
      - fcli-auth:/home/takuto/.config/fcli
      # Project repositories cloned via the dashboard "Setup a New Project" flow
      - workspaces:/workspaces
      # Legacy single-workspace mount (kept for backward compatibility)
      - workspace:/workspace
      # Caches
      - npm-cache:/home/takuto/.npm
      - mise-data:/home/takuto/.local/share/mise
      - mise-cache:/home/takuto/.cache/mise
      - aws-config:/home/takuto/.aws
      - playwright-cache:/home/takuto/.cache/ms-playwright
    environment:
      - TAKUTO_CONFIG=/etc/takuto/config.toml
      - TAKUTO_HOME=/home/takuto
      - TAKUTO_DATA_DIR=/home/takuto/.takuto
      - CURSOR_CONFIG_DIR=/home/takuto/.cursor
      # External database (optional): overrides [database].connection.
      # Leave unset to use the local SQLite default at {data_dir}/takuto.db.
      # - TAKUTO_DATABASE_CONNECTION=postgres://takuto:pw@db.example:5432/takuto
      # DinD connection — over localhost because takuto shares DinD's netns.
      - DOCKER_HOST=tcp://127.0.0.1:2375
      # DinD-side mount prefix of the takuto-data volume, for the per-workflow
      # secrets-bundle path translation.
      - TAKUTO_DIND_DATA_PREFIX=/shared-auth/takuto-data
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
    container_name: takuto-dind
    image: docker:27-dind
    privileged: true
    ports:
      # Takuto's dashboard — published here because the takuto service shares
      # this network namespace (network_mode: service:dind). Editor/terminal
      # are reached through the dashboard's /s/<token> proxy on this port, so
      # no separate worker/editor port range needs publishing.
      - "${TAKUTO_PORT:-8080}:8080"
    environment:
      DOCKER_TLS_CERTDIR: ""
    volumes:
      - workspaces:/workspaces
      - workspace:/workspace
      - dind-storage:/var/lib/docker
      # Per-workflow secrets bundle: the takuto-data volume (mounted at
      # /home/takuto/.takuto in takuto) must also be visible to the DinD
      # daemon here at /shared-auth/takuto-data, so worker containers can
      # bind-mount the per-user secrets (Cursor key, PAT, etc.) at
      # /run/takuto-secrets. Without it the mount is empty and agents fail
      # with "secret files vanished (host TempDir dropped)".
      - takuto-data:/shared-auth/takuto-data
      # Auth + tools volumes shared with worker containers
      - takuto-tools:/shared-auth/takuto-tools
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
      - ./.takuto/config.toml:/etc/takuto/config.toml:ro
      - ./.takuto/workflows:/etc/takuto/workflows:ro
      - ./.takuto/takuto.env:/etc/takuto/env:ro
    healthcheck:
      test: ["CMD", "docker", "info"]
      interval: 5s
      timeout: 3s
      retries: 10
      start_period: 5s

volumes:
  takuto-data:
  takuto-tools:
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
