use axum::Router;
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
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        Router::new()
            .route_service("/", ViteServe::new(Assets::boxed()))
            .route_service("/{*path}", ViteServe::new(Assets::boxed()))
            .into_make_service(),
    )
    .await;

    // see `crates/vite-rs-axum-0-8/test_projects/ctrl_c_handling_test` for an example on how to handle graceful shutdown!
}
