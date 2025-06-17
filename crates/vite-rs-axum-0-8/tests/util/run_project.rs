fn test_project_path(test_project_name: &str) -> std::path::PathBuf {
    let workspace_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Could not determine workspace directory.");

    // for some reason, the current directory is not the root workspace directory, but instead
    // the `crates/vite-rs-axum-0-8` directory when running the tests.
    //
    // let's make sure this comment is correct by doing this assertion:
    assert!(workspace_dir.ends_with("crates/vite-rs-axum-0-8"));

    let test_project_path =
        std::path::PathBuf::from_iter(&[&workspace_dir, "test_projects", test_project_name]);

    test_project_path
}

pub fn run(
    test_project_name: &str,
    release_build: bool,
    feature_flags: Vec<&str>,
) -> std::process::Child {
    assert!(std::process::Command::new("cargo")
        .arg("build")
        .args(if release_build {
            vec!["--release"]
        } else {
            vec![]
        })
        .args(feature_flags)
        .current_dir(test_project_path(test_project_name))
        .status()
        .expect("Failed to compile test project.")
        .success());

    std::process::Command::new("cargo")
        .arg("run")
        .current_dir(test_project_path(test_project_name))
        .spawn()
        .expect("Failed to run test project.")
}

#[cfg(unix)]
pub fn stop(mut child: std::process::Child) {
    assert!(child.kill().is_ok());
    assert!(child.wait().is_ok());
}
