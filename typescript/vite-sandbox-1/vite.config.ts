import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import legacy from '@vitejs/plugin-legacy'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({
      plugins: [
        [
          "@swc/plugin-remove-console", {
          }
        ]
      ]
    }),
    legacy({
      modernTargets: 'last 2 years',
      // modernPolyfills: true,
      // renderLegacyChunks: false
    })
  ],
  build: {
    minify: false,
    target: ["es2020"]
  }
})
