import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { splitChunksPlugin } from './plugin'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    splitChunksPlugin({
      minChunks: 2,
      minSize: 50,
      maxSize: 500
    }),
    react()
  ],
  build: {
    minify: false
  }
})
