use crate::core::manifest::{CuvManifest, Dependency};
use crate::package::cache::CacheManager;
use crate::package::resolver::resolve_and_fetch_package;
use anyhow::{Context, Result};
use colored::Colorize;

pub async fn handle_add(package: &str, version: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let manifest_path = CuvManifest::find_in_ancestors(&cwd)
        .context("No cuv.toml or cxx.toml found in current directory or parents.")?;
    let project_dir = manifest_path.parent().unwrap().to_path_buf();
    let mut mf = CuvManifest::load(&manifest_path)?;

    let cache = CacheManager::default_dir()?;

    // Fetch and mount the package
    resolve_and_fetch_package(package, version.as_deref(), &cache.root, &project_dir).await?;

    let dep = if let Some(v) = version {
        Dependency::Version(v)
    } else {
        Dependency::Version("*".to_string())
    };

    mf.dependencies.insert(package.to_string(), dep);
    mf.save(&manifest_path)?;

    println!(
        "{} Added `{}` to dependencies in {}",
        "✔".green().bold(),
        package.bold(),
        manifest_path.file_name().unwrap().to_string_lossy().dimmed()
    );

    Ok(())
}
