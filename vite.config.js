import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import adapter from "@sveltejs/adapter-static";
import process from "node:process";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(() => ({
  // Tauri has no Node server: adapter-static with an index.html fallback runs the app as an SPA.
  plugins: [sveltekit({ adapter: adapter({ fallback: "index.html" }) })],
  // Keep Rust errors visible and use the fixed port Tauri expects.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  // Bundle every dependency up front. Otherwise Vite discovers some (CodeMirror, Tauri plugins)
  // only when a page first imports them, re-optimizes mid-session and answers the open page with
  // "504 Outdated Optimize Dep", which SvelteKit shows as an internal error.
  optimizeDeps: {
    include: [
      "@codemirror/commands",
      "@codemirror/lang-markdown",
      "@codemirror/language",
      "@codemirror/state",
      "@codemirror/view",
      "@lezer/highlight",
      "@tauri-apps/api/core",
      "@tauri-apps/api/event",
      "@tauri-apps/api/mocks",
      "@tauri-apps/api/webview",
      "@tauri-apps/api/window",
      "@tauri-apps/plugin-dialog",
      "@tauri-apps/plugin-notification",
      "@tauri-apps/plugin-opener",
      "dompurify",
      "marked",
    ],
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
}));
