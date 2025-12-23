#[derive(vite_rs::Embed)]
#[root = "./test_projects/normal_usage_test"]
struct Assets;

/// Note: we only have a single #[test] because we can't run multiple tests in parallel
/// since the vite dev server can't be started multiple times.
#[test]
pub fn test() {
    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    let _guard = Assets::start_dev_server(true);

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    assert!(_guard.is_some());

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    std::thread::sleep(std::time::Duration::from_secs(2));

    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    {
        ensure_asset_list();
        ensure_aliases();
        ensure_no_dot_vite_dir();
    }
    ensure_html_entrypoint();
    ensure_ts_entrypoint();
    ensure_public_dir_files();
    ensure_no_vite_manifest();
    ensure_content_hash_is_correct();
}

#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
fn ensure_asset_list() {
    use std::iter::zip;

    let mut list = Assets::iter().collect::<Vec<_>>();
    list.sort();

    let mut expected = vec![
        "assets/vite-DcBtz0py.svg",
        "assets/index-BZiJcslM.js",
        "assets/pack1-B2m_tRuS.js",
        "assets/index-BPvgi06w.css",
        // ".vite/manifest.json",
        "test.txt",
        "app/index.html",
    ];
    expected.sort();

    assert_eq!(list.len(), expected.len());

    for (file1, file2) in zip(list, expected) {
        assert_eq!(file1, file2);
        assert!(Assets::get(&file1).is_some());
    }
}

#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
fn ensure_aliases() {
    let aliases = vec![("app/pack1.ts", "assets/pack1-B2m_tRuS.js")];

    for alias in aliases {
        assert_eq!(
            Assets::get(alias.0).unwrap().bytes,
            Assets::get(alias.1).unwrap().bytes
        )
    }
}

#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
fn ensure_no_dot_vite_dir() {
    for file in Assets::iter() {
        assert!(!file.starts_with(".vite/"));
    }
}

fn ensure_html_entrypoint() {
    let file = Assets::get("app/index.html").unwrap();

    assert_eq!(file.content_type, "text/html");
    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    if cfg!(windows) {
        assert_eq!(file.content_length, 489);
    } else {
        assert_eq!(file.content_length, 475);
    }
    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    if cfg!(windows) {
        assert_eq!(file.content_length, 484);
    } else {
        assert_eq!(file.content_length, 470);
    }

    let content = std::str::from_utf8(&file.bytes).unwrap();

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    if cfg!(windows) {
        assert_eq!(
            content.replace(" ", ""),
            "<!DOCTYPEhtml>\r\n<htmllang=\"en\">\r\n<head>\n<scripttype=\"module\"src=\"/@vite/client\"></script>\n\r\n<metacharset=\"UTF-8\"/>\r\n<linkrel=\"icon\"type=\"image/svg+xml\"href=\"./vite.svg\"/>\r\n<linkrel=\"stylsheet\"type=\"text/css\"href=\"./index.css\"/>\r\n<metaname=\"viewport\"content=\"width=device-width,initial-scale=1.0\"/>\r\n<title>vite-rs</title>\r\n</head>\r\n<body>\r\n<divid=\"root\"></div>\r\n<scripttype=\"module\"src=\"./index.ts\"></script>\r\n</body>\r\n</html>\r\n"
            .replace(" ", "")
        );
    } else {
        assert_eq!(
            content.replace(" ", ""),
            "<!DOCTYPEhtml>\n<htmllang=\"en\">\n<head>\n<scripttype=\"module\"src=\"/@vite/client\"></script>\n\n<metacharset=\"UTF-8\"/>\n<linkrel=\"icon\"type=\"image/svg+xml\"href=\"./vite.svg\"/>\n<linkrel=\"stylsheet\"type=\"text/css\"href=\"./index.css\"/>\n<metaname=\"viewport\"content=\"width=device-width,initial-scale=1.0\"/>\n<title>vite-rs</title>\n</head>\n<body>\n<divid=\"root\"></div>\n<scripttype=\"module\"src=\"./index.ts\"></script>\n</body>\n</html>\n"
            .replace(" ", "")
        );
    }

    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    if cfg!(windows) {
        assert_eq!(
            content.replace(" ", ""),
            "<!DOCTYPEhtml>\r\n<htmllang=\"en\">\r\n<head>\r\n<metacharset=\"UTF-8\"/>\r\n<linkrel=\"icon\"type=\"image/svg+xml\"href=\"/assets/vite-DcBtz0py.svg\"/>\r\r\n<metaname=\"viewport\"content=\"width=device-width,initial-scale=1.0\"/>\r\n<title>vite-rs</title>\r\n<scripttype=\"module\"crossoriginsrc=\"/assets/index-BZiJcslM.js\"></script>\n<linkrel=\"stylesheet\"crossoriginhref=\"/assets/index-BPvgi06w.css\">\n</head>\r\n<body>\r\n<divid=\"root\"></div>\r\r\n</body>\r\n</html>\r\n"
            .replace(" ", "")
        );
    } else {
        assert_eq!(
            content.replace(" ", ""),
            "<!DOCTYPEhtml>\n<htmllang=\"en\">\n<head>\n<metacharset=\"UTF-8\"/>\n<linkrel=\"icon\"type=\"image/svg+xml\"href=\"/assets/vite-DcBtz0py.svg\"/>\n<metaname=\"viewport\"content=\"width=device-width,initial-scale=1.0\"/>\n<title>vite-rs</title>\n<scripttype=\"module\"crossoriginsrc=\"/assets/index-BZiJcslM.js\"></script>\n<linkrel=\"stylesheet\"crossoriginhref=\"/assets/index-BPvgi06w.css\">\n</head>\n<body>\n<divid=\"root\"></div>\n</body>\n</html>\n"
            .replace(" ", "")
        );
    }
}

fn ensure_ts_entrypoint() {
    let file = Assets::get("app/pack1.ts").unwrap();
    let content = std::str::from_utf8(&file.bytes).unwrap();

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    {
        if cfg!(windows) {
            assert_eq!(file.content_type, "text/javascript");
            assert_eq!(file.content_length, 684);
            assert_eq!(
                content.replace(" ", ""),
                "consttest=(()=>{\nconsole.log(\"Thisisatest\");\nconsta=3;\nreturna;\n})();\nconstnum=test;\nconsole.log(\"NUM:\",num);\n\n//#sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbInBhY2sxLnRzIl0sInNvdXJjZXNDb250ZW50IjpbImNvbnN0IHRlc3QgPSAoKCkgPT4ge1xyXG4gIGNvbnNvbGUubG9nKCdUaGlzIGlzIGEgdGVzdCcpXHJcblxyXG4gIGNvbnN0IGE6IG51bWJlciA9IDNcclxuXHJcbiAgcmV0dXJuIGFcclxufSkoKVxyXG5cclxuY29uc3QgbnVtID0gdGVzdFxyXG5cclxuY29uc29sZS5sb2coJ05VTTogJywgbnVtKVxyXG4iXSwibWFwcGluZ3MiOiJBQUFBLE1BQU0sUUFBUSxNQUFNO0FBQ2xCLFVBQVEsSUFBSSxnQkFBZ0I7QUFFNUIsUUFBTSxJQUFZO0FBRWxCLFNBQU87QUFDVCxHQUFHO0FBRUgsTUFBTSxNQUFNO0FBRVosUUFBUSxJQUFJLFNBQVMsR0FBRzsiLCJuYW1lcyI6W119"
            );
        } else {
            assert_eq!(file.content_type, "text/javascript");
            assert_eq!(file.content_length, 656);
            assert_eq!(
                content.replace(" ", ""),
                "consttest=(()=>{\nconsole.log(\"Thisisatest\");\nconsta=3;\nreturna;\n})();\nconstnum=test;\nconsole.log(\"NUM:\",num);\n\n//#sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbInBhY2sxLnRzIl0sInNvdXJjZXNDb250ZW50IjpbImNvbnN0IHRlc3QgPSAoKCkgPT4ge1xuICBjb25zb2xlLmxvZygnVGhpcyBpcyBhIHRlc3QnKVxuXG4gIGNvbnN0IGE6IG51bWJlciA9IDNcblxuICByZXR1cm4gYVxufSkoKVxuXG5jb25zdCBudW0gPSB0ZXN0XG5cbmNvbnNvbGUubG9nKCdOVU06ICcsIG51bSlcbiJdLCJtYXBwaW5ncyI6IkFBQUEsTUFBTSxRQUFRLE1BQU07QUFDbEIsVUFBUSxJQUFJLGdCQUFnQjtBQUU1QixRQUFNLElBQVk7QUFFbEIsU0FBTztBQUNULEdBQUc7QUFFSCxNQUFNLE1BQU07QUFFWixRQUFRLElBQUksU0FBUyxHQUFHOyIsIm5hbWVzIjpbXX0="
            );
        }
    }

    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    {
        assert_eq!(file.content_type, "application/javascript");
        assert_eq!(file.content_length, 70);
        assert_eq!(
            content.replace(" ", ""),
            "consto=(console.log(\"Thisisatest\"),3),s=o;console.log(\"NUM:\",s);\n"
                .replace(" ", "")
        );
    }
}

fn ensure_public_dir_files() {
    let file = Assets::get("test.txt").unwrap();

    assert_eq!(file.content_type, "text/plain");
    assert_eq!(file.content_length, 4);
    assert_eq!(file.bytes, "test".as_bytes());
}

fn ensure_no_vite_manifest() {
    assert!(Assets::get(".vite/manifest.json").is_none());
}

fn ensure_content_hash_is_correct() {
    let public_file = Assets::get("test.txt").unwrap();
    let input_file = Assets::get("app/index.html").unwrap();
    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    let included_file = Assets::get("app/index.ts").unwrap();
    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
    let included_file = Assets::get("assets/index-BZiJcslM.js").unwrap();

    check_hash(public_file);
    check_hash(input_file);
    check_hash(included_file);

    fn check_hash(file: vite_rs::ViteFile) {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&file.bytes);
        #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
        {
            assert!(file.content_hash.starts_with("W/\""));
            assert!(file.content_hash.ends_with("\""));
            assert!(file.content_hash.len() > 4)
        }

        #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
        {
            let content_hash = format!("{:x}", hash);

            assert_eq!(
                file.content_hash.to_lowercase(),
                content_hash.to_lowercase()
            );
        }
    }
}
