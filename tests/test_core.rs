#[test]
fn test_manifest_serialization() {
    let toml_str = r#"
[project]
name = "test_app"
version = "1.0.0"
standard = "c++20"
kind = "executable"

[dependencies]
fmt = "10.2.1"
"#;
    let val: toml::Value = toml::from_str(toml_str).expect("Valid toml");
    assert_eq!(val["project"]["name"].as_str().unwrap(), "test_app");
    assert_eq!(val["project"]["standard"].as_str().unwrap(), "c++20");
    assert_eq!(val["dependencies"]["fmt"].as_str().unwrap(), "10.2.1");
}

#[test]
fn test_abi_hashing_consistency() {
    use sha2::{Digest, Sha256};

    let mut h1 = Sha256::new();
    h1.update(b"fmt:10.2.1:clang-18:c++20:arm64-apple-darwin:O3");
    let res1 = format!("{:x}", h1.finalize())[..16].to_string();

    let mut h2 = Sha256::new();
    h2.update(b"fmt:10.2.1:clang-18:c++20:arm64-apple-darwin:O3");
    let res2 = format!("{:x}", h2.finalize())[..16].to_string();

    assert_eq!(res1, res2);
}

#[test]
fn test_depfile_parsing_cross_platform() {
    use cuv::compiler::depfile::parse_depfile_content;
    use std::path::PathBuf;

    let unix_content = "target/debug/obj/main.o: src/main.cpp include/foo.hpp";
    let deps = parse_depfile_content(unix_content);
    assert_eq!(
        deps,
        vec![
            PathBuf::from("src/main.cpp"),
            PathBuf::from("include/foo.hpp")
        ]
    );

    let windows_content = r#"C:\Users\runner\target\debug\obj\main.o: \
  C:\Users\runner\src\main.cpp \
  C:\Users\runner\include\foo.hpp"#;
    let win_deps = parse_depfile_content(windows_content);
    assert_eq!(
        win_deps,
        vec![
            PathBuf::from(r#"C:\Users\runner\src\main.cpp"#),
            PathBuf::from(r#"C:\Users\runner\include\foo.hpp"#),
        ]
    );

    let win_slash_content = "D:/a/CUV/target/debug/obj/main.o: D:/a/CUV/src/main.cpp";
    let win_slash_deps = parse_depfile_content(win_slash_content);
    assert_eq!(win_slash_deps, vec![PathBuf::from("D:/a/CUV/src/main.cpp")]);

    let space_content = r#"target/obj.o: C:\My\ Projects\main.cpp"#;
    let space_deps = parse_depfile_content(space_content);
    assert_eq!(
        space_deps,
        vec![PathBuf::from(r#"C:\My Projects\main.cpp"#)]
    );
}
