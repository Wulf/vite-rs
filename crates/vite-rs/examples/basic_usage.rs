use vite_rs::Embed;

#[derive(Embed)]
#[root = "./examples/vite-project-folder"]
struct Assets;

fn main() {
    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    // We use an RAII guard to gracefully exit the dev server
    let _guard = Assets::start_dev_server(true);

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    {
        // only needed for this example
        println!("Waiting for the dev server to start (2 second)");
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    {
        Assets::iter().for_each(|file_name| {
            println!("ENTRY: {}", file_name);
        });
    }

    println!("Reading index.html:");
    let file = Assets::get("app/index.html").unwrap();
    let file_content = std::str::from_utf8(&file.bytes).unwrap();

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

    println!("Reading pack1.js:");
    let file = Assets::get("app/pack1.ts").unwrap();
    let file_content = std::str::from_utf8(&file.bytes).unwrap();

    println!("{}", file_content);

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    assert_eq!(
        strip_space(file_content),
        strip_space(
            r#"
            const test = (() => {
                console.log("This is a test");
                const a = 3;
                return a;
            })();
            const num = test;
            console.log("NUM: ", num);

            //# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbInBhY2sxLnRzIl0sInNvdXJjZXNDb250ZW50IjpbImNvbnN0IHRlc3QgPSAoKCkgPT4ge1xuICBjb25zb2xlLmxvZygnVGhpcyBpcyBhIHRlc3QnKVxuXG4gIGNvbnN0IGE6IG51bWJlciA9IDNcblxuICByZXR1cm4gYVxufSkoKVxuXG5jb25zdCBudW0gPSB0ZXN0XG5cbmNvbnNvbGUubG9nKCdOVU06ICcsIG51bSlcbiJdLCJtYXBwaW5ncyI6IkFBQUEsTUFBTSxRQUFRLE1BQU07QUFDbEIsVUFBUSxJQUFJLGdCQUFnQjtBQUU1QixRQUFNLElBQVk7QUFFbEIsU0FBTztBQUNULEdBQUc7QUFFSCxNQUFNLE1BQU07QUFFWixRQUFRLElBQUksU0FBUyxHQUFHOyIsIm5hbWVzIjpbXX0=
        "#
        )
    );

    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    assert_eq!(
        strip_space(file_content),
        strip_space(r#"const o=(console.log("This is a test"),3),s=o;console.log("NUM: ",s);"#)
    );
}

fn strip_space(s: &str) -> String {
    s.trim().replace(" ", "")
}
