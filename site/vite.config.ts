import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  // Le cœur partagé est consommé en TypeScript, sans étape de compilation.
  resolve: {
    alias: {
      // L'ordre compte : le point d'entrée le plus précis d'abord.
      '@klima/core/ui': fileURLToPath(new URL('../core/src/ui/index.ts', import.meta.url)),
      '@klima/core': fileURLToPath(new URL('../core/src/index.ts', import.meta.url)),
    },
  },
  server: { port: 5174 },
  build: { outDir: 'dist', sourcemap: true },
})
