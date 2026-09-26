use crate::core::manifest::CuvManifest;
use crate::interop::cmake::{export_cmake_lists, generate_cmake_provider};
use anyhow::{bail, Context, Result};
use colored::Colorize;

pub fn handle_export(format: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    if format == "provider" {
        let out_path = cwd.join("cuv.cmake");
        generate_cmake_provider(&out_path)?;
        println!("{} Exported `{}`", "✔".green().bold(), out_path.display());
    } else if format == "cmake" {
        let manifest_path = CuvManifest::find_in_ancestors(&cwd)
            .context("No cuv.toml found to export CMakeLists.txt from.")?;
        let mf = CuvManifest::load(&manifest_path)?;
        let out_path = manifest_path.parent().unwrap().join("CMakeLists.txt");
        export_cmake_lists(&mf, &out_path)?;
        println!("{} Exported `{}`", "✔".green().bold(), out_path.display());
    } else {
        bail!(
            "Unsupported export format: '{}'. Supported: cmake, provider",
            format
        );
    }
    Ok(())
}
