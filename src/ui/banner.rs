use colored::Colorize;
use std::io::{stdout, IsTerminal, Write};
use std::time::Duration;

pub const CUV_ASCII_LOGO: &[&str] = &[
    r"   ______  __  ___    __ ",
    r"  / ____/ / / / / |  / / ",
    r" / /     / / / /| | / /  ",
    r"/ /___  / /_/ / | |/ /   ",
    r"\____/  \____/  |___/    ",
];

pub const TAGLINE: &str = "⚡ C-Ultra-Velocity  •  The uv and bun for C/C++";
pub const SUB_TAGLINE: &str =
    "Sub-millisecond builds, zero-config package manager, universal ABI cache";

/// Prints the static stylized gradient banner
pub fn print_static_banner() {
    println!();
    for (i, line) in CUV_ASCII_LOGO.iter().enumerate() {
        let colored_line = match i {
            0 => line.bright_cyan().bold(),
            1 => line.cyan().bold(),
            2 => line.bright_blue().bold(),
            3 => line.blue().bold(),
            _ => line.magenta().bold(),
        };
        println!("{}", colored_line);
    }
    println!("  {}", TAGLINE.bright_yellow().bold());
    println!("  {}\n", SUB_TAGLINE.dimmed());
}

/// Plays an animated wave shimmer over the ASCII logo
pub async fn play_animated_shimmer() {
    if !stdout().is_terminal() {
        print_static_banner();
        return;
    }

    let colors = [
        colored::Color::BrightCyan,
        colored::Color::Cyan,
        colored::Color::BrightYellow,
        colored::Color::Yellow,
        colored::Color::BrightGreen,
        colored::Color::Green,
    ];

    print!("\x1B[?25l"); // Hide cursor
    let _ = stdout().flush();

    for frame in 0..colors.len() {
        // Clear terminal lines if not first
        if frame > 0 {
            print!("\x1B[{}A", CUV_ASCII_LOGO.len() + 2);
        }

        for (row, line) in CUV_ASCII_LOGO.iter().enumerate() {
            let color_idx = (frame + row) % colors.len();
            let c = colors[color_idx];
            println!("{}", line.color(c).bold());
        }
        println!("  {}", TAGLINE.bright_yellow().bold());
        println!("  {}", SUB_TAGLINE.dimmed());
        let _ = stdout().flush();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    print!("\x1B[?25h"); // Restore cursor
    let _ = stdout().flush();
    println!();
}

/// Helper to render a futuristic rounded HUD box
pub fn render_card(title: &str, rows: &[(&str, String)]) {
    let width: usize = 64;
    let title_badge = format!(" ⚡ {} ", title);
    let border_len = width.saturating_sub(title_badge.len() + 3);

    println!(
        "{}{}{}",
        "╭─".bright_cyan(),
        title_badge.bright_yellow().bold(),
        "─".repeat(border_len).bright_cyan()
    );

    for (label, val) in rows {
        let label_fmt = format!(" {:<18}", label);
        let content = format!("{} {}", label_fmt.cyan(), val);
        println!(
            "{} {:<62} {}",
            "│".bright_cyan(),
            content,
            "│".bright_cyan()
        );
    }

    println!("╰{}╯", "─".repeat(width).bright_cyan());
}
