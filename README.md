_Cargo, please compile & bundle the frontend too. Thanks._

# vite-rs

[![Crates.io](https://img.shields.io/crates/v/vite-rs?labelColor=%2355b5ff&color=%238f6cfe)](https://crates.io/crates/vite-rs)
![Tests](https://img.shields.io/github/actions/workflow/status/Wulf/vite-rs/test.yml?branch=main)

⚡ Seamlessly integrate ViteJS into your Rust project.

- Embeds Vite output in your binary.
- Optionally manages the Vite dev server lifecycle in Rust.
- Low-touch;
  - No build script changes required.
  - No `package.json` changes required.
  - No Vite config changes, but:
    - this crate [forces](#require-manifest-true-note) manifest generation,
    - and requires [specifying the output directory](#require-specifying-custom-vite-build-dir) (in Rust) if you've changed it from the default in your vite config.

> [!CAUTION]
> We've written tests for Unix operating systems. Windows support is still a work-in-progress. Please report any issues you encounter!

```rust
#[derive(vite_rs::Embed)]
#[root = "./app"]
struct Assets;

fn main() {
  let asset: ViteFile = Assets::get("index.html").unwrap();

  println!("Content-Type: {}", asset.content_type);
  println!("Content-Length: {}", asset.content_length);
  println!("Last-Modified: {}", asset.last_modified);
  println!("Content: {}", std::str::from_utf8(&asset.bytes).unwrap());
}
```

## Table of Contents

- [Quick Start](#quick-start)
- [API](#api)
- [Options](#options)
  - [`#[root = "<path>"]`](#root--path)
  - [`#[output = "<path>"]`](#output--path)
  - [`#[dev_server_port = "<port>"]`](#dev_server_port--port)
  - [`#[crate_path = "<path>"]`](#crate_path--path)
- [Full Guide](#full-guide)
- [Notes](#notes)
  - [Vite config options that require special consideration](#vite-config-options-that-require-special-consideration)
  - [Manage the ViteJS dev server lifecycle](#manage-the-vitejs-dev-server-lifecycle-yourself)
  - [Templating](#templating)
  - [Web Frameworks](#web-frameworks)
  - [Ctrl-C Handling](#ctrl-c-handling)
  - [What's included in the release binary?](#whats-included-in-the-release-binary)
  - [How can I automatically bundle all files that match a pattern?](#how-can-i-automatically-bundle-all-files-that-match-a-pattern-like-html-bundletstsxjsjsx-etc-without-manually-listing-them)
  - [A note on compile times and large (or many) assets](#a-note-on-compile-times-and-large-or-many-assets)
  - [A note about unnecessary release rebuilds](#a-note-about-unnecessary-release-rebuilds)
  - [Why double down on ViteJS in your project (as opposed to using a crates that bundle files)?](#why-double-down-on-vitejs-in-your-project-as-opposed-to-using-a-crates-that-bundle-files)
- [Acknowledgements](#acknowledgements)

## Quick Start

1. You'll need a ViteJS project.

   - **Option A**: You already have one. Move on to step 2.

   - **Option B**: Follow the [ViteJS docs](https://vitejs.dev/guide/#scaffolding-your-first-vite-project) to create a new project.

     Example: to create a project at `./app` with the `react-ts` template:

     ```sh
     cd your/rust/project
     npm create vite@latest ./app -- --template react-ts
     ```

   - **Option C**: Create a barebones project from scratch.

     ```sh
     cd your/rust/project
     npm install -D vite
     ```

     Create `app/vite.config.ts`:

     ```ts
     import { defineConfig } from "vite";
     // if using react, uncomment below and install via `npm i -D @vitejs/plugin-react`
     // import react from '@vitejs/plugin-react'

     export default defineConfig({
       plugins: [
         /* react() */
       ],
       build: {
         rollupOptions: {
           input: ["index.html"],
         },
       },
     });
     ```

     You'll eventually have to change the `input` array to include all your entrypoints. Alternatively, see the [section below](#auto-bundle-all-files) for a way to automatically bundle all files that match a pattern.

     Create `app/index.html`:

     ```html
     <!DOCTYPE html>
     <html lang="en">
       <head>
         <title>Hello World</title>
       </head>
       <body>
         <h1>Hello, world!</h1>
       </body>
     </html>
     ```

2. Add this crate as a dependency.

   ```sh
   cargo add vite-rs
   ```

3. Use the crate!

   ```rust
    #[derive(vite_rs::Embed)]
    #[root = "./app"]
    struct Assets;

    fn main() {
        #[cfg(debug_assertions)]
        // Optional: start the dev server and stop it on SIGINT, SIGTERM or when this guard goes out of scope
        let _guard = Assets::start_dev_server(true);

        // ...

        let asset: vite_rs::ViteFile = Assets::get("index.html").unwrap();

        println!("Content-Type: {}", asset.content_type);
        println!("Content-Length: {}", asset.content_length);
        println!("Last-Modified: {:?}", asset.last_modified);
        println!("Content: {}", std::str::from_utf8(&asset.bytes).unwrap());
    }
   ```

4. Run your binary!

   ```sh
   # Assets served from a Vite dev server:
   cargo run

   # or, to see the embedded assets in action:
   cargo run --release
   ```

See the `crates/vite-rs/examples` and `crates/vite-rs/tests` folders for more examples.

## API

When you derive the `vite_rs::Embed` trait, some methods are generated for your struct which allow you to interact with your Vite assets. In development, the methods differ in behavior from release builds.

```rust
#[derive(vite_rs::Embed)]
struct Assets;
```

### Release

- **GET ASSET**: Get an asset by its path. Fetches assets embedded into the binary.

  ```rust
  Assets::get(path: &str) -> Option<vite_rs::ViteFile>
  ```

- **ITERATE OVER ASSETS**: Get an iterator over all assets.

  ```rust
  Assets::iter() -> impl Iterator<Item = Cow<'static, str>>
  ```

### Development

- **GET ASSET**: Get an asset by its path. Fetches assets from the dev server.

  ```rust
  Assets::get(path: &str) -> Option<vite_rs::ViteFile>
  ```

- **START DEV SERVER**: Starts the ViteJS dev server. This function returns an [RAII guard](https://doc.rust-lang.org/rust-by-example/scope/raii.html) that stops the dev server when it goes out of scope.

  ```rust
  // with `ctrlc` feature enabled:
  Assets::start_dev_server(register_ctrl_c_handler: bool) -> vite_rs::ViteProcess

  // without `ctrlc` feature:
  Assets::start_dev_server() -> vite_rs::ViteProcess
  ```

  Note: The `ctrlc` feature is enabled by default. If you pass in `true` for `register_ctrl_c_handler`, it will stop the dev server on SIGTERM/SIGINT/SIGHUP.

- **STOP DEV SERVER**: Stops the ViteJS dev server.

  ```rust
  Assets::stop_dev_server()
  ```

Note: In development, you cannot iterate over all assets because there is no way to do so using the Vite dev server.

### Release

```rust
#[derive(vite_rs::Embed)]
struct Assets;

fn main() {
    // Get an asset by its path
    let asset = Assets::get("index.html"); // -> : Option<vite_rs::ViteFile>

    // Get an iterator over all assets
    let assets = Assets::iter(); // -> impl Iterator<Item = Cow<'static, str>>

    // Start the ViteJS dev server
    let guard: vite_rs::ViteProcess = Assets::start_dev_server(true);

    // Stop the ViteJS dev server
    Assets::stop_dev_server();
}
```

## Options

The derive macro (`#[vite_rs::Embed]`) supports the following options:

### `#[root = "<path>"]`

- Specifies the directory where your Vite config lives.

  **Notes:**

  - Defaults to [`CARGO_MANIFEST_DIR`](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates#:~:text=CARGO_MANIFEST_DIR).

  - If your Vite config is in the same directory as your `Cargo.toml` file, you don't need to specify this.

  - The root directory is where the vite commands are run from; that means `node_modules` should be in this directory (or any parent).

  **Example Usage:**

  - If our vite config was located in `./app`:

    ```rust
    #[vite_rs::Embed]
    #[root = "./app"]
    struct Assets;
    ```

### <a name="options--output"></a>`#[output = "<path>"]`

- Specifies the directory where Vite outputs build files.

  **Notes:**

  - Defaults to `./dist` (Vite's default output directory)

  - [Required if you change `build.outDir`](#require-specifying-custom-vite-build-dir) in your Vite config.
  - Path is relative to the root directory.

  **Example Usage:**

  - If we had a custom value for `build.outDir` in our Vite config:

    ```ts
    // vite.config.ts
    import { defineConfig } from "vite";

    export default defineConfig({
      build: {
        outDir: "build",
      },
    });
    ```

    We would use the attribute like so:

    ```rust
    #[vite_rs::Embed]
    #[output = "./build"] // the outDir from your vite config
    struct Assets;
    ```

### `#[dev_server_port = "<port>"]`

- Specifies which port the Vite dev server is running on.

  **Notes:**

  - Defaults to `3000`.

  - [Use this if you manage the dev server lifecycle yourself](#self-managed-dev-server).

  **Example Usage:**

  - If our Vite dev server was running on port `3001`:

    ```rust
    #[vite_rs::Embed]
    #[dev_server_port = "3001"]
    struct Assets;
    ```

### `#[crate_path = "<path>"]`

- Specifies a custom path to the `vite_rs` crate.

  **Notes:**

  - Defaults to `vite_rs`.

  - This is useful when `vite_rs` is not in the same crate as the struct you're embedding assets in.

  **Example Usage:**

  - If we had a struct in a different crate:

    ```rust
    use my::path::to::vite_rs;

    #[vite_rs::Embed]
    #[crate_path = "my::path::to::vite_rs"]
    struct Assets;
    ```

## Full Guide

`vite-rs` makes it easy to use ViteJS in your Rust project. It tries to be simple by not requiring any changes to build scripts, Vite config files, or introduce additional tools/CLI. Everything is done via `cargo`:

- `cargo build --release` embeds the ViteJS-compiled artifacts into your binary. This means you don't have to copy any files around and manage how they are deployed. They ship with your binary.

- `cargo run` also starts the ViteJS dev server for you. This means you don't have to run a command to start the dev server (ex: `npm run dev`) in a separate terminal. You'll still need your source files for this though; it's meant to be used in development. If you prefer running it yourself, [you can do that too](#self-managed-dev-server).

There are some advantages to using `vite-rs`:

- Other than an `npm install`, you won't really have other steps to take for building/deploying your frontend because it'll ship with your app.

- It's faster by default: your assets are pre-loaded alongside your binary's bytecode.

- It'll make end-to-end testing easier since your Vite project is tightly tied to your Rust project.

- Lastly, it's subjectively going to lead to better development experience because everything is done using `cargo`. (Except `npm install` and `npm test`, I guess?)

As with all things in life, there are considerations to take into account before using `vite-rs`:

- It's one more thing to debug when things go wrong.

- It'll increase compile time. See the note on [compile times](#compile-times) for large assets.

- For those deploying to embedded devices: it'll increase your binary size.

- Shipping frontend with your backend can slow you down as you'll have to wait for your Rust backend to compile everytime you want to release a new build. Similarly, failing CI/CD pipelines pertaining to the backend will also stop your frontend frontend from deploying.

- It may be faster or cheaper to deploy your frontend on CDNs instead of serving it.

We hope this advice helps you decide whether `vite-rs` is right for your project.

#### Why ViteJS?

At the moment, there aren't any other crates that provide similar _frontend-bundling-into-binary_ functionality; but you might ask: why integrate with ViteJS instead of `esbuild`, `swc[-pack]`, or similar tools?

Naturally, since `swc` is written in Rust (and crates like `esbuild-rs` exist), we tried using those first. In short, we created this crate _after_ experimenting with these lower-level build tools. Our conclusion was that integrating with ViteJS is [a better bet for the forseeable future](#why-vite). That being said, it is also worth mentioning that some large communities are trying to move away from these frontend build tools altogether and make their asset pipelines simpler (see Rails 8).

#### Let's compile and embed a view!

A ViteJS project has entrypoints that get compiled and placed in an output directory (`./dist/` by default). For example, if we had an entrypoint file called `views/index.html` in a ViteJS project, it would be compiled and placed in the output directory (e.g. `./dist/views/index.html`). These files can be referenced and used in your Rust project.

For our example, we'll assume your project looks something like this:

```
webapp/
|- node_modules/
|- package.json
|- vite.config.ts  # your vite config
|- Cargo.toml
|- src/
|  |- main.rs      # your binary code
|- views/
   |- index.html
```

Your vite configuration should list your entrypoints:

```ts
// vite.config.ts
import { defineConfig } from "vite";

export default defineConfig({
  build: {
    rollupOptions: {
      input: ["views/index.html"],
    },
  },
});
```

This will tell Vite to compile your `index.html` file and place it, as well as any assets it depends on (scripts/styles/media), in the output directory.

Now, you can use the `vite-rs` crate to embed this file and its dependencies in your Rust project. We'll create a struct to reference assets:

```rust
#[derive(vite_rs::Embed)]
#[root = "./"] // where the vite config lives
struct Assets;
```

Note: we specified the `#[root]` attribute, but it isn't required in this example because (by default) it points to the directory where your `Cargo.toml` file is.

Now, you can use the `Assets` struct to access the compiled view:

```rust
let asset: vite_rs::ViteFile = Assets::get("views/index.html").unwrap();
```

The `ViteFile` struct has the following fields:

- `content_type`: The content type of the asset.
- `content_length`: The length of the asset in bytes.
- `last_modified`: The last modified date of the asset.
- `bytes`: The asset's bytes.

Altogether, your binary code should look something like this:

```rust
#[derive(vite_rs::Embed)]
#[root = "./"] // where the vite config lives
struct Assets;

fn main() {
    let asset: vite_rs::ViteFile = Assets::get("views/index.html").unwrap();

    println!("Asset: {:?}", std::str::from_utf8(&asset.bytes).unwrap());
}
```

When built or run in release mode (`cargo run --release` or `cargo build --release`), your the compiled frontend assets will be embedded into your binary and as a result, it will print the content of `views/index.html`. However, if you tried running this code in debug mode (`cargo run`) without starting a local ViteJS dev server, you'll end up seeing this message:

```
thread 'main' panicked at src/main.rs:5:68:
called `Option::unwrap()` on a `None` value
```

The reason is because we haven't setup Rust to handle the dev server lifecycle. We'll do this in the next section. For now, let's run it ourselves:

```sh
npx vite --port 3000
```

and tell Rust to use this port:

```diff
  #[derive(vite_rs::Embed)]
  #[root = "./"]
+ #[dev_server_port = "3000"]
  struct Assets;
```

Now, a `cargo run` should print out the HTML content as expected.

#### Asking `cargo` to manage the Vite dev server lifecycle

Our next step is to get `vite-rs` to start and stop the dev server in debug builds. **This is entirely optional**, but could be a subjectively better development experience.

Simply call `Assets::start_dev_server` in your binary's main method. You'll want this to be one of the first things that run because the ViteJS dev server needs time to start. This function returns an [RAII](https://doc.rust-lang.org/rust-by-example/scope/raii.html) guard that will stop the dev server when it goes out of scope. For example:

```rust
fn main() {
  #[cfg(debug_assertions)]
  let _guard = Assets::start_dev_server(true);

  // ...

  // The dev server will stop when `_guard` goes out of scope
}
```

It's unlikely that the `_guard` will go out of scope in a web backend because it usually runs indefinitely; however, you may have noticed we passed `true` to `start_dev_server`. This registers a default signal handler that stops the ViteJS dev server for certain signals like Ctrl-C. If you want to handle signals yourself (for example, to do some cleanup before exiting), you can disable the default handler by passing false or disabling the `ctrlc` feature. See the [Ctrl-C Handling](#ctrl-c-handler) section for more details.

Our final binary will look like the following:

```diff
#[derive(vite_rs::Embed)]
#[root = "./"] // where the vite config lives
struct Assets;

fn main() {
+    #[cfg(debug_assertions)]
+    let _guard = Assets::start_dev_server(true);

+    #[cfg(debug_assertions)]
+    std::thread::sleep(std::time::Duration::from_millis(500));

    let asset: vite_rs::ViteFile = Assets::get("views/index.html").unwrap();

    println!("Asset: {:?}", std::str::from_utf8(&asset.bytes).unwrap());

+    // The dev server will stop when `_guard` goes out of scope
}
```

> [!NOTE]
> We give 500ms for the dev server to start in this example. This shouldn't be necessary in webapp projects because the dev server would be ready by the time you switch to your browser. Moreover, frontend changes shouldn't cause a recompile/restart and instead propagate via Vite's [hot-module-replacement](https://vite.dev/guide/features.html#hot-module-replacement).

And that's pretty much all there is to it!

# Notes

### <a name="vite-config-note"></a>Vite config options that require special consideration

- <a name="require-manifest-true-note"></a> [`build.manifest`](https://vite.dev/config/build-options.html#build-manifest): This option will automatically be overriden to `true` so that ViteJS generates a manifest file for builds. You don't need to update your config as `vite-rs` overrides it via a CLI flag when building. This means custom values won't be respected. To illustrate:

  ```ts
  // vite.config.ts
  import { defineConfig } from "vite";

  export default defineConfig({
    build: {
      manifest: false, // this will be overridden to true
    },
  });
  ```

- <a name="require-specifying-custom-vite-build-dir"></a> [`build.outDir`](https://vite.dev/config/build-options.html#build-outdir): When using a custom ViteJS build directory (via the config option), you have to let `vite-rs` know as well by using the `#[output="your-output-dir"]` attribute. See the example in the [attribute's documentation](#options--output) section.

### <a name="self-managed-dev-server"></a>Manage the ViteJS dev server lifecycle yourself

If you'd like to manage the ViteJS dev server lifecycle yourself, you have to tell `vite-rs` which port it's serving from. For that, use the `#[dev_server_port]` attribute:

```rust
#[vite_rs::Embed]
#[root = "./vite-app"]
#[dev_server_port = "3000"] // the port your vite dev server is running on
struct Assets;
```

Now you can `npm start` your ViteJS dev server yourself, and `vite-rs` will know how to fetch assets from it in non-release runs.

### Templating

Integration with templating engines like Askama / Tera / Handlebars is currently out of scope.

Without templating, this library forces us to separate backend<>frontend concerns and also removes the need to introduce template-specific syntax into HTML files. That being said, this isn't necessarily beneficial for every project.

If the community wants to approach this, here are some considerations to take note of:

1. Some templating libraries can preprocess templates and keep them in memory. Since vite-rs embeds the template in the binary, you'll have duplicate copies of the template in memory.

2. Some templating libraries allow you to specify custom functions that can be used in the rendering process. This could be used to load the vite asset from the struct. For example:

   ```
   <!DOCTYPE html>
   <html>
       <body>
           {{ include_bundle('index.ts') }}
       </body>
   </html>
   ```

   The implementation of `include_bundle` would output a script tag that loads the vite asset, and potentially more (to get HMR to work). For all the details, see the ViteJS docs for [backend integrations](https://vite.dev/guide/backend-integration.html).

   Moreover, you can see `create-rust-app`'s [ViteJS integration for Rust backends](https://github.com/Wulf/create-rust-app/blob/main/create-rust-app/src/util/template_utils.rs#L44) which uses Tera for templating.

3. We currently don't embed the manifest file in the binary. See the section about [what is included in the release binary](#what-is-included-in-the-release-binary) for more details. This means the above is likely not possible at the moment. Feel free to open a PR to expose the manifest as an asset (it should be a simple change).

### Web Frameworks

We welcome contributions for specific web frameworks (actix, axum, etc). If you end up creating an integration crate, please let us know so we can link to it here.

### <a name="ctrl-c-handler"></a>Ctrl-C Handling

This library provides a default Ctrl-C handler that stops the ViteJS dev server before the process exits in development builds. If you use a custom termination signal handler, you'll need to disable this by passing in `false` to start_dev_server():

```rust
#[vite_rs::Embed]
#[input = "./assets"]
struct Assets;

fn main() {
    #[cfg(debug_assertions)]
    let _guard = Assets::start_dev_server(false);

    // ...
}
```

Then, you should modify your signal handler to stop the ViteJS dev server process in debug mode. Otherwise, you may see some errors after your binary is killed. Here's an example of how to do this with the `ctrlc` crate:

```rust
#[vite_rs::Embed]
#[input = "./assets"]
struct Assets;

fn main() {
    #[cfg(debug_assertions)]
    let _guard = Assets::start_dev_server(false);

    ctrlc::set_handler(move || {
        #[cfg(debug_assertions)]
        Assets::stop_dev_server();

        // ...

        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");
}
```

See the full example at [example/custom_ctrl_c_handler.rs](example/custom_ctrl_c_handler.rs).

If you don't use the default Ctrl-C handler, you can disable the feature entirely by using the `default-features = false` option in your `Cargo.toml` file:

```toml
[dependencies]
vite-rs = { ..., default-features = false }
```

### <a name="what-is-included-in-the-release-binary"></a>What's included in the release binary?

All compiled assets are included unless they're in the `<output_dir>/.vite` directory. That means `.vite/manifest.json` is not included as an asset.

To see a full example, clone this repository and run `cargo build --test normal_usage --release` in the `crates/vite_rs` directory. You'll see the compiled assets in the `crates/vite_rs/tests/normal_usage/dist` directory.

### <a name="auto-bundle-all-files"></a>How can I automatically bundle all files that match a pattern (like `*.html`, `*.bundle.(ts|tsx|js|jsx)`, etc.) without manually listing them?

> [!NOTE]  
> This has nothing to do with `vite-rs` -- it's something you'll have to configure in your Vite config. Below, we give an example of how to achieve this.

Use the [`glob`](https://github.com/isaacs/node-glob#readme) npm package to find all file names that match a particular pattern. For example, to include all `.html` and `.bundle.(ts|tsx|js|jsx)` files in the directory where `vite.config.ts` lives, you can use the following configuration:

```sh
npm i -D glob
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import path from "path";
import { globSync } from "glob";

export default defineConfig(() => ({
  build: {
    rollupOptions: {
      input: [
        ...globSync(
          [
            path.resolve(__dirname, "**/*.bundle.{js,jsx,ts,tsx}"),
            path.resolve(__dirname, "**/*.html"),
          ],
          { ignore: "node_modules/**" }
        ),
      ],
    },
    manifest: true,
  },
}));
```

### <a name="compile-times"></a>A note on compile times and large (or many) assets

For release builds, ViteJS output will be read and written as binary arrays into your executable using `include_bytes!`. [This could increase compilation times if there are large assets](https://github.com/rust-lang/rust/issues/65818), and, if there are too many.

To troubleshoot, try running `cargo build --release` and check your ViteJS build output directory (e.g. `./dist`) to see which assets may be causing large compile times.

A final note on this matter: Vite's `public` directory is also included in the binary. This means you may have to be conscious of the size and quantity of the assets you're embedding.

If you're experiencing long compile times, please file an issue with your findings and also consider using a different method to embed assets in your binary.

So far, this hasn't been an issue.

### A note about unnecessary release rebuilds

Currently, ViteJS always recompiles production assets from scratch. This trips the rust compiler to rebuild from scratch as well since the ViteJS build output files get modified. We know of a workaround to address this but have chosen not to implement it because it could result in confusing behaviour (plus, we still won't be able to stop vite from recompiling from scratch). For more information, see the [`crates/vite-rs/tests/recompilation_test.rs`](crates/vite-rs/tests/recompilation_test.rs) test file.

### <a name="why-vite"></a>Why double down on ViteJS in your project (as opposed to using crates that bundle files)?

Integrating with bundling tools like `swc-pack` or `esbuild-rs` is a huge maintenance burden. Public API documentation is lacking, some features are not considered production-ready, and their maintenance is not guaranteed. They just don't have the same level of community support as ViteJS. Moreover, reimplementation of module preloading, HMR, code splitting/chunking and third-party plugins is a huge undertaking that we'd rather not take ownership of.

In the future, `rolldown`, which is still in development, may allow us to achieve a Rust-based integration (without NPM/JS) but even if we're slow to switch over, we'll benefit from it's release immediately when ViteJS switches to it internally (that's their plan).

In other words, we believe ViteJS is a good bet for your bundling needs for the forseeable future.

# Acknowledgements

Many thanks to @pyrossh and the greater Rust community for the [rust-embed](https://github.com/pyrossh/rust-embed/commit/a735c9e979e6425f1553c436fe15608f98fa1780) crate which was used as a reference.

# License

This crate is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE, LICENSE-MIT, and COPYRIGHT for details.
