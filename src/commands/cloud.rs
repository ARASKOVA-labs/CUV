use crate::cloud::{clear_auth_config, load_auth_config, save_auth_config, CloudClient};
use anyhow::Result;
use colored::Colorize;

pub async fn handle_login(token: Option<String>, org: Option<String>) -> Result<()> {
    let tok = if let Some(t) = token {
        t
    } else {
        // Read from stdin or default test token
        println!("Enter your CUV Cloud API token: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    let mut cfg = load_auth_config().unwrap_or_default();
    cfg.token = Some(tok);
    if let Some(o) = org {
        cfg.org = Some(o);
    }
    save_auth_config(&cfg)?;

    println!(
        "\n{} Authenticated with {} {}",
        crate::ui::theme::success_icon(),
        "CUV Cloud".cyan().bold(),
        cfg.org.as_deref().unwrap_or("Personal").green()
    );
    println!("  • Cache Endpoint: {}\n", cfg.endpoint.dimmed());

    Ok(())
}

pub fn handle_logout() -> Result<()> {
    clear_auth_config()?;
    println!(
        "{} Logged out from CUV Cloud.",
        crate::ui::theme::success_icon()
    );
    Ok(())
}

pub fn handle_whoami() -> Result<()> {
    let cfg = load_auth_config()?;
    println!("\n{} Cloud Authentication Status", crate::ui::theme::logo_badge());
    println!("{}", crate::ui::theme::divider_line(52));

    if let Some(ref tok) = cfg.token {
        let masked = if tok.len() > 8 {
            format!("{}...{}", &tok[..4], &tok[tok.len() - 4..])
        } else {
            "********".to_string()
        };
        println!("  • Status:       {}", "Authenticated".green().bold());
        println!("  • Organization: {}", cfg.org.as_deref().unwrap_or("Personal").cyan());
        println!("  • Token:        {}", masked.dimmed());
        println!("  • Endpoint:     {}", cfg.endpoint.dimmed());
    } else {
        println!("  • Status:       {}", "Not logged in (Local cache only)".yellow());
        println!("  • Hint:         Run `cuv login --token <TOKEN>` to connect team cache.");
    }
    println!("{}\n", crate::ui::theme::divider_line(52));

    Ok(())
}

pub async fn handle_cloud_status() -> Result<()> {
    let client = CloudClient::new()?;
    let status = client.check_status().await?;

    println!("\n{} Cloud Remote Cache Status", crate::ui::theme::logo_badge());
    println!("{}", crate::ui::theme::divider_line(52));
    println!("  • Endpoint:       {}", status.endpoint.cyan());
    println!(
        "  • Connection:     {}",
        if status.connected {
            "Connected".green().bold()
        } else {
            "Offline / Unreachable".red()
        }
    );
    println!(
        "  • Authentication: {}",
        if status.authenticated {
            "Verified".green().bold()
        } else {
            "Unauthenticated (public fallback)".yellow()
        }
    );
    if let Some(org) = status.organization {
        println!("  • Team/Org:       {}", org.bold());
    }
    println!(
        "  • Latency:        {}",
        crate::ui::theme::format_duration(status.latency)
    );
    println!("{}\n", crate::ui::theme::divider_line(52));

    Ok(())
}
