import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  // set dist to upper directory
  build: {
    outDir: "../static",
    emptyOutDir: true,
    // 生产优化配置
    minify: "esbuild",
    // 代码分割优化
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["react", "react-dom"],
          ui: ["@headlessui/react", "@heroicons/react"],
          charts: ["recharts"],
        },
      },
    },
    // 构建优化
    target: "es2020",
    sourcemap: false, // 生产环境不生成sourcemap
    reportCompressedSize: false, // 跳过压缩大小报告以加快构建
    chunkSizeWarningLimit: 1000,
  },
  server: {
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8484",
        changeOrigin: true,
      },
    },
  },
  // 开发服务器优化
  esbuild: {
    target: "es2020",
  },
});
