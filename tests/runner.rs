use std::{env::consts::EXE_SUFFIX, fs, process::Command};

#[test]
fn running_a_tool_that_is_not_installed_names_it() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let project = dir.path().join("project");
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("rokit.toml"),
        "[tools]\nrojo = \"rojo-rbx/rojo@7.4.4\"\n",
    )
    .unwrap();

    // Rokit runs a tool when its executable is named after that tool
    let link = root.join("bin").join(format!("rojo{EXE_SUFFIX}"));
    fs::copy(env!("CARGO_BIN_EXE_rokit"), &link).unwrap();

    let output = Command::new(&link)
        .arg("--version")
        .current_dir(&project)
        .env("ROKIT_ROOT", &root)
        .env_remove("RUST_LOG")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("Tool 'rojo' (rojo-rbx/rojo@7.4.4) is not installed."),
        "unexpected output: {stderr}"
    );
}
