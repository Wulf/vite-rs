import { defineConfig } from "vite";

export default defineConfig({
  build: {
    rollupOptions: {
      input: { bundle: "./script.js" },
    },
    manifest: true,
  },
  publicDir: "public",
});
