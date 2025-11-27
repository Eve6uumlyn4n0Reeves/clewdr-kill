import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react-swc";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest.setup.ts"],
    coverage: {
      provider: "c8",
      reportsDirectory: "./coverage",
      lines: 70,
      functions: 70,
      statements: 70,
      branches: 60,
      reporter: ["text", "html"],
    },
  },
});
