# Note: this is no longer the case since ViteJS's default emptyOutDir option is set to true, we assume it's safe to create the directory for you.

> `Output directory 'path/to/dist' does not exist. Please create it. rust-analyzer macro-error`

For release builds, this library requires you to manually create a dist folder (where the compiled assets are outputted by Vite). You should make this directory in your CI steps and also locally if you do --release builds.
