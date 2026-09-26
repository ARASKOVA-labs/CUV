use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_cuv_bin() -> PathBuf {
    let mut bin = std::env::current_exe().expect("current exe");
    bin.pop();
    if bin.ends_with("deps") {
        bin.pop();
    }
    bin.join("cuv")
}

fn create_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cuv_test_{}_{}",
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
fn test_executable_build_and_incremental_cache() {
    let temp_dir = create_temp_dir("exec_cache");
    let cuv = get_cuv_bin();

    let status = Command::new(&cuv)
        .arg("init")
        .current_dir(&temp_dir)
        .status()
        .expect("cuv init");
    assert!(status.success());
    assert!(temp_dir.join("cuv.toml").exists());
    assert!(temp_dir.join("src").join("main.cpp").exists());

    let output = Command::new(&cuv)
        .arg("build")
        .current_dir(&temp_dir)
        .output()
        .expect("cuv build");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Built") || stdout.contains("Compiling"));

    let proj_name = temp_dir.file_name().unwrap().to_str().unwrap();
    let bin_path = temp_dir.join("target").join("debug").join(proj_name);
    assert!(bin_path.exists());

    let run_output = Command::new(&bin_path).output().expect("run binary");
    assert!(run_output.status.success());
    assert!(String::from_utf8_lossy(&run_output.stdout).contains("Hello from CUV"));

    let output2 = Command::new(&cuv)
        .arg("build")
        .current_dir(&temp_dir)
        .output()
        .expect("cuv build again");
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("up-to-date"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_static_library_build_and_test() {
    let temp_dir = create_temp_dir("static_lib");
    let cuv = get_cuv_bin();

    let status = Command::new(&cuv)
        .arg("init")
        .arg("--lib")
        .current_dir(&temp_dir)
        .status()
        .expect("cuv init --lib");
    assert!(status.success());

    let output = Command::new(&cuv)
        .arg("build")
        .current_dir(&temp_dir)
        .output()
        .expect("cuv build");
    assert!(output.status.success());

    let proj_name = temp_dir.file_name().unwrap().to_str().unwrap();
    let lib_path = temp_dir
        .join("target")
        .join("debug")
        .join(format!("lib{}.a", proj_name));
    assert!(
        lib_path.exists(),
        "Expected static archive at {}",
        lib_path.display()
    );

    let test_output = Command::new(&cuv)
        .arg("test")
        .current_dir(&temp_dir)
        .output()
        .expect("cuv test");
    let test_stdout = String::from_utf8_lossy(&test_output.stdout);
    assert!(
        test_output.status.success(),
        "cuv test failed: {}",
        test_stdout
    );
    assert!(test_stdout.contains("✔") || test_stdout.contains("passed"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_test_runner_linking_project_code() {
    let temp_dir = create_temp_dir("runner_link");
    let cuv = get_cuv_bin();

    let status = Command::new(&cuv)
        .arg("init")
        .current_dir(&temp_dir)
        .status()
        .expect("cuv init");
    assert!(status.success());

    fs::write(
        temp_dir.join("include").join("calc.h"),
        "#pragma once\nint multiply(int a, int b);\n",
    )
    .unwrap();
    fs::write(
        temp_dir.join("src").join("calc.cpp"),
        "#include \"calc.h\"\nint multiply(int a, int b) { return a * b; }\n",
    )
    .unwrap();

    let test_code = r#"#include "calc.h"
#include <cassert>
#include <iostream>

int main() {
    assert(multiply(6, 7) == 42);
    std::cout << "Multiply test passed!" << std::endl;
    return 0;
}
"#;
    fs::write(temp_dir.join("tests").join("test_calc.cpp"), test_code).unwrap();

    let test_output = Command::new(&cuv)
        .arg("test")
        .current_dir(&temp_dir)
        .output()
        .expect("cuv test");
    let stdout = String::from_utf8_lossy(&test_output.stdout);
    assert!(test_output.status.success(), "cuv test failed: {}", stdout);
    assert!(stdout.contains("test_calc") && (stdout.contains("✔") || stdout.contains("passed")));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dependency_auto_sync() {
    let temp_dir = create_temp_dir("auto_sync");
    let cuv = get_cuv_bin();

    let status = Command::new(&cuv)
        .arg("init")
        .current_dir(&temp_dir)
        .status()
        .expect("cuv init");
    assert!(status.success());

    let manifest_path = temp_dir.join("cuv.toml");
    let manifest_content = fs::read_to_string(&manifest_path).unwrap();
    let updated_manifest =
        manifest_content.replace("[dependencies]", "[dependencies]\nfmt = \"10.2.1\"");
    fs::write(&manifest_path, updated_manifest).unwrap();

    assert!(!temp_dir.join(".cuv").exists());

    let sync_output = Command::new(&cuv)
        .arg("sync")
        .current_dir(&temp_dir)
        .output()
        .expect("cuv sync");
    let sync_stdout = String::from_utf8_lossy(&sync_output.stdout);
    let sync_stderr = String::from_utf8_lossy(&sync_output.stderr);
    assert!(
        sync_output.status.success(),
        "cuv sync failed:\nstdout: {}\nstderr: {}",
        sync_stdout,
        sync_stderr
    );

    assert!(temp_dir
        .join(".cuv")
        .join("include")
        .join("fmt")
        .join("core.h")
        .exists());

    let _ = fs::remove_dir_all(&temp_dir);
}
