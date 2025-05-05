import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue' // Add back Vue plugin import

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()], // Add back Vue plugin usage

  // 防止 Vite 清空 dist 目录，因为 Tauri 可能在那里放置其他内容
  clearScreen: false,
  // Tauri 使用固定端口进行开发，并启用严格端口检查
  server: {
    port: 1420,
    strictPort: true,
  },
  // 配置环境变量前缀，使其与 Tauri 兼容
  envPrefix: ['VITE_', 'TAURI_'],
  // 添加 optimizeDeps 配置
  optimizeDeps: {
    exclude: ['@tauri-apps/api'] // Keep exclude
  },
  build: {
    // Tauri 支持的应用目标浏览器
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // 在调试构建中生成 sourcemap
    sourcemap: !!process.env.TAURI_DEBUG,
    // 指定输出目录，相对于 root ('dist')，所以 '../dist_vite_build' 指向项目根目录的 'dist_vite_build'
    outDir: 'dist',
    // 构建时清空输出目录
    emptyOutDir: true,
    // 禁用 rollup 输出警告
    rollupOptions: {
      output: {
        // 保留固定文件名输出 (如果需要)
        entryFileNames: `assets/[name].js`,
        chunkFileNames: `assets/[name].js`,
        assetFileNames: `assets/[name].[ext]`
      }
    },
  },
})
