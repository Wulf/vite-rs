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

       axum::serve(
           tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap(),
           axum::Router::new()
               // we mount our Vite app to /
               .route_service("/", ViteServe::new(Assets::boxed()))
               .route_service("/{*path}", ViteServe::new(Assets::boxed()))
               // we can also use a fallback service instead:
               // .fallback_service(ViteServe::new(Assets::boxed()))
               .into_make_service(),
       )
       .await
       .unwrap()
   }
   ```

## Single-Page Application (SPA) Routing

In the default fallback strategy (`FallbackStrategy::NotFound`), requests for paths that don't match an embedded asset return `404` with an empty body ([caveat: ViteJS dev server responds 200 in all cases](#vitejs-invalid-path-response)). To let a client-side router (e.g. React Router, Vue Router) handle those paths instead, use `FallbackStrategy::SinglePageApplication`. The right setup depends on how you mount the `ViteServe` service.

#### Option 1: Render SPA as a catch-all/fallback route

The simplest setup: unmatched paths are already forwarded to `ViteServe` by Axum, so only the strategy needs to change.

```diff
use vite_rs_axum_0_8::{FallbackStrategy, ViteServe};

axum::Router::new()
    // ... your other routes ...
+    .fallback_service(
+        ViteServe::new(Assets::boxed())
+            .with_fallback_strategy(FallbackStrategy::SinglePageApplication("index.html".into()))
+    )
```


#### Option 2: Render SPA at a specific path

It's required to use a two-route setup because `/` handles the root and `/{*path}` captures every deeper path.

```diff
use vite_rs_axum_0_8::{FallbackStrategy, ViteServe};

+let spa = ViteServe::new(Assets::boxed())
+    .with_fallback_strategy(FallbackStrategy::SinglePageApplication("index.html".into()));

axum::Router::new()
+    .route_service("/", spa.clone())
+    .route_service("/{*path}", spa)
```

### Routing Response

Any request that doesn't match an embedded asset is served `index.html` with a `200` response. If the named fallback file is not present in the asset map, the response falls back to `404`. 

<a name="vitejs-invalid-path-response"></a>It should be noted that, in development, the ViteJS dev server serves the entrypoint (`index.html` by default) even at paths where resources don't exist.

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
