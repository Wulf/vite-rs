# Note: this was moved to 'unsupported'. At the moment, we can't rename bundles in development mode since that would require parsing the vite config file. Currently, this crate attempts to be low-touch and tries to avoid relying on the vite configuration.

# Renaming Bundles

When the vite config has a hash for rollupOptions.input, the `Asset::get()` method should use the key used as the entrypoint name.

**Why?** This is useful when dealing with naming conflicts between entrypoints and public files served from the `viteConfig.publicDir` option.

# Example

### Directory structure

```
.
├── public
│   └── script.js
├── script.js          // name clash with public/script.js
└── vite.config.ts
```

### `vite.config.ts`

```typescript
import { defineConfig } from "vite";

export default defineConfig({
  build: {
    rollupOptions: {
      input: { bundle: "./script.js" },
    },
    manifest: true,
  },
  publicDir: "./public",
});
```

### Rust usage

```rust
#[vite_rs::Embed]
#[root = "./tests/renaming_bundles"]
struct Assets;

fn main() {
    let bundle = Assets::get("bundle.js");
    let script = Assets::get("script.js");

    // ...
}

```
