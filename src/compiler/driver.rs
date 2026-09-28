use crate::compiler::depfile::{any_object_newer, needs_recompile};
use crate::compiler::linker::{link_executable, link_shared_lib, link_static_lib};
use crate::core::manifest::CuvManifest;
use crate::package::cache::CacheManager;
use crate::toolchain::Toolchain;
use crate::ui::animation::{create_progress_bar, print_build_summary};
use anyhow::{bail, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::process::Command;

pub struct BuildOptions {
    pub release: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CompileResult {
    pub objects: Vec<PathBuf>,
    pub recompiled: usize,
    pub global_cached: usize,
    pub local_cached: usize,
}

pub struct CompilerDriver {
    pub toolchain: Toolchain,
    pub manifest: CuvManifest,
    pub project_dir: PathBuf,
}

impl CompilerDriver {
    pub fn new(toolchain: Toolchain, manifest: CuvManifest, project_dir: PathBuf) -> Self {
        Self {
            toolchain,
            manifest,
            project_dir,
        }
    }

    pub fn discover_sources(&self) -> Vec<PathBuf> {
        let src_dir = self.project_dir.join("src");
        let mut sources = Vec::new();
        if src_dir.is_dir() {
            for entry in walkdir::WalkDir::new(&src_dir).into_iter().flatten() {
                if entry.file_type().is_file() {
                    let ext = entry
                        .path()
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    if ["cpp", "cc", "cxx", "c"].contains(&ext) {
                        sources.push(entry.into_path());
                    }
                }
            }
        }
        sources.sort();
        sources
    }

    pub fn get_object_and_dep_path(&self, src: &Path, mode: &str) -> (PathBuf, PathBuf) {
        let rel = src.strip_prefix(&self.project_dir).unwrap_or(src);
        let obj_dir = self.project_dir.join("target").join(mode).join("obj");
        let obj_path = obj_dir.join(rel).with_extension("o");
        let dep_path = obj_dir.join(rel).with_extension("d");
        (obj_path, dep_path)
    }

    pub fn target_artifact_path(&self, mode: &str) -> PathBuf {
        let target_dir = self.project_dir.join("target").join(mode);
        let name = &self.manifest.project.name;
        match self.manifest.project.kind.as_str() {
            "static-lib" => target_dir.join(format!("lib{}.a", name)),
            "shared-lib" => {
                if cfg!(target_os = "macos") {
                    target_dir.join(format!("lib{}.dylib", name))
                } else if cfg!(target_os = "windows") {
                    target_dir.join(format!("{}.dll", name))
                } else {
                    target_dir.join(format!("lib{}.so", name))
                }
            }
            _ => {
                if cfg!(target_os = "windows") {
                    target_dir.join(format!("{}.exe", name))
                } else {
                    target_dir.join(name)
                }
            }
        }
    }

    pub fn generate_compile_commands(&self, sources: &[PathBuf], target_mode: &str) -> Result<()> {
        let mut entries = Vec::new();
        let include_dir = self.project_dir.join("include");
        let cuv_include = self.project_dir.join(".cuv").join("include");

        let target_cfg = if cfg!(target_os = "macos") {
            self.manifest
                .target
                .get("macos")
                .or_else(|| self.manifest.target.get("darwin"))
        } else if cfg!(target_os = "linux") {
            self.manifest.target.get("linux")
        } else if cfg!(target_os = "windows") {
            self.manifest.target.get("windows")
        } else {
            None
        };

        let mut defines = target_cfg.map(|t| t.defines.clone()).unwrap_or_default();
        if self.manifest.dependencies.contains_key("fmt")
            && !defines.iter().any(|d| d.starts_with("FMT_HEADER_ONLY"))
        {
            defines.push("FMT_HEADER_ONLY=1".to_string());
        }

        for src in sources {
            let (obj_path, _) = self.get_object_and_dep_path(src, target_mode);
            let compiler = self.toolchain.compiler_for_file(src);
            let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("");

            let mut args = vec![compiler.to_string_lossy().into_owned()];

            if ext == "c" {
                args.push("-std=c17".to_string());
            } else {
                args.push(format!("-std={}", self.manifest.project.standard));
            }

            if target_mode == "release" {
                args.push("-O3".to_string());
                args.push("-DNDEBUG".to_string());
            } else {
                args.push("-O0".to_string());
                args.push("-g".to_string());
                args.push("-DDEBUG".to_string());
            }

            for def in &defines {
                args.push(format!("-D{}", def));
            }

            if include_dir.exists() {
                args.push(format!("-I{}", include_dir.display()));
            }
            if cuv_include.exists() {
                args.push(format!("-I{}", cuv_include.display()));
            }

            args.push("-c".to_string());
            args.push(src.to_string_lossy().into_owned());
            args.push("-o".to_string());
            args.push(obj_path.to_string_lossy().into_owned());

            entries.push(serde_json::json!({
                "directory": self.project_dir.to_string_lossy(),
                "arguments": args,
                "file": src.to_string_lossy(),
                "output": obj_path.to_string_lossy()
            }));
        }

        let cc_path = self.project_dir.join("compile_commands.json");
        let json_str = serde_json::to_string_pretty(&entries)?;
        std::fs::write(&cc_path, json_str)?;
        Ok(())
    }

    pub async fn compile_objects(&self, options: &BuildOptions) -> Result<CompileResult> {
        let mode = if options.release { "release" } else { "debug" };
        let sources = self.discover_sources();
        if sources.is_empty() {
            bail!(
                "No C/C++ source files found in {}",
                self.project_dir.join("src").display()
            );
        }

        self.generate_compile_commands(&sources, mode)?;

        let include_dir = self.project_dir.join("include");
        let cuv_include = self.project_dir.join(".cuv").join("include");
        let cache_mgr = CacheManager::default_dir()?;
        let search_paths = vec![
            include_dir.clone(),
            self.project_dir.join("src"),
            cuv_include.clone(),
        ];

        let target_cfg = if cfg!(target_os = "macos") {
            self.manifest
                .target
                .get("macos")
                .or_else(|| self.manifest.target.get("darwin"))
        } else if cfg!(target_os = "linux") {
            self.manifest.target.get("linux")
        } else if cfg!(target_os = "windows") {
            self.manifest.target.get("windows")
        } else {
            None
        };

        let mut defines = target_cfg.map(|t| t.defines.clone()).unwrap_or_default();
        if self.manifest.dependencies.contains_key("fmt")
            && !defines.iter().any(|d| d.starts_with("FMT_HEADER_ONLY"))
        {
            defines.push("FMT_HEADER_ONLY=1".to_string());
        }

        let mut tasks_to_run = Vec::new();
        let mut all_objects = Vec::new();
        let mut local_cached = 0;

        for src in &sources {
            let (obj_path, dep_path) = self.get_object_and_dep_path(src, mode);
            all_objects.push(obj_path.clone());

            if needs_recompile(src, &obj_path, &dep_path) {
                tasks_to_run.push((src.clone(), obj_path, dep_path));
            } else {
                local_cached += 1;
            }
        }

        let total_tasks = tasks_to_run.len();
        let recompiled_count = Arc::new(AtomicUsize::new(0));
        let global_cached_count = Arc::new(AtomicUsize::new(0));

        if total_tasks > 0 {
            let concurrency = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

            let progress_bar = Arc::new(create_progress_bar(total_tasks as u64, "Compiling"));

            let mut handles = Vec::new();

            for (src, obj_path, dep_path) in tasks_to_run {
                if let Some(parent) = obj_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let sem = semaphore.clone();
                let recompiled = recompiled_count.clone();
                let global_cached = global_cached_count.clone();
                let pb = progress_bar.clone();
                let compiler = self.toolchain.compiler_for_file(&src).clone();
                let ext = src
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let standard = self.manifest.project.standard.clone();
                let is_release = options.release;
                let inc_dir = include_dir.clone();
                let cuv_inc = cuv_include.clone();
                let search_paths = search_paths.clone();
                let defines = defines.clone();
                let flags = target_cfg.map(|t| t.flags.clone()).unwrap_or_default();
                let verbose = options.verbose;
                let proj_dir = self.project_dir.clone();
                let tc_version = self.toolchain.version.clone();
                let tc_triple = self.toolchain.target_triple.clone();
                let cache = cache_mgr.clone();

                let handle = tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();

                    let opt_level = if is_release { "release" } else { "debug" };
                    let std_flag = if ext == "c" { "c17" } else { &standard };

                    let obj_hash = CacheManager::compute_object_hash(
                        &src,
                        &search_paths,
                        &tc_version,
                        &tc_triple,
                        std_flag,
                        opt_level,
                        &flags,
                        &defines,
                    )
                    .ok();

                    if let Some(ref hash) = obj_hash {
                        if cache
                            .restore_cached_object(hash, &obj_path, &dep_path, &src, &search_paths)
                            .unwrap_or(false)
                        {
                            global_cached.fetch_add(1, Ordering::SeqCst);
                            let display_path = src.strip_prefix(&proj_dir).unwrap_or(&src);
                            pb.set_message(format!(
                                "{} {}",
                                display_path.display().to_string().cyan(),
                                "[⚡cache hit]".green()
                            ));
                            pb.inc(1);
                            return Ok(());
                        }
                    }

                    let mut cmd = Command::new(&compiler);
                    if ext == "c" {
                        cmd.arg("-std=c17");
                    } else {
                        cmd.arg(format!("-std={}", standard));
                    }

                    if is_release {
                        cmd.arg("-O3").arg("-DNDEBUG");
                    } else {
                        cmd.arg("-O0").arg("-g").arg("-DDEBUG");
                    }

                    if inc_dir.exists() {
                        cmd.arg(format!("-I{}", inc_dir.display()));
                    }
                    if cuv_inc.exists() {
                        cmd.arg(format!("-I{}", cuv_inc.display()));
                    }

                    for def in &defines {
                        cmd.arg(format!("-D{}", def));
                    }
                    for flg in &flags {
                        cmd.arg(flg);
                    }

                    cmd.arg("-MMD").arg("-MF").arg(&dep_path);
                    cmd.arg("-c").arg(&src);
                    cmd.arg("-o").arg(&obj_path);

                    if verbose {
                        println!("{} {:?}", "Compiling:".dimmed(), cmd);
                    }

                    let output = cmd.output().await?;
                    if !output.status.success() {
                        pb.finish_and_clear();
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        bail!(
                            "Compilation failed for `{}`:\n{}{}",
                            src.display(),
                            stdout,
                            stderr
                        );
                    }

                    if let Some(ref hash) = obj_hash {
                        let _ = cache.store_object(hash, &obj_path);
                    }

                    recompiled.fetch_add(1, Ordering::SeqCst);
                    let display_path = src.strip_prefix(&proj_dir).unwrap_or(&src);
                    pb.set_message(format!("{}", display_path.display().to_string().cyan()));
                    pb.inc(1);

                    Ok(())
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.await??;
            }

            progress_bar.finish_and_clear();
        }

        let recompiled = recompiled_count.load(Ordering::SeqCst);
        let global_cached = global_cached_count.load(Ordering::SeqCst);

        Ok(CompileResult {
            objects: all_objects,
            recompiled,
            global_cached,
            local_cached,
        })
    }

    pub fn non_main_objects(&self, all_objects: &[PathBuf]) -> Vec<PathBuf> {
        let entry_stem = self.manifest.project.entry.as_deref().unwrap_or("main");
        all_objects
            .iter()
            .filter(|obj| {
                let stem = obj.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                stem != entry_stem
            })
            .cloned()
            .collect()
    }

    pub async fn build(&self, options: &BuildOptions) -> Result<PathBuf> {
        let start = std::time::Instant::now();
        let mode = if options.release { "release" } else { "debug" };
        let output_path = self.target_artifact_path(mode);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let compile_res = self.compile_objects(options).await?;

        let need_link = compile_res.recompiled > 0
            || compile_res.global_cached > 0
            || !output_path.exists()
            || any_object_newer(&compile_res.objects, &output_path);

        let target_cfg = if cfg!(target_os = "macos") {
            self.manifest
                .target
                .get("macos")
                .or_else(|| self.manifest.target.get("darwin"))
        } else if cfg!(target_os = "linux") {
            self.manifest.target.get("linux")
        } else if cfg!(target_os = "windows") {
            self.manifest.target.get("windows")
        } else {
            None
        };

        if need_link {
            match self.manifest.project.kind.as_str() {
                "static-lib" => {
                    link_static_lib(
                        &self.toolchain,
                        &compile_res.objects,
                        &output_path,
                        options.verbose,
                    )
                    .await?;
                }
                "shared-lib" => {
                    link_shared_lib(
                        &self.toolchain,
                        target_cfg,
                        &self.project_dir,
                        &compile_res.objects,
                        &output_path,
                        options.verbose,
                    )
                    .await?;
                }
                _ => {
                    link_executable(
                        &self.toolchain,
                        target_cfg,
                        &self.project_dir,
                        &compile_res.objects,
                        &output_path,
                        options.release,
                        options.verbose,
                    )
                    .await?;
                }
            }

            print_build_summary(
                &self.manifest.project.name,
                &self.manifest.project.kind,
                &self.manifest.project.standard,
                mode,
                &output_path,
                compile_res.objects.len(),
                compile_res.recompiled,
                compile_res.global_cached,
                start.elapsed(),
            );
        } else {
            print_build_summary(
                &self.manifest.project.name,
                &self.manifest.project.kind,
                &self.manifest.project.standard,
                &format!("{} · up-to-date", mode),
                &output_path,
                compile_res.objects.len(),
                0,
                0,
                start.elapsed(),
            );
        }

        Ok(output_path)
    }
}
