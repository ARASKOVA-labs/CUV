use crate::core::manifest::CuvManifest;
use crate::package::cache::CacheManager;
use crate::package::resolver::ensure_dependencies;
use crate::test_runner::runner::TestRunner;
use crate::toolchain::Toolchain;
use anyhow::{Context, Result};

pub async fn handle_test() -> Result<()> {
    let tc = Toolchain::detect()?;
    let cwd = std::env::current_dir()?;
    let manifest_path = CuvManifest::find_in_ancestors(&cwd)
        .context("No cuv.toml or cxx.toml found in current directory or parents.")?;
    let project_dir = manifest_path.parent().unwrap().to_path_buf();
    let mf = CuvManifest::load(&manifest_path)?;
    let cache = CacheManager::default_dir()?;
    ensure_dependencies(&mf, &cache.root, &project_dir).await?;

    let runner = TestRunner::new(tc, mf, project_dir);
    runner.run_all().await?;

    Ok(())
}
