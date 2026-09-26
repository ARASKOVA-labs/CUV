use crate::core::manifest::CuvManifest;
use crate::package::cache::CacheManager;
use crate::package::registry::{get_known_registry, normalize_target_triple, PackageKind};
use crate::toolchain::Toolchain;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tar::Archive;

pub fn verify_sha256(bytes: &[u8], expected_hex: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = format!("{:x}", hasher.finalize());
    result.eq_ignore_ascii_case(expected_hex)
}

pub async fn resolve_and_fetch_package(
    package: &str,
    version_or_tag: Option<&str>,
    cache_root: &Path,
    project_dir: &Path,
) -> Result<PathBuf> {
    let registry = get_known_registry();
    let known_pkg = registry.get(package);

    let (repo, subpath, tag, is_binary) = if let Some(known) = known_pkg {
        (
            known.repo.to_string(),
            known.include_subpath.to_string(),
            version_or_tag.unwrap_or(known.default_tag).to_string(),
            matches!(known.kind, PackageKind::StaticLibrary { .. }),
        )
    } else if package.starts_with("github:") {
        let repo_part = package.strip_prefix("github:").unwrap();
        (
            repo_part.to_string(),
            "include".to_string(),
            version_or_tag.unwrap_or("main").to_string(),
            false,
        )
    } else if package.contains('/') {
        (
            package.to_string(),
            "include".to_string(),
            version_or_tag.unwrap_or("main").to_string(),
            false,
        )
    } else {
        bail!(
            "Unknown package '{}'. Use a recognized alias or 'github:owner/repo'.",
            package
        );
    };

    let safe_pkg_name = package.replace(['/', ':'], "_");
    let target_cache_dir = cache_root.join("packages").join(&safe_pkg_name).join(&tag);

    if !target_cache_dir.exists() {
        std::fs::create_dir_all(&target_cache_dir)?;
        let spinner = crate::ui::animation::create_spinner(&format!(
            "Fetching {} ({}) from GitHub/Registry...",
            package.cyan(),
            tag.dimmed()
        ));

        let client = reqwest::Client::builder()
            .user_agent("cuv-package-manager/0.1.0")
            .build()?;

        let mut downloaded_bytes = None;

        if let Some(known) = known_pkg {
            if let Ok(tc) = Toolchain::detect() {
                let normalized_triple = normalize_target_triple(&tc.target_triple);
                for artifact in known.artifacts {
                    if artifact.target_triple == normalized_triple {
                        if let Ok(resp) = client.get(artifact.url).send().await {
                            if resp.status().is_success() {
                                if let Ok(bytes) = resp.bytes().await {
                                    if artifact.sha256.is_empty()
                                        || verify_sha256(&bytes, artifact.sha256)
                                    {
                                        downloaded_bytes = Some(bytes);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if downloaded_bytes.is_none() {
            let mut urls_to_try = Vec::new();
            urls_to_try.push(format!(
                "https://github.com/{}/archive/refs/tags/{}.tar.gz",
                repo, tag
            ));
            if !tag.starts_with('v') {
                urls_to_try.push(format!(
                    "https://github.com/{}/archive/refs/tags/v{}.tar.gz",
                    repo, tag
                ));
            } else if let Some(stripped) = tag.strip_prefix('v') {
                urls_to_try.push(format!(
                    "https://github.com/{}/archive/refs/tags/{}.tar.gz",
                    repo, stripped
                ));
            }
            urls_to_try.push(format!(
                "https://github.com/{}/archive/refs/heads/{}.tar.gz",
                repo, tag
            ));
            urls_to_try.push(format!(
                "https://github.com/{}/archive/refs/heads/main.tar.gz",
                repo
            ));
            urls_to_try.push(format!(
                "https://github.com/{}/archive/refs/heads/master.tar.gz",
                repo
            ));

            for url in &urls_to_try {
                if let Ok(resp) = client.get(url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            downloaded_bytes = Some(bytes);
                            break;
                        }
                    }
                }
            }
        }

        let bytes = match downloaded_bytes {
            Some(b) => b,
            None => {
                spinner.finish_and_clear();
                bail!(
                    "Failed to download archive for '{}' from GitHub (tried tags & branches)",
                    package
                );
            }
        };

        let tar = GzDecoder::new(&bytes[..]);
        let mut archive = Archive::new(tar);
        archive
            .unpack(&target_cache_dir)
            .context("Failed to unpack package tarball")?;
        spinner.finish_and_clear();
    }

    let mut include_source = target_cache_dir.clone();
    let mut lib_source = None;

    if let Ok(mut entries) = std::fs::read_dir(&target_cache_dir) {
        if let Some(Ok(first)) = entries.next() {
            if first.file_type()?.is_dir() {
                let candidate = if subpath.is_empty() {
                    first.path()
                } else {
                    first.path().join(&subpath)
                };
                if candidate.exists() {
                    include_source = candidate;
                } else {
                    include_source = first.path();
                }

                let lib_candidate = first.path().join("lib");
                if lib_candidate.is_dir() {
                    lib_source = Some(lib_candidate);
                }
            }
        }
    }

    if lib_source.is_none() && target_cache_dir.join("lib").is_dir() {
        lib_source = Some(target_cache_dir.join("lib"));
    }

    let project_cuv_inc = project_dir.join(".cuv").join("include");
    std::fs::create_dir_all(&project_cuv_inc)?;
    CacheManager::link_tree(&include_source, &project_cuv_inc)?;

    println!(
        "  {} `{}` ({}) mounted to `.cuv/include/`",
        "✔".green().bold(),
        package.bold(),
        tag.dimmed()
    );

    if let Some(lib_dir) = lib_source {
        let project_cuv_lib = project_dir.join(".cuv").join("lib");
        std::fs::create_dir_all(&project_cuv_lib)?;
        CacheManager::link_tree(&lib_dir, &project_cuv_lib)?;
        println!(
            "  {} `{}` ({}) mounted pre-compiled libraries to `.cuv/lib/`",
            "⚡".yellow().bold(),
            package.bold(),
            tag.dimmed()
        );
    } else if is_binary {
        let project_cuv_lib = project_dir.join(".cuv").join("lib");
        std::fs::create_dir_all(&project_cuv_lib)?;
    }

    Ok(include_source)
}

pub async fn ensure_dependencies(
    manifest: &CuvManifest,
    cache_root: &Path,
    project_dir: &Path,
) -> Result<()> {
    if manifest.dependencies.is_empty() {
        return Ok(());
    }

    let cuv_include = project_dir.join(".cuv").join("include");
    let installed_file = project_dir.join(".cuv").join("installed.json");
    let mut installed_map: HashMap<String, String> = if installed_file.exists() {
        std::fs::read_to_string(&installed_file)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    let mut updated = false;
    for (pkg_name, dep) in &manifest.dependencies {
        let req_version = match dep {
            crate::core::manifest::Dependency::Version(v) => {
                if v == "*" {
                    None
                } else {
                    Some(v.as_str())
                }
            }
            crate::core::manifest::Dependency::Detailed { version, tag, .. } => {
                tag.as_deref().or(version.as_deref())
            }
        };

        let already_installed = if let Some(installed_ver) = installed_map.get(pkg_name) {
            if let Some(req) = req_version {
                installed_ver == req
            } else {
                true
            }
        } else {
            false
        };

        if !already_installed || !cuv_include.exists() {
            resolve_and_fetch_package(pkg_name, req_version, cache_root, project_dir).await?;
            installed_map.insert(
                pkg_name.clone(),
                req_version.unwrap_or("default").to_string(),
            );
            updated = true;
        }
    }

    if updated {
        let _ = std::fs::create_dir_all(project_dir.join(".cuv"));
        let _ = std::fs::write(
            &installed_file,
            serde_json::to_string_pretty(&installed_map)?,
        );
    }

    Ok(())
}
