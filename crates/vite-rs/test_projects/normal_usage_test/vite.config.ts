import { defineConfig } from "vite";

export default defineConfig({
  build: {
    rollupOptions: {
      input: ["app/index.html", "app/pack1.ts"],
    },
    manifest: true, // **IMPORTANT**: this is required.
    outDir: "./dist", // this is the default value
  },
  publicDir: "./public", // this is the default value
});
