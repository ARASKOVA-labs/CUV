use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuvManifest {
    pub project: ProjectConfig,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
    #[serde(default)]
    pub target: BTreeMap<String, TargetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    #[serde(default = "default_standard")]
    pub standard: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub entry: Option<String>,
}

fn default_standard() -> String {
    "c++20".to_string()
}

fn default_kind() -> String {
    "executable".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Version(String),
    Detailed {
        version: Option<String>,
        git: Option<String>,
        tag: Option<String>,
        branch: Option<String>,
        header_only: Option<bool>,
        system: Option<bool>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetConfig {
    #[serde(default)]
    pub frameworks: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(default)]
    pub defines: Vec<String>,
}

impl CuvManifest {
    pub fn find_in_ancestors(start: &Path) -> Option<PathBuf> {
        let mut curr = start.to_path_buf();
        loop {
            let candidate = curr.join("cuv.toml");
            if candidate.is_file() {
                return Some(candidate);
            }
            let candidate_alt = curr.join("cxx.toml");
            if candidate_alt.is_file() {
                return Some(candidate_alt);
            }
            if !curr.pop() {
                break;
            }
        }
        None
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest at {}", path.display()))?;
        let manifest: CuvManifest = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML in {}", path.display()))?;
        Ok(manifest)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let s = toml::to_string_pretty(self)?;
        std::fs::write(path, s)?;
        Ok(())
    }
}
