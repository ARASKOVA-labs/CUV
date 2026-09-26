use crate::core::manifest::TargetConfig;
use crate::toolchain::Toolchain;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub fn cuv_lib_link_args(project_dir: &Path) -> Vec<String> {
    let mut args = Vec::new();
    let cuv_lib = project_dir.join(".cuv").join("lib");
    if cuv_lib.is_dir() {
        args.push(format!("-L{}", cuv_lib.display()));
        if let Ok(entries) = std::fs::read_dir(&cuv_lib) {
            let mut libs = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if ext == "a" || ext == "dylib" || ext == "so" {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        let lib_name = stem.strip_prefix("lib").unwrap_or(stem);
                        libs.push(lib_name.to_string());
                    }
                }
            }
            libs.sort();
            for lib in libs {
                args.push(format!("-l{}", lib));
            }
        }
    }
    args
}

pub async fn link_static_lib(
    toolchain: &Toolchain,
    objects: &[PathBuf],
    output_path: &Path,
    verbose: bool,
) -> Result<()> {
    let ar = toolchain
        .ar_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("ar"));
    let mut cmd = Command::new(&ar);
    cmd.arg("rcs").arg(output_path);
    for obj in objects {
        cmd.arg(obj);
    }

    if verbose {
        println!("{} {:?}", "Archiving:".dimmed(), cmd);
    }

    let status = cmd.status().await.context("Failed to run archiver")?;
    if !status.success() {
        bail!(
            "Static library archiving failed with exit status {}",
            status
        );
    }
    Ok(())
}

pub async fn link_shared_lib(
    toolchain: &Toolchain,
    target_cfg: Option<&TargetConfig>,
    project_dir: &Path,
    objects: &[PathBuf],
    output_path: &Path,
    verbose: bool,
) -> Result<()> {
    let mut cmd = Command::new(&toolchain.cxx_path);
    if cfg!(target_os = "macos") {
        cmd.arg("-dynamiclib");
    } else {
        cmd.arg("-shared");
    }

    apply_target_link_flags(&mut cmd, target_cfg, project_dir);

    for obj in objects {
        cmd.arg(obj);
    }
    cmd.arg("-o").arg(output_path);

    if verbose {
        println!("{} {:?}", "Linking shared library:".dimmed(), cmd);
    }

    let status = cmd.status().await.context("Failed to run linker")?;
    if !status.success() {
        bail!("Shared library linking failed with exit status {}", status);
    }
    Ok(())
}

pub async fn link_executable(
    toolchain: &Toolchain,
    target_cfg: Option<&TargetConfig>,
    project_dir: &Path,
    objects: &[PathBuf],
    output_path: &Path,
    release: bool,
    verbose: bool,
) -> Result<()> {
    let mut cmd = Command::new(&toolchain.cxx_path);
    if release {
        cmd.arg("-O3");
    } else {
        cmd.arg("-g");
    }

    apply_target_link_flags(&mut cmd, target_cfg, project_dir);

    for obj in objects {
        cmd.arg(obj);
    }
    cmd.arg("-o").arg(output_path);

    if verbose {
        println!("{} {:?}", "Linking executable:".dimmed(), cmd);
    }

    let status = cmd.status().await.context("Failed to run linker")?;
    if !status.success() {
        bail!("Linking failed with exit status {}", status);
    }
    Ok(())
}

pub fn apply_target_link_flags(
    cmd: &mut Command,
    target_cfg: Option<&TargetConfig>,
    project_dir: &Path,
) {
    for arg in cuv_lib_link_args(project_dir) {
        cmd.arg(arg);
    }

    if let Some(cfg) = target_cfg {
        if cfg!(target_os = "macos") {
            for fw in &cfg.frameworks {
                cmd.arg("-framework").arg(fw);
            }
        }
        for f in &cfg.flags {
            cmd.arg(f);
        }
        for l in &cfg.links {
            cmd.arg(format!("-l{}", l));
        }
    }
}
