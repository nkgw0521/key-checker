import { defineConfig } from "vite";

export default defineConfig({
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // node_modules と Rust のビルド成果物を監視対象から除外
      ignored: ["**/node_modules/**", "**/src-tauri/target/**"],
    },
  },
  build: {
    outDir: "dist",
  },
});
