mod util;

use reqwest::StatusCode;
use tower::ServiceExt;
use vite_rs_axum_0_8::ViteServe;

use axum::{
    body::{self, Body},
    http,
};

#[derive(vite_rs::Embed)]
#[root = "test_projects/basic_usage_test/app"]
struct Assets;

/// Note: we only have a single #[test] because we can't run multiple tests in parallel
/// since the vite dev server can't be started multiple times.
#[tokio::test]
async fn test() {
    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    let _guard = Assets::start_dev_server(true);

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
        {
            Assets::stop_dev_server();
        }

        // run super's panic hook
        hook(info);
    }));

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    std::thread::sleep(std::time::Duration::from_secs(2)); // wait for dev server to start

    for app in [app_with_fallback_service, app_with_mounted_service] {
        ensure_serves_index(app()).await;
        ensure_serves_public_files(app()).await;
        ensure_serves_imports(app()).await;
    }

    test_default_cache_strategy().await; // includes `Eager` and `None` depending on build type (release/debug)
    test_lazy_cache_strategy().await;
    test_custom_cache_strategy().await;

    test_cache_response().await;
}

fn app_with_fallback_service() -> axum::Router {
    axum::Router::new().fallback_service(ViteServe::new(Assets::boxed()))
}

fn app_with_mounted_service() -> axum::Router {
    axum::Router::new()
        .route_service("/", ViteServe::new(Assets::boxed()))
        .route_service("/{*path}", ViteServe::new(Assets::boxed()))
}

async fn test_cache_response() {
    use std::borrow::BorrowMut;

    // Ensure that the cache strategy is applied correctly
    let mut app = axum::Router::new().route_service("/", ViteServe::new(Assets::boxed()));

    let req = http::Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let response = app.borrow_mut().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let etag = response
        .headers()
        .get("Etag")
        .map(|h| h.to_str().unwrap())
        .unwrap();

    let req2 = http::Request::builder()
        .uri("/")
        .header("If-None-Match", etag)
        .body(Body::empty())
        .unwrap();

    let response2 = app.oneshot(req2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::NOT_MODIFIED);
}

async fn test_custom_cache_strategy() {
    // custom cache strategy
    let app = axum::Router::new().route_service(
        "/",
        ViteServe::new(Assets::boxed())
            .with_cache_strategy(vite_rs_axum_0_8::CacheStrategy::Custom("asdf")),
    );

    let req = http::Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("Cache-Control")
            .map(|h| h.to_str().unwrap()),
        Some("asdf")
    );
}

async fn test_lazy_cache_strategy() {
    // lazy cache strategy
    let app = axum::Router::new().route_service(
        "/",
        ViteServe::new(Assets::boxed()).with_cache_strategy(vite_rs_axum_0_8::CacheStrategy::Lazy),
    );

    let req = http::Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("Cache-Control")
            .map(|h| h.to_str().unwrap()),
        Some("max-age=0, stale-while-revalidate=604800")
    );
}

async fn test_default_cache_strategy() {
    // default cache strategy is
    // - `CacheStrategy::None` in debug builds
    // - `CacheStrategy::Eager` in release builds
    let app = axum::Router::new().route_service("/", ViteServe::new(Assets::boxed()));

    //
    // 1. See if the server requests the right cache strategy
    //

    let request = http::Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Ensure that the default cache strategy is `CacheStrategy::None`
    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    assert_eq!(
        response
            .headers()
            .get("Cache-Control")
            .map(|h| h.to_str().unwrap()),
        Some("no-cache")
    );

    #[cfg(not(all(debug_assertions, not(feature = "debug-prod"))))]
    // Ensure that the default cache strategy is `CacheStrategy::Eager`
    assert_eq!(
        response
            .headers()
            .get("Cache-Control")
            .map(|h| h.to_str().unwrap()),
        Some("max-age=0, must-revalidate")
    );
}

async fn ensure_serves_index(app: axum::Router) {
    let request = http::Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), 2048).await.unwrap();

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    assert_eq!(body_bytes, "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <script type=\"module\">import { injectIntoGlobalHook } from \"/@react-refresh\";\ninjectIntoGlobalHook(window);\nwindow.$RefreshReg$ = () => {};\nwindow.$RefreshSig$ = () => (type) => type;</script>\n\n    <script type=\"module\" src=\"/@vite/client\"></script>\n\n    <title>Hello World</title>\n    <link rel=\"stylesheet\" href=\"/test.css\" />\n  </head>\n  <body>\n    <h1>Loading...</h1>\n    <script type=\"module\" src=\"/script.tsx\"></script>\n  </body>\n</html>\n");

    #[cfg(not(all(debug_assertions, not(feature = "debug-prod"))))]
    assert_eq!(body_bytes, "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <title>Hello World</title>\n    <link rel=\"stylesheet\" href=\"/test.css\" />\n    <script type=\"module\" crossorigin src=\"/assets/index-CgRBhnJL.js\"></script>\n  </head>\n  <body>\n    <h1>Loading...</h1>\n  </body>\n</html>\n");
}

async fn ensure_serves_public_files(app: axum::Router) {
    let request = http::Request::builder()
        .uri("/test.css")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), 2048).await.unwrap();

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    assert_eq!(body_bytes, "body {\n  background-color: black;\n  color: white;\n  font-family: Arial, sans-serif;\n  padding: 42px;\n}\n");

    #[cfg(not(all(debug_assertions, not(feature = "debug-prod"))))]
    assert_eq!(body_bytes, "body {\n  background-color: black;\n  color: white;\n  font-family: Arial, sans-serif;\n  padding: 42px;\n}\n");
}

async fn ensure_serves_imports(app: axum::Router) {
    let uri = if cfg!(all(debug_assertions, not(feature = "debug-prod"))) {
        "/script.tsx"
    } else {
        "/assets/index-CgRBhnJL.js"
    };

    let request = http::Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), 262144).await.unwrap();

    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
    assert!(body_bytes.starts_with(b"import __vite__cjsImport0_react_jsxDevRuntime from \"/node_modules/.vite/deps/react_jsx-dev-runtime.js?v="));

    #[cfg(not(all(debug_assertions, not(feature = "debug-prod"))))]
    assert!(body_bytes.starts_with(b"(function(){const vl=document.createElement(\"link\").relList;if(vl&&vl.supports&&vl.supports(\"modulepreload\"))return;for(const Q of document.querySelectorAll('link[rel=\"modulepreload\"]'))r(Q);new MutationObserver(Q=>{for(const L of Q)if(L.type===\"childList\")for(const tl of L.addedNodes)tl.tagName===\"LINK\"&&tl.rel===\"modulepreload\"&&r(tl)}).observe(document,{childList:!0,subtree:!0});function J(Q){const L={};return Q.integrity&&(L.integrity=Q.integrity),Q.referrerPolicy&&(L.referrerPolicy=Q.referrerPolicy),Q.crossOrigin===\"use-credentials\"?L.credentials=\"include\":Q.crossOrigin===\"anonymous\"?L.credentials=\"omit\":L.credentials=\"same-origin\",L}function r(Q){if(Q.ep)return;Q.ep=!0;const L=J(Q);fetch(Q.href,L)}})();const R1=\"modulepreload\",H1=function(_){return\"/\"+_},wv={},N1=function(vl,J,r){let Q=Promise.resolve();if(J&&J.length>0){let tl=function(T){return Promise.all(T.map(U=>Promise.resolve(U).then(k=>({status:\"fulfilled\",value:k}),k=>({status:\"rejected\",reason:k}))))};document.getElementsByTagName(\"link\""));
}
