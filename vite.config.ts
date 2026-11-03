import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Tauri 开发时固定端口，strictPort 保证端口被占用时直接报错而不是静默换端口
// （换了端口 tauri.conf.json 的 devUrl 就对不上了）。
export default defineConfig({
  plugins: [vue()],

  // Tauri CLI 需要看到这些前缀的环境变量
  envPrefix: ["VITE_", "TAURI_"],

  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Rust 源码变更由 tauri dev 自己监听，避免 vite 重复触发
      ignored: ["**/src-tauri/**"],
    },
  },

  build: {
    // Windows 上 Tauri 用 WebView2，基线是 Chromium 105
    target: "chrome105",
    // 开发时保留可读的变量名，便于在 WebView2 devtools 里调试
    minify: process.env.TAURI_ENV_DEBUG ? false : "esbuild",
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
