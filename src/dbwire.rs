//! Auto-wiring a user's **local database container** into Takuto's network.
//!
//! Why this exists: the `takuto` app shares the DinD sidecar's network namespace
//! (`network_mode: "service:dind"` in the generated compose). A connection URL is
//! therefore evaluated from *inside* that namespace — so the `localhost:5433` the
//! user knows from `psql`/DBeaver points back at Takuto, not at their database.
//!
//! Rather than make users hand-translate that, the wizard records the host-facing
//! URL they already know (with `[database].local_container = true`), and at
//! `takuto start` the CLI does the namespace translation itself:
//!   1. resolve the published port (`5433`) to its container + internal port (`5432`),
//!   2. attach that container to Takuto's network under a stable alias (`takuto_db`),
//!   3. write a container-facing `TAKUTO_DATABASE_CONNECTION` (`…@takuto_db:5432/…`)
//!      into `takuto.env`, which overrides `[database].connection` at boot.
//!
//! Steps 1 + 3 happen in [`prepare`] (before `compose up`, so the app boots with
//! the right string); step 2 in [`attach`] (after `up`, once the network exists).

use anyhow::{bail, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::runtime::Runtime;
use crate::TAKUTO_DIR;

/// Stable network alias the rewritten connection string points at. Attaching the
/// DB container under this alias is what makes the string survive container
/// recreation and gives the egress layer a fixed name to key off.
pub const ALIAS: &str = "takuto_db";

const ENV_KEY: &str = "TAKUTO_DATABASE_CONNECTION";

/// Outcome of [`prepare`]: what will be attached to the network in [`attach`].
pub struct Wired {
    pub db_container: String,
    pub published_port: u16,
    pub internal_port: u16,
}

/// Minimal parser for `scheme://[user[:pass]@]host[:port][/tail]` connection URLs
/// (Postgres / MySQL). We only need to swap host+port and keep everything else.
pub struct ConnUrl {
    scheme: String,
    userinfo: Option<String>,
    host: String,
    port: Option<u16>,
    tail: String,
}

impl ConnUrl {
    pub fn parse(s: &str) -> Option<ConnUrl> {
        let (scheme, rest) = s.split_once("://")?;
        if scheme.is_empty() {
            return None;
        }
        let (authority, tail) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, ""),
        };
        let (userinfo, hostport) = match authority.rfind('@') {
            Some(i) => (Some(authority[..i].to_string()), &authority[i + 1..]),
            None => (None, authority),
        };
        let (host, port) = split_host_port(hostport);
        if host.is_empty() {
            return None;
        }
        Some(ConnUrl {
            scheme: scheme.to_string(),
            userinfo,
            host,
            port,
            tail: tail.to_string(),
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Re-render the URL with a different host + port, preserving credentials,
    /// path, and query string.
    pub fn with_host_port(&self, host: &str, port: u16) -> String {
        let mut s = format!("{}://", self.scheme);
        if let Some(u) = &self.userinfo {
            s.push_str(u);
            s.push('@');
        }
        s.push_str(host);
        s.push(':');
        s.push_str(&port.to_string());
        s.push_str(&self.tail);
        s
    }
}

fn split_host_port(hp: &str) -> (String, Option<u16>) {
    // IPv6 literal: [::1]:5432
    if let Some(rest) = hp.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            let host = rest[..end].to_string();
            let port = rest[end + 1..]
                .strip_prefix(':')
                .and_then(|p| p.parse().ok());
            return (host, port);
        }
    }
    match hp.rsplit_once(':') {
        Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
            (h.to_string(), p.parse().ok())
        }
        _ => (hp.to_string(), None),
    }
}

/// True when the host names "this machine" — i.e. something the CLI can find as a
/// published container port rather than a remote address.
pub fn is_local_host(host: &str) -> bool {
    let h = host.trim_start_matches('[').trim_end_matches(']');
    matches!(h, "localhost" | "127.0.0.1" | "::1" | "host.docker.internal")
}

/// Resolved container behind a published host port.
pub struct Resolved {
    pub container: String,
    pub internal_port: u16,
}

/// Find the running container that publishes `port` on the host, and the internal
/// port it maps to. Parses `<runtime> ps` port mappings like
/// `0.0.0.0:5433->5432/tcp`.
pub fn resolve_published_port(rt: &Runtime, port: u16) -> Result<Option<Resolved>> {
    let out = Command::new(rt.runtime_binary())
        .args(["ps", "--format", "{{.Names}}\t{{.Ports}}"])
        .output()?;
    if !out.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(parse_ps_for_port(&text, port))
}

/// Pure core of [`resolve_published_port`], split out for testing.
fn parse_ps_for_port(ps_output: &str, port: u16) -> Option<Resolved> {
    let needle = format!(":{port}->");
    for line in ps_output.lines() {
        let (name, ports) = match line.split_once('\t') {
            Some(x) => x,
            None => continue,
        };
        if let Some(pos) = ports.find(&needle) {
            let after = &ports[pos + needle.len()..];
            let internal: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(internal_port) = internal.parse::<u16>() {
                return Some(Resolved {
                    container: name.trim().to_string(),
                    internal_port,
                });
            }
        }
    }
    None
}

/// The network the `takuto-dind` container is attached to (its first network).
/// Takuto shares this namespace, so attaching the DB here makes the alias
/// resolvable from the app.
pub fn dind_network(rt: &Runtime) -> Result<Option<String>> {
    let out = Command::new(rt.runtime_binary())
        .args([
            "inspect",
            "takuto-dind",
            "--format",
            "{{range $k, $v := .NetworkSettings.Networks}}{{$k}}\n{{end}}",
        ])
        .output()?;
    if !out.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|s| s.to_string()))
}

/// Idempotently attach `container` to `network` under `alias`. Returns `true` if
/// newly attached, `false` if it was already on the network.
pub fn connect_alias(rt: &Runtime, network: &str, container: &str, alias: &str) -> Result<bool> {
    let out = Command::new(rt.runtime_binary())
        .args(["network", "connect", "--alias", alias, network, container])
        .output()?;
    if out.status.success() {
        return Ok(true);
    }
    let err = String::from_utf8_lossy(&out.stderr).to_lowercase();
    // Already-attached is success for our purposes (re-running start is safe).
    if err.contains("already exists")
        || err.contains("already in network")
        || err.contains("endpoint with name")
    {
        return Ok(false);
    }
    bail!(
        "failed to attach `{container}` to network `{network}`: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
}

/// Whether a `takuto.env` line is an (uncommented) assignment of `key`.
fn is_assignment(line: &str, key: &str) -> bool {
    let t = line.trim_start();
    if t.starts_with('#') {
        return false;
    }
    matches!(t.split_once('='), Some((k, _)) if k.trim_end() == key)
}

/// Write (or replace) the `TAKUTO_DATABASE_CONNECTION` override in `takuto.env`,
/// leaving any commented examples and other keys untouched.
pub fn write_env_override(project_dir: &Path, url: &str) -> Result<()> {
    let env_path = project_dir.join(TAKUTO_DIR).join("takuto.env");
    let mut lines: Vec<String> = if env_path.exists() {
        fs::read_to_string(&env_path)?
            .lines()
            .map(|l| l.to_string())
            .collect()
    } else {
        Vec::new()
    };
    lines.retain(|l| !is_assignment(l, ENV_KEY));
    lines.push(format!("{ENV_KEY}={url}"));
    let mut body = lines.join("\n");
    body.push('\n');
    fs::write(&env_path, body)?;
    Ok(())
}

/// Phase A (before `compose up`): if the project is configured for a local DB
/// container, resolve it and write the container-facing override so the app boots
/// with the right string. Returns `None` (with a warning) when nothing should be
/// wired, so the caller can carry on starting normally.
pub fn prepare(rt: &Runtime, project_dir: &Path) -> Result<Option<Wired>> {
    let cfg_path = project_dir.join(TAKUTO_DIR).join("config.toml");
    if !cfg_path.exists() {
        return Ok(None);
    }
    let cfg: crate::config::TakutoConfig =
        toml::from_str(&fs::read_to_string(&cfg_path)?).unwrap_or_default();
    let Some(db) = cfg.database else {
        return Ok(None);
    };
    if db.local_container != Some(true) {
        return Ok(None);
    }

    let Some(url) = ConnUrl::parse(&db.connection) else {
        warn(&format!(
            "could not parse [database].connection (\"{}\"); leaving it unchanged.",
            db.connection
        ));
        return Ok(None);
    };
    if !is_local_host(url.host()) {
        warn(&format!(
            "[database] is marked as a local container but the host `{}` is not local; \
             leaving the connection string unchanged.",
            url.host()
        ));
        return Ok(None);
    }
    let Some(published) = url.port() else {
        warn("the database connection has no port; cannot locate the container.");
        return Ok(None);
    };

    match resolve_published_port(rt, published)? {
        Some(res) => {
            let alias_url = url.with_host_port(ALIAS, res.internal_port);
            write_env_override(project_dir, &alias_url)?;
            Ok(Some(Wired {
                db_container: res.container,
                published_port: published,
                internal_port: res.internal_port,
            }))
        }
        None => {
            warn(&format!(
                "no running container publishes port {published}; start your database \
                 container first, then re-run `takuto start`. Using the connection \
                 string as-is for now."
            ));
            Ok(None)
        }
    }
}

/// Phase B (after `compose up`): attach the resolved DB container to Takuto's
/// network so the `takuto_db` alias resolves from inside the shared namespace.
pub fn attach(rt: &Runtime, wired: &Wired) -> Result<()> {
    let Some(network) = dind_network(rt)? else {
        bail!("could not find the `takuto-dind` network — is the sidecar running?");
    };
    connect_alias(rt, &network, &wired.db_container, ALIAS)?;
    Ok(())
}

fn warn(msg: &str) {
    eprintln!(
        "  {} {}",
        console::style("⚠").yellow().bold(),
        msg
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_rewrites_postgres_url() {
        let u = ConnUrl::parse("postgres://takuto:s3cret@localhost:5433/takuto?sslmode=disable")
            .expect("parse");
        assert_eq!(u.host(), "localhost");
        assert_eq!(u.port(), Some(5433));
        assert_eq!(
            u.with_host_port("takuto_db", 5432),
            "postgres://takuto:s3cret@takuto_db:5432/takuto?sslmode=disable"
        );
    }

    #[test]
    fn rewrite_preserves_no_userinfo_and_no_query() {
        let u = ConnUrl::parse("mysql://127.0.0.1:3307/app").expect("parse");
        assert_eq!(u.with_host_port("takuto_db", 3306), "mysql://takuto_db:3306/app");
    }

    #[test]
    fn ipv6_host_is_local() {
        let u = ConnUrl::parse("postgres://u@[::1]:5432/db").expect("parse");
        assert_eq!(u.host(), "::1");
        assert!(is_local_host(u.host()));
    }

    #[test]
    fn local_host_detection() {
        assert!(is_local_host("localhost"));
        assert!(is_local_host("127.0.0.1"));
        assert!(is_local_host("host.docker.internal"));
        assert!(!is_local_host("db.example.com"));
        assert!(!is_local_host("db"));
    }

    #[test]
    fn finds_container_publishing_port() {
        let ps = "other\t0.0.0.0:8080->80/tcp\n\
                  my_pg\t0.0.0.0:5433->5432/tcp, :::5433->5432/tcp";
        let r = parse_ps_for_port(ps, 5433).expect("found");
        assert_eq!(r.container, "my_pg");
        assert_eq!(r.internal_port, 5432);
        assert!(parse_ps_for_port(ps, 9999).is_none());
    }

    #[test]
    fn env_override_replaces_existing_uncommented_only() {
        let dir = std::env::temp_dir().join(format!("dbwire-test-{}", std::process::id()));
        let mdir = dir.join(TAKUTO_DIR);
        fs::create_dir_all(&mdir).unwrap();
        fs::write(
            mdir.join("takuto.env"),
            "# TAKUTO_DATABASE_CONNECTION=example\nFOO=bar\nTAKUTO_DATABASE_CONNECTION=old\n",
        )
        .unwrap();

        write_env_override(&dir, "postgres://takuto_db:5432/db").unwrap();

        let out = fs::read_to_string(mdir.join("takuto.env")).unwrap();
        assert!(out.contains("# TAKUTO_DATABASE_CONNECTION=example")); // comment kept
        assert!(out.contains("FOO=bar")); // other keys kept
        assert!(out.contains("TAKUTO_DATABASE_CONNECTION=postgres://takuto_db:5432/db"));
        assert!(!out.contains("TAKUTO_DATABASE_CONNECTION=old")); // replaced
        assert_eq!(out.matches("\nTAKUTO_DATABASE_CONNECTION=").count(), 1);

        fs::remove_dir_all(&dir).ok();
    }
}
