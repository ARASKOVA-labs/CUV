use colored::Colorize;

pub fn logo_badge() -> String {
    format!("{} {}", "⚡".yellow().bold(), "CUV".cyan().bold())
}

pub fn success_icon() -> String {
    "✔".green().bold().to_string()
}

pub fn error_icon() -> String {
    "✖".red().bold().to_string()
}

pub fn package_icon() -> String {
    "📦".bright_cyan().bold().to_string()
}

pub fn test_icon() -> String {
    "🧪".magenta().bold().to_string()
}

pub fn rocket_icon() -> String {
    "🚀".bright_purple().bold().to_string()
}

pub fn sparkle_icon() -> String {
    "✨".bright_yellow().bold().to_string()
}

pub fn format_duration(dur: std::time::Duration) -> String {
    let millis = dur.as_millis();
    if millis < 1000 {
        format!("{}ms", millis).cyan().to_string()
    } else {
        format!("{:.2}s", dur.as_secs_f64()).cyan().to_string()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn divider_line(len: usize) -> String {
    "─".repeat(len).dimmed().to_string()
}
