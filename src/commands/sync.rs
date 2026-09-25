use crate::core::manifest::CuvManifest;
use crate::package::cache::CacheManager;
use crate::package::resolver::ensure_dependencies;
use anyhow::{Context, Result};
use colored::Colorize;

pub async fn handle_sync() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let manifest_path = CuvManifest::find_in_ancestors(&cwd)
        .context("No cuv.toml or cxx.toml found in current directory or parents.")?;
    let project_dir = manifest_path.parent().unwrap().to_path_buf();
    let mf = CuvManifest::load(&manifest_path)?;
    let cache = CacheManager::default_dir()?;

    ensure_dependencies(&mf, &cache.root, &project_dir).await?;
    println!("{} Dependencies are up-to-date.", "✔".green().bold());
    Ok(())
}
