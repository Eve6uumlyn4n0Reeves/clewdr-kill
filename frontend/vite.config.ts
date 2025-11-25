import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  // set dist to upper directory
  build: {
    outDir: "../static",
    emptyOutDir: true,
  },
});
