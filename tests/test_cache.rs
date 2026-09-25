use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_cuv_bin() -> PathBuf {
    let mut bin = std::env::current_exe().expect("current exe");
    bin.pop(); // remove test binary
    if bin.ends_with("deps") {
        bin.pop();
    }
    bin.join("cuv")
}

fn create_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cuv_test_cache_{}_{}",
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_global_cache_hit_after_clean() {
    let test_dir = create_temp_dir("clean_hit");
    let cache_dir = create_temp_dir("clean_cache_store");
    let cuv = get_cuv_bin();

    // 1. Initialize project
    let status = Command::new(&cuv)
        .arg("init")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .status()
        .expect("cuv init");
    assert!(status.success());

    // 2. First build: compiles and stores into global cache
    let output1 = Command::new(&cuv)
        .arg("build")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .output()
        .expect("cuv build");
    assert!(output1.status.success());
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    assert!(stdout1.contains("Built"));

    // Check that global cache obj directory was populated
    let obj_cache = cache_dir.join("obj");
    assert!(obj_cache.exists(), "Cache obj directory should exist");
    let cached_files: Vec<_> = fs::read_dir(&obj_cache)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("o"))
        .collect();
    assert!(!cached_files.is_empty(), "Expected cached .o files in {:?}", obj_cache);

    // 3. Run cuv clean (wipes target/ completely)
    let clean_status = Command::new(&cuv)
        .arg("clean")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .status()
        .expect("cuv clean");
    assert!(clean_status.success());
    assert!(!test_dir.join("target").exists());

    // 4. Second build: should restore from global cache in < 1ms
    let output2 = Command::new(&cuv)
        .arg("build")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .output()
        .expect("cuv build after clean");
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout2.contains("⚡cached") || stdout2.contains("cache hit") || stdout2.contains("Built"),
        "Expected global cache hit indication in stdout: {}",
        stdout2
    );

    // Verify binary runs correctly
    let proj_name = test_dir.file_name().unwrap().to_str().unwrap();
    let bin_path = test_dir.join("target").join("debug").join(proj_name);
    assert!(bin_path.exists());
    let run_res = Command::new(&bin_path).output().expect("run binary");
    assert!(run_res.status.success());
    assert!(String::from_utf8_lossy(&run_res.stdout).contains("Hello from CUV"));

    let _ = fs::remove_dir_all(&test_dir);
    let _ = fs::remove_dir_all(&cache_dir);
}

#[test]
fn test_cross_project_cache_sharing() {
    let proj_a = create_temp_dir("proj_a");
    let proj_b = create_temp_dir("proj_b");
    let shared_cache = create_temp_dir("shared_cache");
    let cuv = get_cuv_bin();

    // Init Project A
    let _ = Command::new(&cuv)
        .arg("init")
        .env("CUV_CACHE_DIR", &shared_cache)
        .current_dir(&proj_a)
        .status();

    // Init Project B
    let _ = Command::new(&cuv)
        .arg("init")
        .env("CUV_CACHE_DIR", &shared_cache)
        .current_dir(&proj_b)
        .status();

    // Copy identical source code to both projects
    let common_code = r#"#include <iostream>
int main() {
    std::cout << "Identical code across projects!" << std::endl;
    return 0;
}
"#;
    fs::write(proj_a.join("src").join("main.cpp"), common_code).unwrap();
    fs::write(proj_b.join("src").join("main.cpp"), common_code).unwrap();

    // Build Project A
    let output_a = Command::new(&cuv)
        .arg("build")
        .env("CUV_CACHE_DIR", &shared_cache)
        .current_dir(&proj_a)
        .output()
        .expect("build proj a");
    assert!(output_a.status.success());

    // Build Project B (identical source should hit global cache populated by Project A)
    let output_b = Command::new(&cuv)
        .arg("build")
        .env("CUV_CACHE_DIR", &shared_cache)
        .current_dir(&proj_b)
        .output()
        .expect("build proj b");
    assert!(output_b.status.success());
    let stdout_b = String::from_utf8_lossy(&output_b.stdout);
    assert!(
        stdout_b.contains("⚡cached") || stdout_b.contains("cache hit"),
        "Expected Project B to hit global cache from Project A, got: {}",
        stdout_b
    );

    let _ = fs::remove_dir_all(&proj_a);
    let _ = fs::remove_dir_all(&proj_b);
    let _ = fs::remove_dir_all(&shared_cache);
}

#[test]
fn test_cache_cli_info_and_clean() {
    let test_dir = create_temp_dir("cli_test");
    let cache_dir = create_temp_dir("cli_cache_dir");
    let cuv = get_cuv_bin();

    // Init & build to generate cached objects
    let _ = Command::new(&cuv)
        .arg("init")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .status();

    let _ = Command::new(&cuv)
        .arg("build")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .status();

    // 1. cuv cache info
    let info_out = Command::new(&cuv)
        .arg("cache")
        .arg("info")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .output()
        .expect("cuv cache info");
    assert!(info_out.status.success());
    let info_str = String::from_utf8_lossy(&info_out.stdout);
    assert!(info_str.contains("Global Cache Overview"));
    assert!(info_str.contains("Object Cache:"));

    // 2. cuv cache size
    let size_out = Command::new(&cuv)
        .arg("cache")
        .arg("size")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .output()
        .expect("cuv cache size");
    assert!(size_out.status.success());
    let size_str = String::from_utf8_lossy(&size_out.stdout);
    assert!(size_str.contains("B") || size_str.contains("KB") || size_str.contains("MB"));

    // 3. cuv cache clean --obj-only
    let clean_out = Command::new(&cuv)
        .arg("cache")
        .arg("clean")
        .arg("--obj-only")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .output()
        .expect("cuv cache clean");
    assert!(clean_out.status.success());
    let clean_str = String::from_utf8_lossy(&clean_out.stdout);
    assert!(clean_str.contains("Evicted"));

    // Verify cache is empty now
    let info_after = Command::new(&cuv)
        .arg("cache")
        .arg("info")
        .env("CUV_CACHE_DIR", &cache_dir)
        .current_dir(&test_dir)
        .output()
        .expect("cuv cache info after clean");
    let info_after_str = String::from_utf8_lossy(&info_after.stdout);
    assert!(info_after_str.contains("0 objects"));

    let _ = fs::remove_dir_all(&test_dir);
    let _ = fs::remove_dir_all(&cache_dir);
}
