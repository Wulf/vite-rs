import { defineConfig } from "vite";

export default defineConfig({
  build: {
    rollupOptions: {
      input: "file.txt",
    },
    manifest: true,
  },
});
