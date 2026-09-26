use crate::package::cache::CacheManager;
use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

#[derive(Subcommand, Debug, Clone)]
pub enum CacheAction {
    #[command(about = "Display global cache location, object count, and size breakdown")]
    Info,
    #[command(about = "Evict cached items from the global content-addressable cache")]
    Clean {
        #[arg(
            long,
            help = "Only evict compilation objects, preserving downloaded packages"
        )]
        obj_only: bool,
    },
    #[command(about = "Print total disk size consumed by the global cache")]
    Size,
}

pub fn handle_cache(action: CacheAction) -> Result<()> {
    let cache = CacheManager::default_dir()?;

    match action {
        CacheAction::Info => {
            let stats = cache.get_cache_stats()?;
            println!("\n{} Global Cache Overview", crate::ui::theme::logo_badge());
            println!("{}", crate::ui::theme::divider_line(52));
            println!(
                "  {} Root Directory:    {}",
                "•".cyan(),
                stats.root_dir.display().to_string().cyan()
            );
            println!(
                "  {} Object Cache:      {} ({} objects)",
                "•".cyan(),
                crate::ui::theme::format_bytes(stats.object_bytes)
                    .green()
                    .bold(),
                stats.object_count.to_string().bold()
            );
            println!(
                "  {} Package Cache:     {} ({} packages)",
                "•".cyan(),
                crate::ui::theme::format_bytes(stats.package_bytes)
                    .green()
                    .bold(),
                stats.package_count.to_string().bold()
            );
            println!(
                "  {} Total Footprint:   {}",
                "•".cyan(),
                crate::ui::theme::format_bytes(stats.total_bytes)
                    .yellow()
                    .bold()
            );
            println!("{}\n", crate::ui::theme::divider_line(52));
        }
        CacheAction::Clean { obj_only } => {
            let clean_stats = cache.clean_cache(obj_only)?;
            let freed_str = crate::ui::theme::format_bytes(clean_stats.freed_bytes);
            if obj_only {
                println!(
                    "{} Evicted {} compilation objects (freed {}).",
                    crate::ui::theme::success_icon(),
                    clean_stats.evicted_objects.to_string().cyan().bold(),
                    freed_str.green().bold()
                );
            } else {
                let mut parts = Vec::new();
                if clean_stats.evicted_objects > 0 {
                    parts.push(format!("{} objects", clean_stats.evicted_objects));
                }
                if clean_stats.evicted_packages > 0 {
                    parts.push(format!("{} packages", clean_stats.evicted_packages));
                }
                let desc = if parts.is_empty() {
                    "cache entries".to_string()
                } else {
                    parts.join(", ")
                };
                println!(
                    "{} Purged global cache: {} (freed {}).",
                    crate::ui::theme::success_icon(),
                    desc.cyan(),
                    freed_str.green().bold()
                );
            }
        }
        CacheAction::Size => {
            let stats = cache.get_cache_stats()?;
            println!(
                "{} ({})",
                crate::ui::theme::format_bytes(stats.total_bytes)
                    .yellow()
                    .bold(),
                stats.root_dir.display().to_string().dimmed()
            );
        }
    }

    Ok(())
}
