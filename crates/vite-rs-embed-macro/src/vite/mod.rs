/// PROD build
#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
pub mod build {
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use std::{collections::BTreeMap, path::PathBuf};

    mod file_entry;
    use file_entry::FileEntry;
    mod vite_manifest;

    fn list_compiled_files(absolute_output_path: &str) -> Vec<String> {
        let compiled_files = walkdir::WalkDir::new(absolute_output_path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| {
                let path = entry.path();
                let path = path.strip_prefix(absolute_output_path).unwrap();
                let path = path.to_str().unwrap().to_string();

                path
            })
            .filter(|path| !path.starts_with(".vite")) // ignore vite manifest or other vite-internal files
            .collect::<Vec<_>>();

        compiled_files
    }

    pub fn generate_rust_code(
        crate_path: &syn::Path,
        struct_ident: &syn::Ident,
        absolute_root_dir: &str,
        relative_output_dir: &str,
    ) -> syn::Result<TokenStream2> {
        // proc_macro::tracked_path::path(absolute_root_dir); // => please see comments @ crates/vite-rs/tests/recompilation_test.rs:43

        let absolute_output_path = {
            let p = PathBuf::from_iter(&[absolute_root_dir, relative_output_dir]);
            let p = p.canonicalize().expect(&format!(
                "Could not canonicalize output directory path. Does it exist? (path: {:?})",
                p
            ));

            p.to_str().unwrap().to_string()
        };

        let vite_build = std::process::Command::new("npx")
            .arg("vite")
            .arg("build")
            .arg("--manifest") // force manifest generation to `.vite/manifest.json`
            .arg("--outDir")
            .arg(&absolute_output_path)
            .current_dir(absolute_root_dir)
            .spawn()
            .expect("failed to build")
            .wait()
            .expect("failed to wait for build to complete")
            .success();

        if !vite_build {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "ViteJS build failed",
            ));
        }

        // the vite manifest is only available AFTER the build, so don't move this line up :)
        let absolute_vite_manifest_path = {
            let p = PathBuf::from_iter(&[&absolute_output_path, ".vite", "manifest.json"])
                .canonicalize()
                .expect(&format!(
                    "Could not canonicalize ViteJS manifest path. Does it exist? (path: {:?})",
                    absolute_output_path
                ));

            p.to_str().unwrap().to_string()
        };

        let vite_manifest = vite_manifest::load_vite_manifest(&absolute_vite_manifest_path);

        let mut match_values = BTreeMap::new();
        let mut list_values = Vec::<String>::new();

        list_compiled_files(&absolute_output_path)
            .iter()
            .flat_map(|relative_file_path| {
                let absolute_file_path = {
                    let p = PathBuf::from_iter(&[&absolute_output_path, relative_file_path])
                        .canonicalize()
                        .expect("Failed to canonicalize");

                    p.to_str().expect("Failed to convert to string").to_string()
                };

                list_values.push(relative_file_path.clone());
                println!(
                    "Adding entry: {} for {}",
                    &relative_file_path, absolute_file_path
                );

                FileEntry::new(relative_file_path.clone(), absolute_file_path).map_err(|e| {
                    return syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Failed to read Vite manifest: {}", e),
                    );
                })
            })
            .for_each(|entry| {
                match_values.insert(entry.match_key().clone(), entry.match_value(&crate_path));
            });

        // Aliases help us refer to entrypoints from their uncompiled name.
        //
        // For example:
        // - A compiled file 'dist/pack1-1234.js' would originally be 'src/pack1.ts'.
        //   Therefore, Struct::get("src/pack1.ts") should return the contents of 'dist/pack1-1234.js'.
        let aliases = {
            let mut aliases = BTreeMap::new();

            vite_manifest
                .iter()
                .filter(|e| e.1.isEntry.unwrap_or(false))
                .for_each(|(key, value)| {
                    if !match_values.contains_key(key) {
                        aliases.insert(key.clone(), value.file.clone());
                    }
                });

            aliases.into_iter().map(|(alias, path)| {
                quote! {
                    (#alias, #path),
                }
            })
        };

        let match_values = match_values.into_iter().map(|(path, bytes)| {
            quote! {
                (#path, #bytes),
            }
        });

        let array_len = list_values.len();

        Ok(quote! {
            impl #struct_ident {
                /// Path resolution; handles aliasing for file paths
                fn resolve(path: &str) -> &str {
                    const ALIASES: &'static [(&'static str, &'static str)] = &[
                        #(#aliases)*
                    ];

                    let path = ALIASES.binary_search_by_key(&path, |entry| entry.0).ok().map(|index| ALIASES[index].1).unwrap_or(path);

                    path
                }

                pub fn get(path: &str) -> Option<#crate_path::ViteFile> {
                    let path = Self::resolve(path);

                    const ENTRIES: &'static [(&'static str, #crate_path::ViteFile)] = &[
                        #(#match_values)*
                    ];
                    let position = ENTRIES.binary_search_by_key(&path, |entry| entry.0);
                    position.ok().map(|index| ENTRIES[index].1.clone())
                }

                fn names() -> ::std::slice::Iter<'static, &'static str> {
                    const ITEMS: [&str; #array_len] = [#(#list_values),*];
                    ITEMS.iter()
                }

                /// Iterates over the file paths in the compiled ViteJS output directory
                pub fn iter() -> impl ::std::iter::Iterator<Item = ::std::borrow::Cow<'static, str>> {
                    Self::names().map(|x| ::std::borrow::Cow::from(*x))
                }

                pub fn boxed() -> ::std::boxed::Box<dyn #crate_path::GetFromVite> {
                    ::std::boxed::Box::new(#struct_ident {})
                }
            }

            impl #crate_path::GetFromVite for #struct_ident {
                fn get(&self, file_path: &str) -> ::std::option::Option<#crate_path::ViteFile> {
                    #struct_ident::get(file_path)
                }

                fn clone_box(&self) -> ::std::boxed::Box<dyn #crate_path::GetFromVite> {
                    ::std::boxed::Box::new(#struct_ident {})
                }
            }
        })
    }
}

/// DEV build
#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
pub mod build {
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;

    pub fn generate_rust_code(
        dev_server_host: &str,
        dev_server_port: u16,
        crate_path: &syn::Path,
        struct_ident: &syn::Ident,
        absolute_root_dir: &str,
    ) -> syn::Result<TokenStream2> {
        #[cfg(feature = "ctrlc")]
        let start_dev_server = quote! {
            pub fn start_dev_server(
                register_ctrl_c_handler: bool,
            ) -> Option<#crate_path::vite_rs_dev_server::ViteProcess> {
                #crate_path::vite_rs_dev_server::start_dev_server(#absolute_root_dir, #dev_server_host, #dev_server_port, register_ctrl_c_handler)
            }
        };

        #[cfg(not(feature = "ctrlc"))]
        let start_dev_server = quote! {
            pub fn start_dev_server() -> Option<#crate_path::vite_rs_dev_server::ViteProcess> {
                #crate_path::vite_rs_dev_server::start_dev_server(#absolute_root_dir, #dev_server_host, #dev_server_port)
            }
        };

        let etag = if cfg!(feature = "content-hash") {
            quote! {
                let etag = res
                    .headers()
                    .get(#crate_path::vite_rs_dev_server::reqwest::header::ETAG)
                    .expect("FATAL: ViteJS dev server did not return an `ETag` header.")
                    .to_str()
                    .unwrap()
                    .to_string();
            }
        } else {
            quote! {}
        };

        let content_hash = if cfg!(feature = "content-hash") {
            quote! { content_hash: etag, }
        } else {
            quote! {}
        };

        Ok(quote! {
            impl #struct_ident {
                #start_dev_server

                pub fn stop_dev_server() {
                    #crate_path::vite_rs_dev_server::stop_dev_server()
                }

                pub fn iter() -> impl ::std::iter::Iterator<Item = ::std::borrow::Cow<'static, str>> {
                    // https://github.com/rust-lang/rust/issues/36375
                    if true {
                        unimplemented!("iter() is out of scope for dev builds and is left unimplemented. It is available in release builds (or when the `debug-prod` feature is enabled)")
                    } else {
                        vec![].into_iter()
                    }
                }

                pub fn get(path: &str) -> Option<#crate_path::ViteFile> {
                    let path = path.to_string();
                    std::thread::spawn(move || {
                        let client = #crate_path::vite_rs_dev_server::reqwest::blocking::Client::new();
                        let url = format!(
                            "http://{}:{}/{}",
                            #dev_server_host,
                            #dev_server_port,
                            path
                        );

                        match client.get(&url).send() {
                            Ok(res) => {
                                if res.status() == 404 {
                                    return None;
                                }

                                let content_type = res
                                    .headers()
                                    .get(#crate_path::vite_rs_dev_server::reqwest::header::CONTENT_TYPE)
                                    .expect("FATAL: ViteJS dev server did not return a content type!")
                                    .to_str()
                                    .unwrap()
                                    .to_string();

                                let content_length = res
                                    .content_length()
                                    .expect("FATAL: ViteJS dev server did not return a `Content-Length` header.");

                                #etag

                                let mut bytes = res.bytes().unwrap().to_vec();

                                /// check if the path is a css file, if so, print a warning if the content type isn't text/css:
                                if path.ends_with(".css") && !content_type.eq("text/css") {
                                    println!(
                                        "**WARNING**: A request for a CSS resource ({}) was served javascript. This is likely because you've referenced a CSS file in your HTML which was not in the `public` directory (configurable in the vite's config). This will still be fine in release builds, but the CSS won't be applied correctly in development. You can move the CSS file to the public directory (for example: `public/styles.css`) and reference it in your HTML with a `<link rel=\"stylesheet\" href=\"/styles.css\">` tag. Alternatively, you can import the CSS file in your JavaScript code with `import './styles.css';`.",
                                        &url
                                    );
                                }

                                Some(#crate_path::ViteFile {
                                    last_modified: None, /* we don't send this in dev! */
                                    content_type: content_type,
                                    content_length: content_length,
                                    bytes: bytes,
                                    #content_hash
                                })
                            }
                            Err(e) => {
                                println!("ERR! {:#?}", e);
                                None
                            },
                        }
                    })
                    .join()
                    .expect("Failed to spawn thread to fetch ViteJS dev server resource.")
                }

                pub fn boxed() -> ::std::boxed::Box<dyn #crate_path::GetFromVite> {
                    ::std::boxed::Box::new(#struct_ident {})
                }
            }

            impl #crate_path::GetFromVite for #struct_ident {
                fn get(&self, file_path: &str) -> Option<#crate_path::ViteFile>  {
                    #struct_ident::get(file_path)
                }

                fn clone_box(&self) -> ::std::boxed::Box<dyn #crate_path::GetFromVite> {
                    ::std::boxed::Box::new(#struct_ident {})
                }
            }
        })
    }
}
