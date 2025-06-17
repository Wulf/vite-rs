use axum::Router;
use std::panic;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::broadcast;

use vite_rs_axum_0_8::ViteServe;

#[derive(vite_rs::Embed)]
#[root = "./app"]
struct Assets;

#[tokio::main]
async fn main() {
    // Set up panic hook to ensure dev server is stopped even in case of panic
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        println!("Server panic occurred: {}", panic_info);

        #[cfg(debug_assertions)]
        Assets::stop_dev_server();

        // Call the default panic hook
        default_hook(panic_info);
    }));

    // Shutdown signal
    let (tx_for_ctrl_c, rx) = broadcast::channel::<()>(1);
    let tx_for_error = tx_for_ctrl_c.clone();

    // Use an atomic to track if we've already initiated shutdown
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();

    #[cfg(debug_assertions)]
    let _guard = Assets::start_dev_server();

    println!("GET / http://localhost:3000/");

    // Set up Ctrl+C handler before starting the server
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, shutting down...");

        // Only initiate shutdown once
        if !shutdown_flag.load(Ordering::SeqCst) {
            shutdown_flag.store(true, Ordering::SeqCst);

            #[cfg(debug_assertions)]
            Assets::stop_dev_server();

            let _ = tx_for_ctrl_c.send(());
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Start the server with graceful shutdown
    println!("Server started. Press Ctrl+C to stop.");

    // Use tokio::spawn with a Result handling to catch potential panics
    let server_handle = tokio::spawn(async move {
        // Create a receiver for the shutdown signal
        let mut rx = rx;

        let server = axum::serve(
            tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
            Router::new()
                .route_service("/", ViteServe::new(Assets::boxed()))
                .route_service("/{*path}", ViteServe::new(Assets::boxed()))
                .into_make_service(),
        )
        .with_graceful_shutdown(async move {
            // Wait for the shutdown signal
            let _ = rx.recv().await;
            println!("Server shutdown complete");
        });

        server.await
    });

    // Wait for the server to complete - this will block until the server is shutdown
    // via the Ctrl+C handler or if a panic occurs
    match server_handle.await {
        Ok(server_result) => {
            if let Err(e) = server_result {
                eprintln!("Server error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Server task failed: {}", e);

            // In case the server task panicked, make sure dev server is stopped
            if !shutdown_flag_clone.load(Ordering::SeqCst) {
                shutdown_flag_clone.store(true, Ordering::SeqCst);
                #[cfg(debug_assertions)]
                Assets::stop_dev_server();

                // Signal shutdown in case something is still waiting
                let _ = tx_for_error.send(());
            }
        }
    }

    println!("Exiting application");
}
