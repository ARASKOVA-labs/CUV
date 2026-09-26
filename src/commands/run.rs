use crate::compiler::driver::{BuildOptions, CompilerDriver};
use crate::core::manifest::CuvManifest;
use crate::package::cache::CacheManager;
use crate::package::resolver::ensure_dependencies;
use crate::toolchain::Toolchain;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::process::Command;
use std::time::Instant;

pub async fn handle_run(release: bool, verbose: bool, args: Vec<String>) -> Result<()> {
    let tc = Toolchain::detect()?;
    let cwd = std::env::current_dir()?;
    let manifest_path = CuvManifest::find_in_ancestors(&cwd)
        .context("No cuv.toml or cxx.toml found in current directory or parents.")?;
    let project_dir = manifest_path.parent().unwrap().to_path_buf();
    let mf = CuvManifest::load(&manifest_path)?;

    if mf.project.kind == "static-lib" || mf.project.kind == "shared-lib" {
        bail!(
            "Project `{}` is configured as a {} and cannot be executed directly. Use `cuv build` or `cuv test`.",
            mf.project.name,
            mf.project.kind
        );
    }
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
    let binary = driver.build(&BuildOptions { release, verbose }).await?;

    let run_title = format!(" 🚀 {} ", binary.display());
    let width: usize = 64;
    let border_len = width.saturating_sub(run_title.len() + 3);
    println!(
        "{}{}{}",
        "╭─".bright_purple(),
        run_title.cyan().bold(),
        "─".repeat(border_len).bright_purple()
    );

    let run_start = Instant::now();
    let status = Command::new(&binary).args(&args).status()?;
    let duration = run_start.elapsed();

    println!("╰{}╯", "─".repeat(width).bright_purple());
    if status.success() {
        println!(
            "{} Process exited with code 0 in {}\n",
            "✨".bright_yellow(),
            crate::ui::theme::format_duration(duration)
        );
    } else {
        println!(
            "{} Process exited with code {:?} in {}\n",
            "✖".red().bold(),
            status.code(),
            crate::ui::theme::format_duration(duration)
        );
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
