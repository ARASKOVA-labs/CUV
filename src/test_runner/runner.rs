use crate::compiler::driver::{BuildOptions, CompilerDriver};
use crate::core::manifest::CuvManifest;
use crate::toolchain::Toolchain;
use crate::ui::animation::{print_test_suite_result, print_test_summary};
use anyhow::{bail, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

pub struct TestRunner {
    pub toolchain: Toolchain,
    pub manifest: CuvManifest,
    pub project_dir: PathBuf,
}

impl TestRunner {
    pub fn new(toolchain: Toolchain, manifest: CuvManifest, project_dir: PathBuf) -> Self {
        Self {
            toolchain,
            manifest,
            project_dir,
        }
    }

    pub fn discover_test_files(&self) -> Vec<PathBuf> {
        let mut tests = Vec::new();
        let candidate_dirs = [
            self.project_dir.join("tests"),
            self.project_dir.join("test"),
        ];
        for dir in &candidate_dirs {
            if dir.is_dir() {
                for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
                    if entry.file_type().is_file() {
                        let ext = entry
                            .path()
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("");
                        if ["cpp", "cc", "cxx"].contains(&ext) {
                            tests.push(entry.into_path());
                        }
                    }
                }
            }
        }
        tests.sort();
        tests
    }

    pub async fn run_all(&self) -> Result<()> {
        let test_files = self.discover_test_files();
        if test_files.is_empty() {
            println!(
                "{} No tests found in `tests/` or `test/`.",
                "Note:".dimmed()
            );
            return Ok(());
        }

        let target_test_dir = self.project_dir.join("target").join("tests");
        std::fs::create_dir_all(&target_test_dir)?;

        let overall_start = Instant::now();

        let driver = CompilerDriver::new(
            self.toolchain.clone(),
            self.manifest.clone(),
            self.project_dir.clone(),
        );

        let non_main_objs = if !driver.discover_sources().is_empty() {
            let res = driver
                .compile_objects(&BuildOptions {
                    release: false,
                    verbose: false,
                })
                .await?;
            driver.non_main_objects(&res.objects)
        } else {
            Vec::new()
        };

        println!(
            "\n{} {} {} test suite(s)...",
            "🧪".magenta(),
            "Running".cyan().bold(),
            test_files.len().to_string().bold()
        );

        let mut passed = 0;
        let mut failed = 0;

        for test_src in &test_files {
            let test_start = Instant::now();
            let test_name = test_src.file_stem().unwrap().to_string_lossy();
            let bin_path = target_test_dir.join(format!("test_{}", test_name));

            let mut cmd = Command::new(&self.toolchain.compiler_path);
            cmd.arg(format!("-std={}", self.manifest.project.standard));
            cmd.arg("-O0").arg("-g");

            let inc_dir = self.project_dir.join("include");
            if inc_dir.exists() {
                cmd.arg(format!("-I{}", inc_dir.display()));
            }
            let cuv_inc = self.project_dir.join(".cuv").join("include");
            if cuv_inc.exists() {
                cmd.arg(format!("-I{}", cuv_inc.display()));
            }

            cmd.arg(test_src);
            for obj in &non_main_objs {
                cmd.arg(obj);
            }

            let target_cfg = if cfg!(target_os = "macos") {
                self.manifest
                    .target
                    .get("macos")
                    .or_else(|| self.manifest.target.get("darwin"))
            } else if cfg!(target_os = "linux") {
                self.manifest.target.get("linux")
            } else {
                None
            };
            for arg in crate::compiler::linker::cuv_lib_link_args(&self.project_dir) {
                cmd.arg(arg);
            }
            if let Some(cfg) = target_cfg {
                if cfg!(target_os = "macos") {
                    for fw in &cfg.frameworks {
                        cmd.arg("-framework").arg(fw);
                    }
                }
                for f in &cfg.flags {
                    cmd.arg(f);
                }
                for l in &cfg.links {
                    cmd.arg(format!("-l{}", l));
                }
            }

            cmd.arg("-o").arg(&bin_path);

            let compile_output = cmd.output()?;
            if !compile_output.status.success() {
                let stderr = String::from_utf8_lossy(&compile_output.stderr);
                print_test_suite_result(&test_name, false, test_start.elapsed(), Some(&stderr));
                failed += 1;
                continue;
            }

            let run_output = Command::new(&bin_path).output()?;
            if run_output.status.success() {
                print_test_suite_result(&test_name, true, test_start.elapsed(), None);
                passed += 1;
            } else {
                let stderr = String::from_utf8_lossy(&run_output.stderr);
                let stdout = String::from_utf8_lossy(&run_output.stdout);
                let combined = format!("{}{}", stdout, stderr);
                print_test_suite_result(&test_name, false, test_start.elapsed(), Some(&combined));
                failed += 1;
            }
        }

        print_test_summary(passed, failed, overall_start.elapsed());

        if failed > 0 {
            bail!("One or more tests failed.");
        }

        Ok(())
    }
}
