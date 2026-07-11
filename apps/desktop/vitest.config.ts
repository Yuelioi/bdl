import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [vue()],
  test: {
    exclude: ['tests/visual/**', 'node_modules/**', 'dist/**'],
    environment: 'happy-dom',
    clearMocks: true,
    restoreMocks: true,
  },
})
