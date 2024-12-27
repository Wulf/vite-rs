#[derive(vite_rs::Embed)]
#[root = "./tests/renaming_bundles"]
struct Assets;

#[test]
pub fn test() {
    #[cfg(debug_assertions)]
    let _guard = Assets::start_dev_server(true);

    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_secs(2));

    #[cfg(not(debug_assertions))]
    Assets::iter().for_each(|asset| {
        println!("(renaming_bundles) asset: '{:?}'", asset);
    });

    ensure_public_script();
    ensure_script_is_renamed_to_bundle();
}

fn ensure_public_script() {
    let file = Assets::get("script.js").unwrap();

    let content = std::str::from_utf8(&file.bytes).unwrap();

    assert_eq!(content, "console.log(\"public\");");
}

fn ensure_script_is_renamed_to_bundle() {
    let file = Assets::get("bundle.js").unwrap();

    let content = std::str::from_utf8(&file.bytes).unwrap();

    assert_eq!(content, "console.log(\"private\");");
}
