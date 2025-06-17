use vite_rs::Embed;

#[derive(Embed)]
#[root = "./examples/vite-project-folder"]
struct Assets;

fn main() {
    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    // We use an RAII guard to gracefully exit the dev server
    let _guard = Assets::start_dev_server(false);

    ctrlc::try_set_handler(|| {
        #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
        Assets::stop_dev_server();
        std::process::exit(0);
    })
    .unwrap();

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    {
        println!("Waiting for the dev server to start (2 second)");
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    let file = Assets::get("app/index.html").unwrap();
    let file_content = std::str::from_utf8(&file.bytes).unwrap();

    println!("Reading index.html:");
    println!("{}", file_content);

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    assert_eq!(
        strip_space(file_content),
        strip_space(
            r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                    <script type="module">
                import RefreshRuntime from "/@react-refresh"
                RefreshRuntime.injectIntoGlobalHook(window)
                window.$RefreshReg$ = () => {}
                window.$RefreshSig$ = () => (type) => type
                window.__vite_plugin_react_preamble_installed__ = true
                </script>

                    <script type="module" src="/@vite/client"></script>

                    <meta charset="UTF-8" />
                    <link rel="icon" type="image/svg+xml" href="./vite.svg" />
                    <link rel="stylsheet" type="text/css" href="./index.css" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <title>vite-rs</title>
                </head>
                <body>
                    <div id="root"></div>
                    <script type="module" src="./index.ts"></script>
                </body>
                </html>"#
        )
    );

    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    assert_eq!(
        strip_space(file_content),
        strip_space(
            "<!DOCTYPEhtml>\n<htmllang=\"en\">\n<head>\n<metacharset=\"UTF-8\"/>\n<linkrel=\"icon\"type=\"image/svg+xml\"href=\"/assets/vite-DcBtz0py.svg\"/>\n<metaname=\"viewport\"content=\"width=device-width,initial-scale=1.0\"/>\n<title>vite-rs</title>\n<scripttype=\"module\"crossoriginsrc=\"/assets/index-BZiJcslM.js\"></script>\n<linkrel=\"stylesheet\"crossoriginhref=\"/assets/index-BPvgi06w.css\">\n</head>\n<body>\n<divid=\"root\"></div>\n</body>\n</html>"
        )
    );
}

fn strip_space(s: &str) -> String {
    s.trim().replace(" ", "")
}
