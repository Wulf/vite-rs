import { defineConfig } from "vite";

export default defineConfig({
  build: {
    rollupOptions: {
      input: "file.txt",
    },
    outDir: "./custom-output-dir/dist",
    manifest: true,
  },
});
