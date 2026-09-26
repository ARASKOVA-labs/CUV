use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CacheManager {
    pub root: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub object_count: usize,
    pub object_bytes: u64,
    pub package_count: usize,
    pub package_bytes: u64,
    pub total_bytes: u64,
    pub root_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct CacheCleanStats {
    pub evicted_objects: usize,
    pub evicted_packages: usize,
    pub freed_bytes: u64,
}

impl CacheManager {
    pub fn default_dir() -> Result<Self> {
        let root = if let Ok(custom) = std::env::var("CUV_CACHE_DIR") {
            PathBuf::from(custom)
        } else {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".cuv").join("cache")
        };
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn obj_cache_dir(&self) -> PathBuf {
        self.root.join("obj")
    }

    pub fn collect_headers(file_path: &Path, search_paths: &[PathBuf]) -> Vec<PathBuf> {
        let mut visited = HashSet::new();
        let mut headers = Vec::new();
        if let Ok(canon) = file_path.canonicalize() {
            visited.insert(canon);
        }
        Self::scan_headers_recursive(file_path, search_paths, &mut visited, &mut headers, 0);
        headers
    }

    fn scan_headers_recursive(
        file_path: &Path,
        search_paths: &[PathBuf],
        visited: &mut HashSet<PathBuf>,
        headers: &mut Vec<PathBuf>,
        depth: usize,
    ) {
        if depth > 10 {
            return;
        }
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("#include") {
                continue;
            }
            let rest = trimmed["#include".len()..].trim();
            if rest.is_empty() {
                continue;
            }
            let header_name = if (rest.starts_with('"') && rest.ends_with('"'))
                || (rest.starts_with('<') && rest.ends_with('>'))
            {
                &rest[1..rest.len() - 1]
            } else {
                continue;
            };

            let mut resolved = None;
            if let Some(parent) = file_path.parent() {
                let candidate = parent.join(header_name);
                if candidate.is_file() {
                    resolved = Some(candidate);
                }
            }

            if resolved.is_none() {
                for sp in search_paths {
                    let candidate = sp.join(header_name);
                    if candidate.is_file() {
                        resolved = Some(candidate);
                        break;
                    }
                }
            }

            if let Some(h_path) = resolved {
                if let Ok(canon) = h_path.canonicalize() {
                    if visited.insert(canon.clone()) {
                        headers.push(canon.clone());
                        Self::scan_headers_recursive(
                            &canon,
                            search_paths,
                            visited,
                            headers,
                            depth + 1,
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_object_hash(
        src_path: &Path,
        search_paths: &[PathBuf],
        compiler_version: &str,
        target_triple: &str,
        standard: &str,
        opt_level: &str,
        extra_flags: &[String],
        defines: &[String],
    ) -> Result<String> {
        let mut hasher = Sha256::new();

        let src_bytes = std::fs::read(src_path)?;
        hasher.update(&src_bytes);

        let headers = Self::collect_headers(src_path, search_paths);
        for h in headers {
            hasher.update(b"|INC:");
            hasher.update(h.to_string_lossy().as_bytes());
            if let Ok(bytes) = std::fs::read(&h) {
                hasher.update(&bytes);
            }
        }

        hasher.update(b"|VER:");
        hasher.update(compiler_version.as_bytes());
        hasher.update(b"|TRIPLE:");
        hasher.update(target_triple.as_bytes());
        hasher.update(b"|STD:");
        hasher.update(standard.as_bytes());
        hasher.update(b"|OPT:");
        hasher.update(opt_level.as_bytes());

        for flg in extra_flags {
            hasher.update(b"|FLAG:");
            hasher.update(flg.as_bytes());
        }
        for def in defines {
            hasher.update(b"|DEF:");
            hasher.update(def.as_bytes());
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn get_cached_object(&self, hash: &str) -> Option<PathBuf> {
        let path = self.obj_cache_dir().join(format!("{}.o", hash));
        if path.is_file() {
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.len() > 0 {
                    return Some(path);
                }
            }
        }
        None
    }

    pub fn store_object(&self, hash: &str, obj_path: &Path) -> Result<PathBuf> {
        let obj_dir = self.obj_cache_dir();
        std::fs::create_dir_all(&obj_dir)?;

        let target_obj = obj_dir.join(format!("{}.o", hash));
        if !target_obj.exists() && std::fs::hard_link(obj_path, &target_obj).is_err() {
            let _ = std::fs::copy(obj_path, &target_obj);
        }

        Ok(target_obj)
    }

    pub fn restore_cached_object(
        &self,
        hash: &str,
        dst_obj: &Path,
        dst_dep: &Path,
        src_file: &Path,
        search_paths: &[PathBuf],
    ) -> Result<bool> {
        if let Some(cached_obj) = self.get_cached_object(hash) {
            if let Some(parent) = dst_obj.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let _ = std::fs::remove_file(dst_obj);
            let linked = std::fs::hard_link(&cached_obj, dst_obj).is_ok();
            if !linked {
                std::fs::copy(&cached_obj, dst_obj)?;
            }
            let now = std::time::SystemTime::now();
            if let Ok(file) = std::fs::OpenOptions::new().write(true).open(dst_obj) {
                let times = std::fs::FileTimes::new().set_modified(now);
                let _ = file.set_times(times);
            }

            let headers = Self::collect_headers(src_file, search_paths);
            let mut dep_content = format!("{}: {}", dst_obj.display(), src_file.display());
            for h in headers {
                dep_content.push(' ');
                dep_content.push_str(&h.display().to_string());
            }
            dep_content.push('\n');
            let _ = std::fs::write(dst_dep, dep_content);

            return Ok(true);
        }
        Ok(false)
    }

    pub fn get_cache_stats(&self) -> Result<CacheStats> {
        let mut stats = CacheStats {
            root_dir: self.root.clone(),
            ..Default::default()
        };

        let obj_dir = self.obj_cache_dir();
        if obj_dir.is_dir() {
            for entry in std::fs::read_dir(&obj_dir)?.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("o") {
                    stats.object_count += 1;
                    if let Ok(meta) = entry.metadata() {
                        stats.object_bytes += meta.len();
                    }
                }
            }
        }

        if self.root.is_dir() {
            for entry in std::fs::read_dir(&self.root)?.flatten() {
                let name = entry.file_name();
                if name != "obj" && entry.path().is_dir() {
                    stats.package_count += 1;
                    for pe in walkdir::WalkDir::new(entry.path()).into_iter().flatten() {
                        if pe.file_type().is_file() {
                            if let Ok(meta) = pe.metadata() {
                                stats.package_bytes += meta.len();
                            }
                        }
                    }
                }
            }
        }

        stats.total_bytes = stats.object_bytes + stats.package_bytes;
        Ok(stats)
    }

    pub fn clean_cache(&self, obj_only: bool) -> Result<CacheCleanStats> {
        let mut stats = CacheCleanStats::default();

        let obj_dir = self.obj_cache_dir();
        if obj_dir.is_dir() {
            for entry in std::fs::read_dir(&obj_dir)?.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = entry.metadata() {
                        stats.freed_bytes += meta.len();
                    }
                    if path.extension().and_then(|s| s.to_str()) == Some("o") {
                        stats.evicted_objects += 1;
                    }
                    let _ = std::fs::remove_file(&path);
                }
            }
        }

        if !obj_only && self.root.is_dir() {
            for entry in std::fs::read_dir(&self.root)?.flatten() {
                let name = entry.file_name();
                if name != "obj" && entry.path().is_dir() {
                    stats.evicted_packages += 1;
                    for pe in walkdir::WalkDir::new(entry.path()).into_iter().flatten() {
                        if pe.file_type().is_file() {
                            if let Ok(meta) = pe.metadata() {
                                stats.freed_bytes += meta.len();
                            }
                        }
                    }
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }

        Ok(stats)
    }

    pub fn compute_abi_hash(
        package: &str,
        version: &str,
        compiler_version: &str,
        cxx_std: &str,
        target_triple: &str,
        opt_level: &str,
        extra_flags: &[String],
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(package.as_bytes());
        hasher.update(b":");
        hasher.update(version.as_bytes());
        hasher.update(b":");
        hasher.update(compiler_version.as_bytes());
        hasher.update(b":");
        hasher.update(cxx_std.as_bytes());
        hasher.update(b":");
        hasher.update(target_triple.as_bytes());
        hasher.update(b":");
        hasher.update(opt_level.as_bytes());
        for flag in extra_flags {
            hasher.update(b":");
            hasher.update(flag.as_bytes());
        }
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string()
    }

    pub fn get_package_dir(&self, package: &str, abi_hash: &str) -> PathBuf {
        self.root.join(package).join(abi_hash)
    }

    pub fn link_tree(src: &Path, dst: &Path) -> Result<()> {
        std::fs::create_dir_all(dst)?;
        if src.exists() {
            for entry in walkdir::WalkDir::new(src) {
                let entry = entry?;
                let rel = entry.path().strip_prefix(src)?;
                if rel.as_os_str().is_empty() {
                    continue;
                }
                let target = dst.join(rel);
                if entry.file_type().is_dir() {
                    let _ = std::fs::create_dir_all(&target);
                } else if entry.file_type().is_file() {
                    if let Some(p) = target.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    let _ = std::fs::remove_file(&target);
                    let _ = std::fs::hard_link(entry.path(), &target)
                        .or_else(|_| std::fs::copy(entry.path(), &target).map(|_| ()));
                }
            }
        }
        Ok(())
    }
}
