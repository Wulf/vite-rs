// #![feature(track_path)] // => please see comments @ crates/vite-rs/tests/recompilation_test.rs:43
#![forbid(unsafe_code)]

#[cfg(all(
    feature = "content-hash",
    any(feature = "debug-prod", not(debug_assertions))
))]
mod hash_utils;

mod syn_utils;
mod vite;

use std::{
    env,
    path::{Path, PathBuf},
};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse_macro_input, DeriveInput};

/// The root directory is the project directory where the `Cargo.toml` file is located.
/// It can be overridden by specifying a `#[root = "./"]` attribute under the derive macro.
///
/// This is used to resolve relative paths for the input and output directories.
fn derive_absolute_root_dir(ast: &syn::DeriveInput) -> syn::Result<String> {
    let mut root_attrs = syn_utils::find_attribute_values(ast, "root");
    if root_attrs.len() > 1 {
        return Err(syn::Error::new_spanned(ast, "When specifying a custom root directory, #[derive(vite_rs::Embed)] must only contain a single #[root = \"./\"] attribute."));
    }

    let root_dir = if root_attrs.len() == 0 {
        env::var("CARGO_MANIFEST_DIR").unwrap()
    } else {
        root_attrs.remove(0)
    };

    let root_dir = PathBuf::from(root_dir);
    let root_dir = if root_dir.is_relative() {
        let base = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        Path::new(&base).join(&root_dir)
    } else {
        root_dir
    };
    let root_dir = root_dir.canonicalize().expect(&format!(
        "Could not canonicalize root directory path. Does it exist? (path: {:?})",
        root_dir
    ));
    let root_dir_str = root_dir.to_str().unwrap();

    Ok(root_dir_str.to_string())
}

/// The output directory is where the compiled JS/assets are placed, relative to the `root_dir`.
/// By default, it is set to `./dist` but can be overridden by specifying a `#[output = "./dist"]` attribute under the derive macro.
///
/// Moreover, any output directory specified must be within `root_dir`.
///
/// Since this deals with compiled assets, it shouldn't be necessary for non-release builds.
#[cfg(any(feature = "debug-prod", not(debug_assertions)))]
fn derive_relative_output_dir(
    ast: &syn::DeriveInput,
    absolute_root_dir: &str,
) -> syn::Result<String> {
    let mut output_attrs = syn_utils::find_attribute_values(ast, "output");
    if output_attrs.len() > 1 {
        return Err(syn::Error::new_spanned(
            ast,
            "When specifying a custom output directory, #[derive(vite_rs::Embed)] must only contain a single #[output = \"./dist\"] attribute.",
        ));
    }

    let mut output_dir = PathBuf::from(if output_attrs.len() == 0 {
        "dist".to_string()
    } else {
        output_attrs.remove(0)
    });

    if output_dir.is_relative() {
        let root_dir = Path::new(absolute_root_dir);
        output_dir = root_dir.join(&output_dir);
    }

    // Instead of raising an error when the output directory doesn't exist,
    // we create it in release builds. This is a nicer experience.
    //
    // // if !output_dir.exists() {
    // //     return Err(syn::Error::new_spanned(
    // //         ast,
    // //         format!(
    // //             "Output directory '{}' does not exist. Please create it.",
    // //             output_dir.display()
    // //         ),
    // //     ));
    // // }
    create_output_dir_if_not_exists(&ast, &output_dir)?;

    if !output_dir.is_dir() {
        return Err(syn::Error::new_spanned(
            ast,
            format!(
                "Output directory '{}' must be a directory",
                output_dir.display()
            ),
        ));
    }

    let output_dir = output_dir.canonicalize().unwrap();

    let relative_output_dir = output_dir
            .strip_prefix(&absolute_root_dir)
            .expect("output dir specified with #[output = \"...\"] must be within the project root directory.")
            .to_str()
            .unwrap();

    Ok(relative_output_dir.to_string())
}

#[cfg(any(feature = "debug-prod", not(debug_assertions)))]
fn create_output_dir_if_not_exists(
    ast: &syn::DeriveInput,
    output_dir: &PathBuf,
) -> syn::Result<()> {
    let create_output_dir = std::fs::create_dir_all(&output_dir);

    if create_output_dir.is_err_and(|e| e.kind() != std::io::ErrorKind::AlreadyExists) {
        return Err(syn::Error::new_spanned(
            ast,
            format!("Could not create output directory (path: {:?})", output_dir),
        ));
    }

    Ok(())
}

/// The dev server port is the port where the vite-rs dev server will run and serve from.
/// By default, it is set to a free port in the range 21012..22022 but can be overridden by specifying a `#[dev_server_port = "123"]` attribute under the derive macro.
fn derive_dev_server_port(ast: &syn::DeriveInput) -> u16 {
    let dev_server_port_attrs = syn_utils::find_attribute_values(ast, "dev_server_port");
    if dev_server_port_attrs.len() > 1 {
        panic!(
            "When specifying a custom dev server port, #[derive(vite_rs::Embed)] must only contain a single #[dev_server_port = \"<YOUR_PORT>\"] attribute."
        );
    }

    let dev_server_port = dev_server_port_attrs.get(0).map(|port| {
        let port = port
            .parse::<u16>()
            .expect("dev_server_port must be a valid unsigned integer (usize).");

        // // We don't compile-time check if the port is free because the user has the option
        // // of running the dev server themselves. A runtime check does exist; see the `vite-rs-dev-server` crate.
        // if !vite_rs_dev_server::util::is_port_free(port as u16) {
        //     panic!(
        //         "Selected vite-rs dev server port '{}' is not available.",
        //         port
        //     )
        // }

        port
    });

    dev_server_port.unwrap_or_else(|| {
        // If the user doesn't specify a dev_server_port, this function
        // returns a free port in the range 21012..22022.
        //
        // This can be a source of hair-pulling because this happens on
        // macro codegen. In other words, the selected free port may be
        // available when the macro code is initially compiled, but may
        // no longer be available in subsequent runs.
        //
        // At the same time, this is necessary because we need to
        // coordinate the dev server's port with the generated code
        // (to forward requests to the dev server).
        //
        // To save everyone's time, we'll strongly encourage users to
        // specify a #[dev_server_port = 123].
        vite_rs_dev_server::util::find_free_port(21012..22022)
            .expect("Could not find a free port for the ViteJS dev server")
    })
}

/// If crate_path is defined, use that as a syn::Path, otherwise use the crate's name.
/// This is useful when someone is using this crate from a crate path that is different from
/// the default: `crate::vite_rs`. In that case, they can specify something like:
/// `#[crate_path = "some::path::to::vite_rs"]`.
fn derive_crate_path(ast: &syn::DeriveInput) -> syn::Result<syn::Path> {
    let crate_path_attrs = syn_utils::find_attribute_values(ast, "crate_path");
    if crate_path_attrs.len() > 1 {
        return Err(syn::Error::new_spanned(
            ast,
            "When specifying a custom crate path, #[derive(vite_rs::Embed)] must only contain a single #[crate_path = \"crate_name\"] attribute.",
        ));
    }

    let crate_path = {
        if crate_path_attrs.len() == 0 {
            // we don't use env!("CARGO_PKG_NAME") because this code is in the vite-rs-embed-macro, but the end user will be using vite-rs
            "vite_rs"
        } else {
            &crate_path_attrs.get(0).unwrap()
        }
    };

    Ok(syn::parse_str::<syn::Path>(crate_path)?)
}

fn impl_vitejs_embed(ast: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    syn_utils::ensure_unit_struct(ast)?;

    let absolute_root_dir = derive_absolute_root_dir(ast)?;
    #[cfg(any(feature = "debug-prod", not(debug_assertions)))]
    let relative_output_dir = derive_relative_output_dir(ast, &absolute_root_dir)?;
    let crate_path = derive_crate_path(ast)?;

    let dev_server_host = "localhost";
    let dev_server_port = derive_dev_server_port(ast);

    vite::build::generate_rust_code(
        /* dev-only */
        #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
        dev_server_host,
        /* dev-only */
        #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
        dev_server_port,
        &crate_path,
        &ast.ident,
        &absolute_root_dir,
        /* prod-only */
        #[cfg(any(feature = "debug-prod", not(debug_assertions)))]
        &relative_output_dir,
    )
}

/// For explanations of the attributes, please see:
/// - #[root]: derive_absolute_root_dir (define above)
/// - #[output]: derive_relative_output_dir (define above)
/// - #[dev_server_port]: derive_dev_server_port (define above)
/// - #[crate_path]: derive_crate_path (define above)
#[proc_macro_derive(Embed, attributes(root, output, dev_server_port, crate_path))]
pub fn derive_input_object(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match impl_vitejs_embed(&ast) {
        Ok(ok) => ok.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
