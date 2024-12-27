#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
use command_group::GroupChild;

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
#[cfg(feature = "ctrlc")]
pub use ctrlc;

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
pub use reqwest; // exported for use in derived code

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
use std::sync::{Arc, Mutex};

pub mod util;

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
pub struct ViteProcess(pub Arc<Mutex<GroupChild>>);

#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
pub struct ViteProcess;

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
lazy_static::lazy_static! {
    static ref VITE_PROCESS: Arc<Mutex<Option<ViteProcess>>> = Arc::new(Mutex::new(None));
}

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
fn set_dev_server(process: ViteProcess) {
    let original = VITE_PROCESS.lock().unwrap().replace(process);
    if original.is_some() {
        original
            .unwrap()
            .0
            .lock()
            .expect("(!) Could not shutdown ViteJS dev server: Mutex poisoned")
            .kill()
            .expect("(!) Could not shutdown ViteJS dev server.");
    }
}

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
fn unset_dev_server() {
    let process = VITE_PROCESS.lock().unwrap().take();
    if process.is_some() {
        process
            .unwrap()
            .0
            .lock()
            .expect("(!) Could not shutdown ViteJS dev server: Mutex poisoned")
            .kill()
            .expect("(!) Could not shutdown ViteJS dev server.");
    }
}

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
impl Drop for ViteProcess {
    fn drop(&mut self) {
        unset_dev_server();
    }
}

/// Starts the ViteJS dev server.
///
/// Example 1 (with the included `ctrlc` feature enabled):
///
/// ```ignore
/// fn main() {
///     #[cfg(debug_assertions)]
///     let _guard = Assets::start_dev_server(true);
///
///    // ...
/// }
/// ```
///
/// Example 2 (using the `ctrlc` crate to handle Ctrl-C):
///
/// ```ignore
/// fn main() {
///   #[cfg(debug_assertions)]
///   let _guard = Assets::start_dev_server();
///
///   ctrlc::try_set_handler(|| {
///     #[cfg(debug_assertions)]
///     Assets::stop_dev_server();
///     std::process::exit(0);
///  }).unwrap();
/// }
///
/// ```
#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
pub fn start_dev_server(
    absolute_root_dir: &str,
    host: &str,
    port: u16,
    #[cfg(feature = "ctrlc")] register_ctrl_c_handler: bool,
) -> Option<ViteProcess> {
    use command_group::CommandGroup;

    if !util::is_port_free(port as u16) {
        panic!(
            "Selected vite-rs dev server port '{}' is not available.\na) If self-selecting a port via #[dev_server_port = XXX], ensure it is free.\nb) Otherwise, remove the #[dev_server_port] attribute and let vite-rs select a free port for you at compile time.",
            port
        )
    }

    // println!("Starting dev server!");
    // start ViteJS dev server
    let child = Arc::new(Mutex::new(
        std::process::Command::new("npx")
            .arg("vite")
            .arg("--host")
            .arg(host)
            .arg("--port")
            .arg(port.to_string())
            .arg("--strictPort")
            .arg("--clearScreen")
            .arg("false")
            // we don't want to send stdin to the dev server; this also
            // hides the "press h + enter to show help" message that the dev server prints
            .stdin(std::process::Stdio::null())
            .current_dir(
                absolute_root_dir, /*format!(
                                       "{}/examples/basic_usage",
                                       std::env::var("CARGO_MANIFEST_DIR").unwrap()
                                   )*/
            )
            .group_spawn()
            .expect("failed to start ViteJS dev server"),
    ));
    set_dev_server(ViteProcess(child.clone()));

    #[cfg(feature = "ctrlc")]
    {
        if register_ctrl_c_handler {
            // We handle Ctrl-C because the node process does not exit properly otherwise
            ctrlc::try_set_handler({
                move || {
                    unset_dev_server();
                    std::process::exit(0);
                }
            })
            .expect("vite-rs: Error setting Ctrl-C handler; if you are using a custom one, disable the ctrlc feature for the vite-rs crate, and follow the documentation here to integrate it: https://github.com/Wulf/vite-rs#ctrl-c-handler");
        }
    }

    // We build an RAII guard around the child process so that the dev server is killed when it's dropped
    Some(ViteProcess(child.clone()))
}

#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
pub fn start_dev_server(
    #[cfg(feature = "ctrlc")] _register_ctrl_c_handler: bool,
) -> Option<ViteProcess> {
    None
}

#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
pub fn stop_dev_server() {
    unset_dev_server();
}

#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
pub fn stop_dev_server() {
    // do nothing
}
