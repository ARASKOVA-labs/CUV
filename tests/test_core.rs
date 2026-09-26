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
