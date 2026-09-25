use std::path::{Path, PathBuf};

pub fn parse_depfile(depfile_path: &Path) -> Vec<PathBuf> {
    let content = match std::fs::read_to_string(depfile_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let after_colon = match content.split_once(':') {
        Some((_, rest)) => rest,
        None => return Vec::new(),
    };

    let mut deps = Vec::new();
    let mut current_dep = String::new();
    let mut escaped = false;

    for ch in after_colon.chars() {
        if escaped {
            if ch != '\n' && ch != '\r' {
                current_dep.push(ch);
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch.is_whitespace() {
            if !current_dep.is_empty() {
                deps.push(PathBuf::from(std::mem::take(&mut current_dep)));
            }
        } else {
            current_dep.push(ch);
        }
    }
    if !current_dep.is_empty() {
        deps.push(PathBuf::from(current_dep));
    }
    deps
}

pub fn needs_recompile(src: &Path, obj: &Path, depfile: &Path) -> bool {
    let obj_meta = match std::fs::metadata(obj) {
        Ok(m) => m,
        Err(_) => return true,
    };
    let obj_mtime = match obj_meta.modified() {
        Ok(t) => t,
        Err(_) => return true,
    };

    let src_mtime = match std::fs::metadata(src).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };
    if src_mtime > obj_mtime {
        return true;
    }

    if !depfile.exists() {
        return true;
    }

    let deps = parse_depfile(depfile);
    for dep in deps {
        if let Ok(m) = std::fs::metadata(&dep) {
            if let Ok(dep_mtime) = m.modified() {
                if dep_mtime > obj_mtime {
                    return true;
                }
            }
        } else {
            return true;
        }
    }

    false
}

pub fn any_object_newer(objects: &[PathBuf], target: &Path) -> bool {
    let target_mtime = match std::fs::metadata(target).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };

    for obj in objects {
        if let Ok(m) = std::fs::metadata(obj) {
            if let Ok(obj_mtime) = m.modified() {
                if obj_mtime > target_mtime {
                    return true;
                }
            }
        }
    }
    false
}
