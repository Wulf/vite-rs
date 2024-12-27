#[derive(vite_rs::Embed)]
#[root = "./test_projects/custom_dev_server_port_test"]
#[dev_server_port = "21232"]
struct Assets;

#[derive(vite_rs::Embed)]
#[root = "./test_projects/custom_dev_server_port_test"]
#[dev_server_port = 21222] // without quotes
struct AssetsWithoutQuotes;

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
#[test]
fn test_dev_server_port() {
    // we're going to set a custom Ctrl-C handler because this test starts
    // several dev servers and vite-rs doesn't handle this case out-of-the-box
    ctrlc::set_handler(move || {
        #[cfg(debug_assertions)]
        {
            AssetsWithoutQuotes::stop_dev_server();
            Assets::stop_dev_server();
        }

        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    #[cfg(unix)]
    {
        // Rust-managed: Test that the dev server runs on the specified port
        {
            let _guard = AssetsWithoutQuotes::start_dev_server(false);

            // we wait 2 seconds to make sure the dev server has time to start
            std::thread::sleep(std::time::Duration::from_secs(2));

            match assert_dev_server_running_on_port(21222) {
                Ok(resp) => {
                    assert_eq!(resp, "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <script type=\"module\" src=\"/@vite/client\"></script>\n\n    <meta charset=\"UTF-8\" />\n    <title>Custom Dev Server Port Test</title>\n  </head>\n  <body>\n    <h1>Custom Dev Server Port Test</h1>\n    <p>It works!</p>\n  </body>\n</html>\n");
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }

        // Rust-managed: Test that the dev server runs on the specified port
        {
            assert!(assert_dev_server_running_on_port(21222).is_err()); // ensure the previous dev server has stopped

            let _guard = Assets::start_dev_server(false);

            // we wait 2 seconds to make sure the dev server has time to start
            std::thread::sleep(std::time::Duration::from_secs(2));

            match assert_dev_server_running_on_port(21232) {
                Ok(resp) => {
                    assert_eq!(resp, "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <script type=\"module\" src=\"/@vite/client\"></script>\n\n    <meta charset=\"UTF-8\" />\n    <title>Custom Dev Server Port Test</title>\n  </head>\n  <body>\n    <h1>Custom Dev Server Port Test</h1>\n    <p>It works!</p>\n  </body>\n</html>\n");
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }

        // Self-managed: Test that the dev server connects to the specified port
        {
            // should fail because the dev server would have shut down as it went out of scope
            assert!(assert_dev_server_running_on_port(21232).is_err());

            use command_group::CommandGroup; // npx spawns a group process, so we use this in order to kill it later.
            let mut cmd = std::process::Command::new("npx")
                .arg("vite")
                .arg("--port")
                .arg("21232")
                .arg("--strictPort")
                .arg("--clearScreen")
                .arg("false")
                .stdin(std::process::Stdio::null())
                .current_dir("./test_projects/custom_dev_server_port_test")
                .group_spawn() // npx spawns a group process, so we use a group_spawn in order to kill it later.
                .expect("Failed to start dev server");

            let mut close_dev_server = || {
                cmd.kill()
                    .expect("Failed to stop the ViteJS dev server for this test");
            };

            // we wait 2 seconds to make sure the dev server has time to start
            std::thread::sleep(std::time::Duration::from_secs(2));

            match assert_dev_server_running_on_port(21232) {
                Ok(resp) => {
                    close_dev_server();
                    assert_eq!(resp, "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <script type=\"module\" src=\"/@vite/client\"></script>\n\n    <meta charset=\"UTF-8\" />\n    <title>Custom Dev Server Port Test</title>\n  </head>\n  <body>\n    <h1>Custom Dev Server Port Test</h1>\n    <p>It works!</p>\n  </body>\n</html>\n");
                }
                Err(e) => {
                    close_dev_server();
                    panic!("{}", e);
                }
            }
        }
    }
}

fn assert_dev_server_running_on_port(port: u16) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("http://localhost:{}", port);

    match client.get(url).send() {
        Ok(res) => {
            if res.status() != 200 {
                return Err("Expected 200 status code".to_string());
            } else {
                return Ok(std::str::from_utf8(&res.bytes().unwrap())
                    .unwrap()
                    .to_string());
            }
        }
        Err(e) => {
            return Err(
                format!("Failed to connect to dev server on port {}: {}", port, e).to_string(),
            );
        }
    };
}
