# Detecting a locally-running Vite dev server

_This is a thought experiment_.

Imagine a user wants to run a ViteJS dev server locally instead of having the `cargo run` manage its lifecycle. Perhaps for debugging reasons.

This one-off scenario could be supported by checking if a ViteJS dev server is running on the configured port; however, this behaviour is problematic given that a vite dev server running for another app on that port would serve unexpected content.

So, perhaps, it's not such a good idea.

The question becomes: can we confirm that the vite dev server is running for the project specified by `#[root = "path/to/project"]`?

```rust
#[vite_rs::Embed]
#[root = "path/to/project"]
struct Assets;

fn main() {
    #[cfg(debug_assertions)]
    {
        const port = Asset::get_dev_server_port();

        let expected_signature = Asset::get_dev_server_signature();

        if vite_rs_dev_server::is_vite_dev_server_running(port, expected_signature) {
            println!("Dev server is running!");
        } else {
            println!("Dev server is not running!");
        }
    }
}
```

Note: `get_dev_server_signature` and `is_vite_dev_server_running(port, signature)` are the implementation details we're interested in here.
The signature has to be something that the vite dev server can provide, and it would be unique to the project.
