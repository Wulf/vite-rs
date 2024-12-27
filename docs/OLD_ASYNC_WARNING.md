# Note: this is no longer the case and the warning has been removed from the main readme. We keep this checked-in for historic reasons.

# Async usage warning

**This applies to #[cfg(debug_assertions)] only:**

In development mode, we use a blocking `reqwest` client to retrieve files from the ViteJS dev server. This has the consequence that usage of `YourStruct::get("./some/asset.html")` will need to be wrapped in a `tokio::task::spawn_blocking` in async environments. See https://docs.rs/reqwest/latest/reqwest/blocking/index.html for more information. Here's the particular excerpt:

> [**Module reqwest::blocking**](https://docs.rs/reqwest/latest/reqwest/blocking/index.html)
>
> [...] If the immediate context is only synchronous, but a transitive caller is async, consider changing that caller to use [`tokio::task::spawn_blocking`](https://docs.rs/tokio/1.37.0/tokio/task/fn.spawn_blocking.html) around the calls that need to block.

### Async example (tokio)

```rust
#[vite_rs::Embed]
#[input = "./assets"]
struct Assets;

#[tokio::main]
async fn main() {
    // DEV example:
    #[cfg(debug_assertions)]
    {
        let asset = tokio::task::spawn_blocking(|| {
            Assets::get("index.html")
        }).await.unwrap();;

        println!("{}", asset);
    }


    // PROD example:
    #[cfg(not(debug_assertions))]
    {
        let asset = Assets::get("index.html");
        println!("{}", asset);
    }
}
```

### Why doesn't `vite_rs` use an async `reqwest` client and make the `::get()` fn async?

The `rust_embed` crate doesn't do this, and we wanted to keep the API consistent with that crate.
