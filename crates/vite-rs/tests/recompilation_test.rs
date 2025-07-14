/// NOTE: Breaking change: we changed the `last_modified` field to be of type String instead of
/// u64 to match the HTTP standard for Last-Modified headers. This test needs to be updated before it is re-enabled.

/// Note: we only have a single #[test] because we can't run multiple tests in parallel
/// since the vite dev server can't be started multiple times.
#[test]
fn test() {
    // These recompilation tests are meant for release builds only
    // They're here to ensure the assets that are embedded are properly watched for changes
    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    {
        release_tests::compile_test_project();
        release_tests::ensure_binary_recompiles_on_asset_change();
        release_tests::ensure_binary_does_not_recompile_on_other_changes();
    }
}

#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
mod release_tests {
    use std::path::PathBuf;

    pub fn ensure_binary_recompiles_on_asset_change() {
        delete_asset_if_exists(&format!("app{}test2.txt", std::path::MAIN_SEPARATOR));
        compile_test_project();
        ensure_assets_exist(vec![/* "app/test.txt" -> */ &format!(
            "assets{}test-BPR99Ku7.txt",
            std::path::MAIN_SEPARATOR
        )]);
        let binary_last_modified = get_compiled_binary_modified_time();

        add_asset(&format!("app{}test2.txt", std::path::MAIN_SEPARATOR), "123");
        compile_test_project();
        ensure_assets_exist(vec![
            /* "app/test2.txt" -> */
            &format!("assets{}test2-CajEw_O3.txt", std::path::MAIN_SEPARATOR),
            /* "app/test.txt" -> */
            &format!("assets{}test-BPR99Ku7.txt", std::path::MAIN_SEPARATOR),
        ]);
        let binary_last_modified_2 = get_compiled_binary_modified_time();

        delete_asset_if_exists(&format!("app{}test2.txt", std::path::MAIN_SEPARATOR)); // cleanup

        assert!(
            binary_last_modified_2 - binary_last_modified > 0,
            "Binary was not recompiled on asset change"
        );
    }

    /// Note: this test is disabled because, currently, ViteJS recomplies assets
    ///       even if there is no change. Effectively, this results in a full rebuild
    ///       of the rust binary.
    ///
    ///       Full example:
    ///       1. We run `cargo build --release`
    ///       2. Internally, `vite build` is run, generating compiled assets
    ///       3. Assets are included using `include_bytes!` which causes the rust compiler
    ///          to track these files for changes.
    ///       4. If we run `cargo build --release` again, the rust compiler notices that
    ///          the assets have changed, triggering steps 2 and 3 again.
    ///
    ///       Options moving forward:
    ///       1. Wait for ViteJS to implement a feature that prevents recompilation when
    ///          there are no changes.
    ///       2. Use `std::fs::read` instead of `include_bytes!` to load assets at runtime.
    ///          This will prevent the rust compiler from tracking the assets for changes.
    pub fn ensure_binary_does_not_recompile_on_other_changes() {
        // delete_asset_if_exists("app/test2.txt");

        // compile_test_project();
        // ensure_assets_exist(vec![
        //     /* "app/test.txt" -> */ "assets/test-BPR99Ku7.txt",
        // ]);
        // let binary_last_modified = get_compiled_binary_modified_time();

        // add_asset("test.txt", "// some unrelated change");
        // compile_test_project();
        // ensure_assets_exist(vec![
        //     /* "app/test.txt" -> */ "assets/test-BPR99Ku7.txt",
        // ]);
        // let binary_last_modified_2 = get_compiled_binary_modified_time();

        // delete_asset_if_exists("test.txt"); // cleanup

        // assert_eq!(
        //     binary_last_modified_2, binary_last_modified,
        //     "Binary was recompiled on asset change"
        // );
    }

    fn ensure_assets_exist(assets: Vec<&str>) {
        // sort the compiled assets
        let mut compiled_assets: Vec<String> = get_binary_asset_list();
        compiled_assets.sort();

        // sort the expected assets
        let mut assets = assets.clone();
        assets.sort();

        assert_eq!(assets, compiled_assets);
    }

    #[allow(dead_code)]
    fn add_asset(asset: &str, content: &str) {
        let asset_path = test_project_path().join(asset);

        std::fs::write(&asset_path, content)
            .expect(&format!("Failed to write to the asset file: {asset}"));
    }

    fn delete_asset_if_exists(asset: &str) {
        let asset_path = test_project_path().join(asset);

        if asset_path.exists() {
            std::fs::remove_file(asset_path)
                .expect(&format!("Failed to delete the asset file: {asset}"));
        }
    }

    fn test_project_path() -> PathBuf {
        let workspace_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("Could not determine workspace directory.");

        // for some reason, the current directory is not the root workspace directory, but instead
        // the `crates/vite-rs` directory when running the tests.
        //
        // let's make sure this comment is correct by doing this assertion:
        assert!(workspace_dir.ends_with(&format!("crates{}vite-rs", std::path::MAIN_SEPARATOR)));

        let test_project_path = PathBuf::from_iter(&[
            &workspace_dir,
            &format!(
                "test_projects{}recompilation_test",
                std::path::MAIN_SEPARATOR
            ),
        ]);

        test_project_path
    }

    pub fn compile_test_project() {
        // Compile the project
        let has_compiled = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(test_project_path())
            .status()
            .expect("Failed to compile the project")
            .success();

        assert!(has_compiled);
    }

    fn get_binary_asset_list() -> Vec<String> {
        let executable_name = if cfg!(windows) {
            "recompilation_test.exe"
        } else {
            "recompilation_test"
        };
        let base_dir = test_project_path().join("target/release");
        let executable_path = base_dir.join(executable_name);

        let output = std::process::Command::new(executable_path)
            .current_dir(base_dir)
            .output()
            .expect("Failed to get the list of assets")
            .stdout;

        String::from_utf8(output)
            .expect("Failed to convert the output to a string")
            .lines()
            .map(|line| line.to_string())
            .collect()
    }

    fn get_compiled_binary_modified_time() -> u128 {
        let executable_name = if cfg!(windows) {
            "recompilation_test.exe"
        } else {
            "recompilation_test"
        };
        let binary_path = test_project_path()
            .join("target/release")
            .join(executable_name);

        let modified_time = binary_path
            .metadata()
            .expect("Failed to get metadata for the compiled binary")
            .modified()
            .expect("Failed to get modified time of the compiled binary");

        modified_time
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to convert the modified time to a SystemTime")
            .as_nanos()
    }
}
