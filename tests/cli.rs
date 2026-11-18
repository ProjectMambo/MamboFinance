use std::process::Command;

fn command(data_home: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mambofinance"));
    command.env_remove("MAMBOFINANCE_MCP_TOKEN");
    #[cfg(target_os = "windows")]
    command.env("LOCALAPPDATA", data_home);
    #[cfg(target_os = "macos")]
    command.env("HOME", data_home);
    #[cfg(all(unix, not(target_os = "macos")))]
    command.env("XDG_DATA_HOME", data_home);
    command
}

#[test]
fn invalid_mode_fails_before_creating_application_data() {
    let directory = tempfile::tempdir().expect("temporary directory");

    let output = command(directory.path())
        .arg("unknown")
        .output()
        .expect("mambofinance process");

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8 stderr"),
        "mambofinance: usage: mambofinance [mcp]\n"
    );
    assert!(
        directory
            .path()
            .read_dir()
            .expect("data directory")
            .next()
            .is_none()
    );
}

#[test]
fn mcp_mode_without_a_token_fails_before_binding() {
    let directory = tempfile::tempdir().expect("temporary directory");

    let output = command(directory.path())
        .arg("mcp")
        .output()
        .expect("mambofinance process");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");

    assert!(!output.status.success());
    assert!(stderr.contains("set MAMBOFINANCE_MCP_TOKEN"));
    assert!(!stderr.contains("MCP listening"));
}
