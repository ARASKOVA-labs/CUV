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
        "cuv_test_cloud_{}_{}",
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
fn test_cloud_login_whoami_logout_lifecycle() {
    let config_dir = create_temp_dir("auth_lifecycle");
    let cuv = get_cuv_bin();

    let whoami_1 = Command::new(&cuv)
        .arg("whoami")
        .env("CUV_CONFIG_DIR", &config_dir)
        .output()
        .expect("cuv whoami before login");
    assert!(whoami_1.status.success());
    let stdout1 = String::from_utf8_lossy(&whoami_1.stdout);
    assert!(stdout1.contains("Not logged in"));

    let login_out = Command::new(&cuv)
        .arg("login")
        .arg("--token")
        .arg("cuv_test_live_secret_token_123456")
        .arg("--org")
        .arg("Araskova-Engineering")
        .env("CUV_CONFIG_DIR", &config_dir)
        .output()
        .expect("cuv login");
    assert!(login_out.status.success());
    let login_str = String::from_utf8_lossy(&login_out.stdout);
    assert!(login_str.contains("Authenticated with CUV Cloud"));
    assert!(login_str.contains("Araskova-Engineering"));

    assert!(config_dir.join("credentials.toml").exists());

    let whoami_2 = Command::new(&cuv)
        .arg("whoami")
        .env("CUV_CONFIG_DIR", &config_dir)
        .output()
        .expect("cuv whoami after login");
    assert!(whoami_2.status.success());
    let stdout2 = String::from_utf8_lossy(&whoami_2.stdout);
    assert!(stdout2.contains("Authenticated"));
    assert!(stdout2.contains("Araskova-Engineering"));

    let logout_out = Command::new(&cuv)
        .arg("logout")
        .env("CUV_CONFIG_DIR", &config_dir)
        .output()
        .expect("cuv logout");
    assert!(logout_out.status.success());
    let logout_str = String::from_utf8_lossy(&logout_out.stdout);
    assert!(logout_str.contains("Logged out"));

    let whoami_3 = Command::new(&cuv)
        .arg("whoami")
        .env("CUV_CONFIG_DIR", &config_dir)
        .output()
        .expect("cuv whoami after logout");
    let stdout3 = String::from_utf8_lossy(&whoami_3.stdout);
    assert!(stdout3.contains("Not logged in"));

    let _ = fs::remove_dir_all(&config_dir);
}

#[test]
fn test_cloud_status_cli_command() {
    let cuv = get_cuv_bin();
    let out = Command::new(&cuv)
        .arg("cloud")
        .arg("status")
        .output()
        .expect("cuv cloud status");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("CUV Cloud Remote Cache Status"));
    assert!(stdout.contains("Endpoint:"));
    assert!(stdout.contains("Latency:"));
}
