use crate::compiler::driver::{BuildOptions, CompilerDriver};
use crate::core::manifest::CuvManifest;
use crate::package::cache::CacheManager;
use crate::package::resolver::ensure_dependencies;
use crate::toolchain::Toolchain;
use anyhow::{Context, Result};
use colored::Colorize;

pub async fn handle_build(release: bool, verbose: bool) -> Result<()> {
    let tc = Toolchain::detect()?;
    let cwd = std::env::current_dir()?;
    let manifest_path = CuvManifest::find_in_ancestors(&cwd)
        .context("No cuv.toml or cxx.toml found in current directory or parents.")?;
    let project_dir = manifest_path.parent().unwrap().to_path_buf();
    let mf = CuvManifest::load(&manifest_path)?;

    // Auto-sync dependencies if missing
    let cache = CacheManager::default_dir()?;
    ensure_dependencies(&mf, &cache.root, &project_dir).await?;

    println!(
        "{} {} v{} ({} [{}])",
        "⚡".yellow(),
        mf.project.name.bold(),
        mf.project.version.dimmed(),
        tc.compiler_name.cyan(),
        tc.target_triple.dimmed()
    );

    let driver = CompilerDriver::new(tc, mf, project_dir);
    driver.build(&BuildOptions { release, verbose }).await?;

    Ok(())
}
