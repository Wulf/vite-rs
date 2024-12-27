#[derive(vite_rs::Embed)]
#[root = "."]
struct Assets;

fn main() {
    #[cfg(not(debug_assertions))]
    {
        panic!("This binary should not be run in release mode.");
    }

    #[cfg(debug_assertions)]
    {
        #[cfg(all(
            not(feature = "builtin-ctrl-c-handler"),
            not(feature = "custom-ctrl-c-handler")
        ))]
        {
            panic!("You need to enable one of the 'builtin-ctrl-c-handler' or 'custom-ctrl-c-handler' features to run this test.");
        }

        ///
        /// Feature: Built-in ctrl-c handling
        ///

        #[cfg(feature = "builtin-ctrl-c-handler")]
        let _guard = Assets::start_dev_server(true);

        ///
        /// Feature: Custom ctrl-c handling
        ///

        #[cfg(feature = "custom-ctrl-c-handler")]
        let _guard = Assets::start_dev_server();

        #[cfg(feature = "custom-ctrl-c-handler")]
        ctrlc::set_handler(move || {
            println!("Custom handler called!");

            Assets::stop_dev_server();

            std::process::exit(0);
        })
        .expect("Could not set ctrl-c handler.");

        std::thread::sleep(std::time::Duration::from_secs(12));
    }
}
