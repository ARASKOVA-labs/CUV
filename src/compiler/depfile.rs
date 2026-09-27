use std::path::{Path, PathBuf};

pub fn parse_depfile(depfile_path: &Path) -> Vec<PathBuf> {
    let content = match std::fs::read_to_string(depfile_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    parse_depfile_content(&content)
}

pub fn parse_depfile_content(content: &str) -> Vec<PathBuf> {
    let bytes = content.as_bytes();
    let mut colon_pos = None;

    for (i, &b) in bytes.iter().enumerate() {
        if b == b':' {
            let is_drive_letter = (i == 1 || (i >= 2 && bytes[i - 2].is_ascii_whitespace()))
                && bytes[i - 1].is_ascii_alphabetic()
                && matches!(bytes.get(i + 1), Some(&b'/' | &b'\\'));
            if !is_drive_letter {
                colon_pos = Some(i);
                break;
            }
        }
    }

    let after_colon = match colon_pos {
        Some(pos) => &content[pos + 1..],
        None => return Vec::new(),
    };

    let mut deps = Vec::new();
    let mut current_dep = String::new();
    let chars: Vec<char> = after_colon.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        if ch == '\\' {
            if i + 1 < len {
                let next = chars[i + 1];
                if next == '\r' || next == '\n' {
                    i += 1;
                    if next == '\r' && i + 1 < len && chars[i + 1] == '\n' {
                        i += 1;
                    }
                    if !current_dep.is_empty() {
                        deps.push(PathBuf::from(std::mem::take(&mut current_dep)));
                    }
                } else if next == ' ' || next == '\t' {
                    current_dep.push(next);
                    i += 1;
                } else if next == '\\' {
                    current_dep.push('\\');
                    i += 1;
                } else {
                    current_dep.push('\\');
                }
            } else {
                current_dep.push('\\');
            }
        } else if ch.is_whitespace() {
            if !current_dep.is_empty() {
                deps.push(PathBuf::from(std::mem::take(&mut current_dep)));
            }
        } else {
            current_dep.push(ch);
        }
        i += 1;
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
    if deps.is_empty() {
        return true;
    }

    for dep in deps {
        let meta = std::fs::metadata(&dep).or_else(|_| {
            if dep.is_relative() {
                if let Some(parent) = src.parent() {
                    std::fs::metadata(parent.join(&dep))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "not found",
                    ))
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "not found",
                ))
            }
        });

        if let Ok(m) = meta {
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
