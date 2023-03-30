import { env } from 'process'
import { defineConfig, UserConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig(async () => {
  const internal_ip = await import("internal-ip")
  const host = await internal_ip.internalIpV4()

  const config: UserConfig = {
    plugins: [vue()],
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
      target: ['es2021', 'chrome97', 'safari13'],
      minify: !env.TAURI_DEBUG ? 'esbuild' : false,
      sourcemap: !!env.TAURI_DEBUG,
      chunkSizeWarningLimit: 1000,
    },
    server: {
      fs: {
        strict: false
      },
      host: '0.0.0.0', // listen on all addresses
      port: 5173,
      strictPort: true,
      hmr: {
        protocol: 'ws',
        host,
        port: 5183,
      },
    }
  }

  return config
})
