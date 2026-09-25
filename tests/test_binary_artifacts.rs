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
        "cuv_test_binary_{}_{}",
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
fn test_precompiled_binary_auto_linking() {
    let test_dir = create_temp_dir("auto_link");
    let cuv = get_cuv_bin();

    // 1. Init project
    let status = Command::new(&cuv)
        .arg("init")
        .current_dir(&test_dir)
        .status()
        .expect("cuv init");
    assert!(status.success());

    // 2. Build a real static library archive (.a) to simulate a pre-compiled binary artifact
    let lib_build_dir = create_temp_dir("build_mock_lib");
    let util_c = lib_build_dir.join("testutil.c");
    let util_o = lib_build_dir.join("testutil.o");
    let util_a = lib_build_dir.join("libtestutil.a");

    fs::write(
        &util_c,
        r#"
int get_secret_number(void) {
    return 4242;
}
"#,
    )
    .unwrap();

    // Compile with host cc
    let cc_status = Command::new("cc")
        .arg("-c")
        .arg(&util_c)
        .arg("-o")
        .arg(&util_o)
        .status()
        .expect("compile mock lib object");
    assert!(cc_status.success());

    let ar_status = Command::new("ar")
        .arg("rcs")
        .arg(&util_a)
        .arg(&util_o)
        .status()
        .expect("ar mock lib");
    assert!(ar_status.success());

    // 3. Mount into .cuv/lib/ and .cuv/include/
    let cuv_lib = test_dir.join(".cuv").join("lib");
    let cuv_inc = test_dir.join(".cuv").join("include");
    fs::create_dir_all(&cuv_lib).unwrap();
    fs::create_dir_all(&cuv_inc).unwrap();

    fs::copy(&util_a, cuv_lib.join("libtestutil.a")).unwrap();
    fs::write(
        cuv_inc.join("testutil.h"),
        r#"
#pragma once
extern "C" int get_secret_number(void);
"#,
    )
    .unwrap();

    // 4. Update main.cpp to call testutil
    let main_cpp = test_dir.join("src").join("main.cpp");
    fs::write(
        &main_cpp,
        r#"
#include <iostream>
#include "testutil.h"

int main() {
    std::cout << "Secret is: " << get_secret_number() << std::endl;
    return 0;
}
"#,
    )
    .unwrap();

    // 5. Build project with CUV
    let build_out = Command::new(&cuv)
        .arg("build")
        .current_dir(&test_dir)
        .output()
        .expect("cuv build with precompiled library");
    assert!(
        build_out.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_out.stderr)
    );

    // 6. Run the compiled binary and verify execution
    let proj_name = test_dir.file_name().unwrap().to_str().unwrap();
    let bin_path = test_dir.join("target").join("debug").join(proj_name);
    assert!(bin_path.exists());

    let run_res = Command::new(&bin_path).output().expect("run binary");
    assert!(run_res.status.success());
    let stdout = String::from_utf8_lossy(&run_res.stdout);
    assert!(stdout.contains("Secret is: 4242"), "Expected output to contain secret: {}", stdout);

    let _ = fs::remove_dir_all(&test_dir);
    let _ = fs::remove_dir_all(&lib_build_dir);
}

#[test]
fn test_registry_binary_packages_metadata() {
    use cuv::package::registry::{get_known_registry, normalize_target_triple, PackageKind};

    let reg = get_known_registry();
    assert!(reg.contains_key("sqlite3"));
    assert!(reg.contains_key("zlib"));
    assert!(reg.contains_key("raylib"));

    let sqlite = reg.get("sqlite3").unwrap();
    assert_eq!(sqlite.kind, PackageKind::StaticLibrary { lib_name: "sqlite3" });
    assert!(!sqlite.artifacts.is_empty());

    let triple = normalize_target_triple("arm64-apple-darwin23.0.0");
    assert_eq!(triple, "aarch64-apple-darwin");

    let linux_triple = normalize_target_triple("x86_64-unknown-linux-gnu");
    assert_eq!(linux_triple, "x86_64-unknown-linux-gnu");
}
