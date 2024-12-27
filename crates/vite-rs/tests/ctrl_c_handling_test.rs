/// Note: we only have a single #[test] because we can't run multiple tests in parallel
/// since the vite dev server can't be started multiple times.
#[test]
fn test() {
    const CTRL_HANDLER_FEATURE_MATRIX: [&str; 2] =
        ["builtin-ctrl-c-handler", "custom-ctrl-c-handler"];

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    {
        #[cfg(unix)]
        {
            const UNIX_SIGNAL_MATRIX: [nix::sys::signal::Signal; 3] = [
                nix::sys::signal::Signal::SIGINT,
                nix::sys::signal::Signal::SIGTERM,
                nix::sys::signal::Signal::SIGHUP,
            ];

            for feature in CTRL_HANDLER_FEATURE_MATRIX.into_iter() {
                for signal in UNIX_SIGNAL_MATRIX.into_iter() {
                    println!(
                        "Running test for feature: {} and signal: {:?}",
                        feature, signal
                    );
                    dev_tests::unix_ensure_dev_server_exits_on_signal(feature, signal);
                }
            }
        }
    }
}

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
mod dev_tests {
    #[cfg(unix)]
    pub fn unix_ensure_dev_server_exits_on_signal(feature: &str, signal: nix::sys::signal::Signal) {
        assert_dev_server_is_not_running();

        let child = run(feature);

        assert_dev_server_is_running();

        send_term_signal(child, signal);

        assert_dev_server_is_not_running();
    }

    #[cfg(unix)]
    fn send_term_signal(child: std::process::Child, signal: nix::sys::signal::Signal) {
        use nix::sys::signal;
        use nix::unistd::Pid;

        let pid = Pid::from_raw(child.id() as i32);

        signal::kill(pid, signal).expect("Failed to send SIGTERM signal.");
    }

    fn assert_dev_server_is_not_running() {
        use reqwest::blocking::Client;

        let client = Client::new();
        // Since it's possible that port 21012 is taken already and the dev server
        // starts on a new port, this test could fail.
        //
        // But for the sake of this test, we assume the first port is available.
        let response = client.get("http://localhost:21012").send();

        assert!(response.is_err());
        assert!(response.unwrap_err().is_request());
    }

    fn assert_dev_server_is_running() {
        let asset = "file.txt";
        let asset_content = "some asset 123";

        // we wait 2 seconds to make sure the dev server has time to start
        std::thread::sleep(std::time::Duration::from_secs(2));

        use reqwest::blocking::Client;

        let client = Client::new();
        // Since it's possible that port 21012 is taken already and the dev server
        // starts on a new port, this test could fail.
        //
        // But for the sake of this test, we assume the first port is available.
        let response = client
            .get(format!("http://localhost:21012/{asset}"))
            .send()
            .expect("Failed to send request.");

        assert!(response.status().is_success());
        assert_eq!(
            response.text().expect("Failed to get response text."),
            asset_content
        );
    }

    fn test_project_path() -> std::path::PathBuf {
        let workspace_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("Could not determine workspace directory.");

        // for some reason, the current directory is not the root workspace directory, but instead
        // the `crates/vite-rs` directory when running the tests.
        //
        // let's make sure this comment is correct by doing this assertion:
        assert!(workspace_dir.ends_with("crates/vite-rs"));

        let test_project_path =
            std::path::PathBuf::from_iter(&[&workspace_dir, "test_projects/ctrl_c_handling_test"]);

        test_project_path
    }

    fn run(features: &str) -> std::process::Child {
        assert!(std::process::Command::new("cargo")
            .arg("build")
            .arg("--features")
            .arg(features)
            .current_dir(test_project_path())
            .status()
            .expect("Failed to compile test project.")
            .success());

        std::process::Command::new("cargo")
            .arg("run")
            .arg("--features")
            .arg(features)
            .current_dir(test_project_path())
            .spawn()
            .expect("Failed to run test project.")
    }
}
