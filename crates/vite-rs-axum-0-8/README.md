# Axum integration for `vite-rs`

This crate provides a Tower service that can be used in Axum projects to serve your embedded ViteJS assets.

The exposed `ViteServe` service can be used as a fallback service or mounted at a specific path in your Axum router.

## Quick Start

1. Add dependencies:

   ```sh
   cargo add vite-rs
   cargo add vite-rs-axum-0-8
   cargo add axum@0.8
   cargo add tokio --features macros,rt-multi-thread
   ```

2. Create a Vite project in `./app` (it should contain a `vite.config.js` file). For help, refer to the Quick Start section in the `vite-rs` README.

3. Update your binary:

   ```rs
   // src/main.rs
   use vite_rs_axum_0_8::ViteServe;

   #[derive(vite_rs::Embed)]
   #[root = "./app"]
   struct Assets;

   #[tokio::main]
   async fn main() {
       #[cfg(debug_assertions)]
       let _guard = Assets::start_dev_server(true);

       println!("Starting server on http://localhost:3000");

       let _ = axum::serve(
           tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap(),
           axum::Router::new()
               // we mount our Vite app to /
               .route_service("/", ViteServe::new(Assets::boxed()))
               .route_service("/{*path}", ViteServe::new(Assets::boxed()))
               // we can also use a fallback service instead:
               // .fallback_service(ViteServe::new(Assets::boxed()))
               .into_make_service(),
       )
       .await;
   }
   ```

## HTTP Caching Behaviour

See [CacheStrategy rust docs](https://docs.rs/vite-rs-axum-0-8?search=CacheStrategy) for details on the caching strategies available. By default, release builds use the `Eager` caching strategy, while debug builds use `None`. You can override this by explicitly setting the cache strategy. Use them as follows:

```diff
use vite_rs_axum_0_8::{CacheStrategy, ViteServe};

#[derive(vite_rs::Embed)]
#[root = "./app"]
struct Assets;

fn main() {
    let service = ViteServe::new(Assets::boxed())
+        .with_cache_strategy(CacheStrategy::Eager);
}
```

## Graceful shutdown

It's recommended to use `test_projects/ctrl_c_handling_test` as a reference in setting up your server binary. This will help you gracefully handle Ctrl-C and other signals in unix when managing the ViteJS dev server in Rust. Alternatively, manage the dev server lifecycle yourself (refer to `vite-rs` crate docs), and use Axum's graceful shutdown example instead.

**If any of this is overwhelming**, use the quick start above, and kill your dev server with `killall node` if you find your dev server has not shutdown properly. Although not necessary, you can add a bit more more robustness by adding a panic hook handler after your dev server starts (this will ensure your dev server is stopped on unlikely panics outside axum handlers):

```rs
#[cfg(debug_assertions)]
let _guard = Assets::start_dev_server(true);

#[cfg(debug_assertions)]
{
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        Assets::stop_dev_server();

        // run super's panic hook
        hook(info);
    }));
}
```
