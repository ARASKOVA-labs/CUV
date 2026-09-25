use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Toolchain {
    pub compiler_path: PathBuf, // C++ compiler path (backwards compatibility)
    pub cxx_path: PathBuf,
    pub cc_path: Option<PathBuf>,
    pub ar_path: Option<PathBuf>,
    pub compiler_name: String,
    pub version: String,
    pub target_triple: String,
    pub default_stdlib: String,
}

impl Toolchain {
    pub fn detect() -> Result<Self> {
        // Priority for C++: clang++ -> g++ -> c++
        let cxx_candidates = ["clang++", "g++", "c++"];
        let mut cxx_info = None;
        for c in cxx_candidates {
            if let Ok(path) = which::which(c) {
                if let Ok(info) = Self::probe_compiler(&path, c) {
                    cxx_info = Some((path, c.to_string(), info));
                    break;
                }
            }
        }

        let (cxx_path, cxx_name, (version, target_triple, default_stdlib)) = match cxx_info {
            Some(v) => v,
            None => bail!("No C++ compiler found on PATH (searched for clang++, g++, c++). Please install LLVM/Clang or GCC."),
        };

        // Priority for C: clang -> gcc -> cc
        let cc_candidates = ["clang", "gcc", "cc"];
        let mut cc_path = None;
        for c in cc_candidates {
            if let Ok(path) = which::which(c) {
                cc_path = Some(path);
                break;
            }
        }

        // Priority for static archiver: ar -> llvm-ar
        let ar_candidates = ["ar", "llvm-ar"];
        let mut ar_path = None;
        for a in ar_candidates {
            if let Ok(path) = which::which(a) {
                ar_path = Some(path);
                break;
            }
        }

        Ok(Self {
            compiler_path: cxx_path.clone(),
            cxx_path,
            cc_path,
            ar_path,
            compiler_name: cxx_name,
            version,
            target_triple,
            default_stdlib,
        })
    }

    pub fn compiler_for_file(&self, file: &Path) -> &PathBuf {
        let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "c" {
            self.cc_path.as_ref().unwrap_or(&self.cxx_path)
        } else {
            &self.cxx_path
        }
    }

    fn probe_compiler(path: &PathBuf, _name: &str) -> Result<(String, String, String)> {
        let output = Command::new(path)
            .arg("-v")
            .output()
            .with_context(|| format!("Failed to run {}", path.display()))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}\n{}", stdout, stderr);

        let mut target_triple = "unknown".to_string();
        let mut version = "unknown".to_string();

        for line in combined.lines() {
            if line.contains("Target:") {
                if let Some(target) = line.split("Target:").nth(1) {
                    target_triple = target.trim().to_string();
                }
            }
            if (line.contains("clang version") || line.contains("gcc version") || line.contains("Apple clang")) && version == "unknown" {
                version = line.trim().to_string();
            }
        }

        let default_stdlib = if cfg!(target_os = "macos") {
            "libc++".to_string()
        } else {
            "libstdc++".to_string()
        };

        Ok((version, target_triple, default_stdlib))
    }
}
