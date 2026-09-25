use anyhow::{bail, Result};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInfo {
    pub file_path: PathBuf,
    pub module_name: Option<String>,
    pub is_interface: bool,
    pub is_implementation: bool,
    pub imported_modules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleDAG {
    pub modules: HashMap<String, ModuleInfo>,
    pub consumers: Vec<ModuleInfo>,
}

impl ModuleDAG {
    pub fn scan(sources: &[PathBuf]) -> Result<Self> {
        let mut modules = HashMap::new();
        let mut consumers = Vec::new();

        for src in sources {
            let info = Self::scan_file(src)?;
            if let Some(ref mod_name) = info.module_name {
                if info.is_interface {
                    modules.insert(mod_name.clone(), info);
                } else {
                    consumers.push(info);
                }
            } else if !info.imported_modules.is_empty() {
                consumers.push(info);
            }
        }

        Ok(Self { modules, consumers })
    }

    pub fn has_modules(&self) -> bool {
        !self.modules.is_empty()
    }

    pub fn scan_file(file_path: &Path) -> Result<ModuleInfo> {
        let content = std::fs::read_to_string(file_path).unwrap_or_default();
        let mut module_name = None;
        let mut is_interface = false;
        let mut is_implementation = false;
        let mut imported_modules = Vec::new();

        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let is_module_ext = ["cppm", "ixx", "cxxm", "ccm"].contains(&ext);

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            if trimmed.starts_with("export") {
                let rest = trimmed["export".len()..].trim();
                if rest.starts_with("module") {
                    let after_module = rest["module".len()..].trim();
                    if let Some(semi) = after_module.find(';') {
                        let name = after_module[..semi].trim();
                        module_name = Some(name.to_string());
                        is_interface = true;
                    }
                } else if rest.starts_with("import") {
                    let after_import = rest["import".len()..].trim();
                    if let Some(semi) = after_import.find(';') {
                        let name = after_import[..semi].trim();
                        if !name.starts_with('<') && !name.starts_with('"') {
                            imported_modules.push(name.to_string());
                        }
                    }
                }
            } else if trimmed.starts_with("module") {
                let rest = trimmed["module".len()..].trim();
                if let Some(semi) = rest.find(';') {
                    let name = rest[..semi].trim();
                    if !name.is_empty() {
                        module_name = Some(name.to_string());
                        is_implementation = true;
                    }
                }
            } else if trimmed.starts_with("import") {
                let rest = trimmed["import".len()..].trim();
                if let Some(semi) = rest.find(';') {
                    let name = rest[..semi].trim();
                    if !name.starts_with('<') && !name.starts_with('"') {
                        imported_modules.push(name.to_string());
                    }
                }
            }
        }

        if is_module_ext && module_name.is_none() {
            let stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("module");
            module_name = Some(stem.to_string());
            is_interface = true;
        }

        Ok(ModuleInfo {
            file_path: file_path.to_path_buf(),
            module_name,
            is_interface,
            is_implementation,
            imported_modules,
        })
    }

    /// Topologically sort module interfaces so dependencies are built before dependents.
    pub fn topological_stages(&self) -> Result<Vec<Vec<ModuleInfo>>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        for mod_name in self.modules.keys() {
            in_degree.insert(mod_name.clone(), 0);
            dependents.insert(mod_name.clone(), Vec::new());
        }

        for (mod_name, info) in &self.modules {
            for dep in &info.imported_modules {
                if self.modules.contains_key(dep) {
                    *in_degree.entry(mod_name.clone()).or_insert(0) += 1;
                    dependents.entry(dep.clone()).or_default().push(mod_name.clone());
                }
            }
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(k, _)| k.clone())
            .collect();

        let mut stages = Vec::new();
        let mut visited_count = 0;

        while !queue.is_empty() {
            let stage_size = queue.len();
            let mut current_stage = Vec::new();

            for _ in 0..stage_size {
                let mod_name = queue.pop_front().unwrap();
                visited_count += 1;

                if let Some(info) = self.modules.get(&mod_name) {
                    current_stage.push(info.clone());
                }

                if let Some(deps) = dependents.get(&mod_name) {
                    for dep in deps {
                        if let Some(deg) = in_degree.get_mut(dep) {
                            *deg -= 1;
                            if *deg == 0 {
                                queue.push_back(dep.clone());
                            }
                        }
                    }
                }
            }

            stages.push(current_stage);
        }

        if visited_count < self.modules.len() {
            bail!("Cyclic dependency detected among C++20 module interfaces!");
        }

        Ok(stages)
    }

    pub fn precompile_args(
        standard: &str,
        modules_dir: &Path,
        src: &Path,
        pcm_out: &Path,
    ) -> Vec<String> {
        vec![
            format!("-std={}", standard),
            "--precompile".to_string(),
            "-x".to_string(),
            "c++-module".to_string(),
            src.to_string_lossy().into_owned(),
            format!("-fprebuilt-module-path={}", modules_dir.display()),
            "-o".to_string(),
            pcm_out.to_string_lossy().into_owned(),
        ]
    }

    pub fn consumer_module_args(modules_dir: &Path) -> Vec<String> {
        vec![format!("-fprebuilt-module-path={}", modules_dir.display())]
    }
}
