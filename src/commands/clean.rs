use crate::core::manifest::CuvManifest;
use anyhow::{bail, Result};
use colored::Colorize;

pub fn handle_clean() -> Result<()> {
    let cwd = std::env::current_dir()?;
    if let Some(mf_path) = CuvManifest::find_in_ancestors(&cwd) {
        let target_dir = mf_path.parent().unwrap().join("target");
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
            println!("{} Removed target/", "✔".green().bold());
        } else {
            println!("{} Nothing to clean.", "Note:".dimmed());
        }
    } else {
        bail!("No cuv.toml found.");
    }
    Ok(())
}
