use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Duration;

pub fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .expect("Valid progress template"),
    );
    pb.set_message(msg.to_string());
    pb
}

pub fn create_progress_bar(total: u64, prefix: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold} {spinner:.cyan} [{elapsed_precise}] [{bar:24.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .expect("Valid bar template")
            .progress_chars("━╸─"),
    );
    pb.set_prefix(prefix.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

#[allow(clippy::too_many_arguments)]
pub fn print_build_summary(
    project_name: &str,
    kind: &str,
    standard: &str,
    mode: &str,
    artifact_path: &Path,
    total_objects: usize,
    recompiled: usize,
    global_cached: usize,
    duration: Duration,
) {
    let local_cached = total_objects.saturating_sub(recompiled + global_cached);
    let size_str = if let Ok(meta) = std::fs::metadata(artifact_path) {
        crate::ui::theme::format_bytes(meta.len())
    } else {
        "unknown".to_string()
    };

    println!(
        "\n{} {} `{}` ({}) in {}",
        "⚡".yellow().bold(),
        "Built".green().bold(),
        project_name.bold(),
        kind.cyan(),
        crate::ui::theme::format_duration(duration)
    );
    let cache_info = if global_cached > 0 {
        format!(
            "({} recompiled, {} {}cached, {} local cached)",
            recompiled.to_string().cyan(),
            global_cached.to_string().green().bold(),
            "⚡".yellow(),
            local_cached.to_string().dimmed()
        )
    } else {
        format!(
            "({} recompiled, {} cached)",
            recompiled.to_string().cyan(),
            local_cached.to_string().green()
        )
    };
    println!(
        "   {} Objects: {} {}",
        "├──".dimmed(),
        total_objects.to_string().bold(),
        cache_info
    );
    println!(
        "   {} Profile: {} ({})",
        "├──".dimmed(),
        mode.bold(),
        standard.dimmed()
    );
    println!(
        "   {} Output:  {} ({})\n",
        "└──".dimmed(),
        artifact_path.display().to_string().cyan(),
        size_str.dimmed()
    );
}

pub fn print_test_suite_result(name: &str, passed: bool, dur: Duration, details: Option<&str>) {
    let time_str = format!("({})", crate::ui::theme::format_duration(dur));
    if passed {
        println!(
            "  {} {} {}",
            "✔".green().bold(),
            name.bold(),
            time_str.dimmed()
        );
    } else {
        println!(
            "  {} {} {}",
            "✖".red().bold(),
            name.bold(),
            time_str.dimmed()
        );
        if let Some(err) = details {
            for line in err.lines() {
                println!("    {}", line.dimmed());
            }
        }
    }
}

pub fn print_test_summary(passed: usize, failed: usize, duration: Duration) {
    println!("{}", crate::ui::theme::divider_line(52));
    if failed == 0 {
        println!(
            "{} All test suites passed! {} passed in {}\n",
            "✨".bright_yellow(),
            passed.to_string().green().bold(),
            crate::ui::theme::format_duration(duration)
        );
    } else {
        println!(
            "{} Tests failed: {} passed, {} failed in {}\n",
            "✖".red().bold(),
            passed.to_string().green().bold(),
            failed.to_string().red().bold(),
            crate::ui::theme::format_duration(duration)
        );
    }
}
