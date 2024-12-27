import { defineConfig } from "vite";
import path from "path";
import { globSync } from "glob";

export default defineConfig(() => ({
  build: {
    rollupOptions: {
      input: globSync(path.resolve(__dirname, "*.txt")),
    },
    manifest: true,
  },
}));
