import vue from '@vitejs/plugin-vue'
import ui from '@nuxt/ui/vite'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [
    vue(),
    ui({
      theme: {
        colors: ['primary', 'success', 'warning', 'error', 'neutral'],
        defaultVariants: {
          color: 'primary',
          size: 'sm',
        },
      },
    }),
  ],
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
  },
})
