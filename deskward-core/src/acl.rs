//! ACL groups — shared between deskward-id and deskward-console.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AclGroup {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct AclFile {
    #[serde(default)]
    groups: Vec<AclGroup>,
}

pub fn acl_path() -> PathBuf {
    std::env::var("DESKWARD_ACL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".deskward/acl.json")
        })
}

pub fn load_groups() -> Vec<AclGroup> {
    let path = acl_path();
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    match serde_json::from_str::<AclFile>(&text) {
        Ok(doc) => doc.groups,
        Err(_) => Vec::new(),
    }
}

pub fn save_groups(groups: &[AclGroup]) -> std::io::Result<()> {
    let path = acl_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let doc = AclFile {
        groups: groups.to_vec(),
    };
    let text = serde_json::to_string_pretty(&doc).map_err(std::io::Error::other)?;
    fs::write(path, text)
}

/// Allow punch when ACL empty; else `from` and `to` must share a group.
pub fn punch_allowed(from: &str, to: &str, groups: &[AclGroup]) -> bool {
    if groups.is_empty() {
        return true;
    }
    groups.iter().any(|g| {
        g.members.iter().any(|m| m == from) && g.members.iter().any(|m| m == to)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_acl_allows_all() {
        assert!(punch_allowed("a", "b", &[]));
    }

    #[test]
    fn shared_group_required() {
        let groups = vec![AclGroup {
            name: "home".into(),
            members: vec!["mac".into(), "phone".into()],
        }];
        assert!(punch_allowed("mac", "phone", &groups));
        assert!(!punch_allowed("mac", "stranger", &groups));
    }
}
