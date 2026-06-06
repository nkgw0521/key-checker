import { defineConfig } from "vite";

export default defineConfig({
  // Tauri が期待するポート
  server: {
    port: 1420,
    strictPort: true,
  },
  // 本番ビルド出力先
  build: {
    outDir: "dist",
  },
});
