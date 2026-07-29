//! Multi-relay registry — `relays.toml` or `DESKWARD_RELAYS` env.

use std::env;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayEndpoint {
    pub host: String,
    pub port: u16,
    pub region: Option<String>,
}

pub fn configured_relays() -> Vec<RelayEndpoint> {
    if let Some(from_file) = load_relays_file() {
        if !from_file.is_empty() {
            return from_file;
        }
    }
    env::var("DESKWARD_RELAYS")
        .unwrap_or_else(|_| "127.0.0.1:29117".into())
        .split(',')
        .filter_map(parse_host_port)
        .collect()
}

pub fn pick_relay(preferred_region: Option<&str>) -> Option<RelayEndpoint> {
    select_relay(&configured_relays(), preferred_region)
}

fn select_relay(relays: &[RelayEndpoint], preferred_region: Option<&str>) -> Option<RelayEndpoint> {
    if let Some(region) = preferred_region {
        if let Some(found) = relays
            .iter()
            .find(|r| r.region.as_deref() == Some(region))
        {
            return Some(found.clone());
        }
    }
    if let Ok(region) = env::var("DESKWARD_REGION") {
        if let Some(found) = relays
            .iter()
            .find(|r| r.region.as_deref() == Some(region.as_str()))
        {
            return Some(found.clone());
        }
    }
    relays.first().cloned()
}

fn relays_config_path() -> Option<String> {
    env::var("DESKWARD_RELAYS_CONFIG").ok().or_else(|| {
        let cwd = env::current_dir().ok()?;
        let path = cwd.join("relays.toml");
        path.exists().then(|| path.to_string_lossy().into_owned())
    })
}

fn load_relays_file() -> Option<Vec<RelayEndpoint>> {
    let path = relays_config_path()?;
    let text = std::fs::read_to_string(&path).ok()?;
    parse_relays_toml(&text)
}

fn parse_relays_toml(text: &str) -> Option<Vec<RelayEndpoint>> {
    let doc: toml::Value = toml::from_str(text).ok()?;
    let relays = doc.get("relays")?.as_array()?;
    let mut out = Vec::new();
    for entry in relays {
        let table = entry.as_table()?;
        let host = table.get("host")?.as_str()?.to_string();
        let port = table.get("port")?.as_integer()? as u16;
        let region = table
            .get("region")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        out.push(RelayEndpoint {
            host,
            port,
            region,
        });
    }
    Some(out)
}

fn parse_host_port(s: &str) -> Option<RelayEndpoint> {
    let s = s.trim();
    let (host, port) = s.rsplit_once(':')?;
    port.parse().ok().map(|port| RelayEndpoint {
        host: host.to_string(),
        port,
        region: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_relays_toml() {
        let text = r#"
[[relays]]
host = "10.0.0.1"
port = 29117
region = "eu"
"#;
        let relays = parse_relays_toml(text).unwrap();
        assert_eq!(relays.len(), 1);
        assert_eq!(relays[0].host, "10.0.0.1");
        assert_eq!(relays[0].port, 29117);
        assert_eq!(relays[0].region.as_deref(), Some("eu"));
    }

    #[test]
    fn pick_relay_prefers_region() {
        let relays = vec![
            RelayEndpoint {
                host: "us.relay".into(),
                port: 29117,
                region: Some("us".into()),
            },
            RelayEndpoint {
                host: "eu.relay".into(),
                port: 29117,
                region: Some("eu".into()),
            },
        ];
        let picked = select_relay(&relays, Some("eu")).unwrap();
        assert_eq!(picked.host, "eu.relay");
    }

    #[test]
    fn load_relays_from_temp_file() {
        let dir = std::env::temp_dir().join(format!("deskward-relays-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("relays.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            "[[relays]]\nhost = \"relay.test\"\nport = 29117\n"
        )
        .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let relays = parse_relays_toml(&text).unwrap();
        assert_eq!(relays[0].host, "relay.test");
        let _ = std::fs::remove_dir_all(dir);
    }
}
