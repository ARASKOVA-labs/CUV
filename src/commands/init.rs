use anyhow::{bail, Result};
use colored::Colorize;
use std::path::PathBuf;

pub fn handle_init(name: &str, standard: &str, is_lib: bool) -> Result<()> {
    let target_dir = if name == "." {
        std::env::current_dir()?
    } else {
        let p = PathBuf::from(name);
        std::fs::create_dir_all(&p)?;
        p
    };

    let proj_name = target_dir
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "my_cpp_project".to_string());

    let manifest_file = target_dir.join("cuv.toml");
    if manifest_file.exists() {
        bail!("A cuv.toml already exists in {}", target_dir.display());
    }

    let kind = if is_lib { "static-lib" } else { "executable" };

    let manifest_content = format!(
        r#"[project]
name = "{proj_name}"
version = "0.1.0"
standard = "{standard}"
kind = "{kind}"

[dependencies]
# fmt = "^10.2.0"
# nlohmann_json = "^3.11.3"

[target.macos]
# frameworks = ["CoreGraphics"]
"#
    );

    std::fs::write(&manifest_file, manifest_content)?;

    let src_dir = target_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    if is_lib {
        let lib_cpp = src_dir.join("lib.cpp");
        if !lib_cpp.exists() {
            let sample_lib = r#"#include <iostream>

extern "C" void hello_cuv() {
    std::cout << "Hello from CUV library component!" << std::endl;
}
"#;
            std::fs::write(&lib_cpp, sample_lib)?;
        }
        let inc_dir = target_dir.join("include");
        std::fs::create_dir_all(&inc_dir)?;
        let header = inc_dir.join(format!("{}.h", proj_name));
        if !header.exists() {
            std::fs::write(&header, "#pragma once\n\nextern \"C\" void hello_cuv();\n")?;
        }

        let tests_dir = target_dir.join("tests");
        std::fs::create_dir_all(&tests_dir)?;
        let test_file = tests_dir.join("test_lib.cpp");
        if !test_file.exists() {
            let sample_test = format!(
                r#"#include <cassert>
#include <iostream>
#include "{proj_name}.h"

int main() {{
    hello_cuv();
    std::cout << "All library unit tests passed!" << std::endl;
    return 0;
}}
"#
            );
            std::fs::write(&test_file, sample_test)?;
        }
    } else {
        let main_cpp = src_dir.join("main.cpp");
        if !main_cpp.exists() {
            let sample_cpp = r#"#include <iostream>
#include <string_view>

int main(int argc, char* argv[]) {
    constexpr std::string_view greeting = "Hello from CUV (C-Ultra-Velocity)!";
    std::cout << greeting << std::endl;
    std::cout << "Engine: High-Performance Modern C++ Toolchain by Araskova" << std::endl;
    return 0;
}
"#;
            std::fs::write(&main_cpp, sample_cpp)?;
        }
        let inc_dir = target_dir.join("include");
        std::fs::create_dir_all(&inc_dir)?;

        let tests_dir = target_dir.join("tests");
        std::fs::create_dir_all(&tests_dir)?;
        let test_file = tests_dir.join("test_basic.cpp");
        if !test_file.exists() {
            let sample_test = r#"#include <cassert>
#include <iostream>

int main() {
    assert(1 + 1 == 2);
    std::cout << "Basic tests passed!" << std::endl;
    return 0;
}
"#;
            std::fs::write(&test_file, sample_test)?;
        }
    }

    let gitignore = target_dir.join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(
            &gitignore,
            "target/\n.cuv/\ncompile_commands.json\n.DS_Store\n",
        )?;
    }

    println!(
        "{} Initialized {} `{}` with C++ standard {}",
        "✔".green().bold(),
        kind.cyan(),
        proj_name.bold(),
        standard.dimmed()
    );

    Ok(())
}
