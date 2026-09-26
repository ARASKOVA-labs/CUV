use crate::package::cache::CacheManager;
use crate::toolchain::Toolchain;
use anyhow::Result;
use colored::Colorize;

pub fn handle_info() -> Result<()> {
    let tc = Toolchain::detect()?;
    let cache = CacheManager::default_dir()?;

    println!(
        "\n{} Toolchain & Environment",
        crate::ui::theme::logo_badge()
    );
    println!("{}", crate::ui::theme::divider_line(52));
    println!(
        "  {} Compiler Path:  {}",
        "•".cyan(),
        tc.compiler_path.display()
    );
    println!(
        "  {} Compiler Name:  {}",
        "•".cyan(),
        tc.compiler_name.bold()
    );
    println!("  {} Version:        {}", "•".cyan(), tc.version.dimmed());
    println!(
        "  {} Target Triple:  {}",
        "•".cyan(),
        tc.target_triple.cyan()
    );
    println!("  {} Default Stdlib: {}", "•".cyan(), tc.default_stdlib);
    if let Some(ar) = &tc.ar_path {
        println!("  {} Archiver:       {}", "•".cyan(), ar.display());
    }
    if let Some(cc) = &tc.cc_path {
        println!("  {} C Compiler:     {}", "•".cyan(), cc.display());
    }
    println!(
        "  {} Cache Root:     {}",
        "•".cyan(),
        cache.root.display().to_string().dimmed()
    );
    println!("{}\n", crate::ui::theme::divider_line(52));

    Ok(())
}
