use std::collections::HashMap;

#[allow(dead_code, non_snake_case)]
#[derive(serde::Deserialize)]
pub struct ViteManifestEntry {
    /// Script content to load for this entry
    pub file: String,

    /// Script content to lazy-load for this entry
    pub dynamicImports: Option<Vec<String>>, // using `import(..)`

    /// Style content to load for this entry
    pub css: Option<Vec<String>>, // using import '*.css'

    /// If true, eager-load this content
    pub isEntry: Option<bool>,

    /// If true, lazy-load this content
    pub isDynamicEntry: Option<bool>, // src: String, /* => not necessary :) */
                                      // assets: Option<Vec<String>>, /* => these will be served by the server! */
}

pub type ViteManifest = HashMap<String, ViteManifestEntry>;

pub fn load_vite_manifest(path: &str) -> ViteManifest {
    let manifest_json_str = std::fs::read_to_string(path).unwrap();

    parse_vite_manifest_json_str(&manifest_json_str)
}

pub fn parse_vite_manifest_json_str(manifest_json: &str) -> ViteManifest {
    let manifest_json = serde_json::from_str(manifest_json).expect("failed to parse vite manifest");

    let mut manifest: ViteManifest = HashMap::new();

    match manifest_json {
        serde_json::Value::Object(obj) => {
            obj.keys().for_each(|manifest_key| {
                let details = obj.get(manifest_key).unwrap();

                let manifest_entry = serde_json::from_value::<ViteManifestEntry>(details.clone())
                    .expect(
                        "invalid vite manifest (or perhaps the vite-rs parser isn't up-to-date!)",
                    );

                manifest.insert(manifest_key.to_string(), manifest_entry);
            });
            // done parsing manifest
        }
        _ => {
            panic!("invalid vite manifest (or perhaps the vite-rs parser isn't up-to-date!)");
        }
    }

    manifest
}
