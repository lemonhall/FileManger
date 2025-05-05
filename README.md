# Vue 3 + Vite

This template should help get you started developing with Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

Learn more about IDE Support for Vue in the [Vue Docs Scaling up Guide](https://vuejs.org/guide/scaling-up/tooling.html#ide-support).

## ⚙️ 开发经验与注意事项 (补充)

在配置和调试此项目（或类似的 Tauri v2 + Vite 项目）时，遇到了一些关于 API 导入和构建配置的关键问题，总结如下：

1.  **Tauri v2 API 导入路径是关键**: 与 Tauri v1 不同，v2 将 API 拆分到了子模块。核心功能（如 `invoke`）必须从 `@tauri-apps/api/core` 导入，事件监听（如 `listen`）从 `@tauri-apps/api/event` 导入。使用旧的路径（如 `@tauri-apps/api/tauri`）会导致构建失败或运行时错误。

2.  **Vite 构建 (`vite build`) 与 `external` 配置**: 
    *   当使用 Tauri 内建服务器模式（通过 `beforeDevCommand: "npm run build"` 触发 `vite build`）时，如果 API 导入路径**不正确**，构建会失败。此时强行在 `vite.config.js` 中将 `@tauri-apps/api` 设为 `external` 可以**绕过构建错误**，但会导致**运行时**浏览器无法解析模块而报错。
    *   **正确的做法是确保 API 导入路径正确 (`@tauri-apps/api/core` 等)，并移除 `vite.config.js` 中的 `external: [/^@tauri-apps\/api($|\/)/]` 配置。** 这样 `vite build` 可以正常处理导入，运行时也不会出错。

3.  **Vue 集成**: 最初的复杂性部分源于错误的 API 导入路径。确认路径正确后，标准的 Vue + Vite 配置（使用 `@vitejs/plugin-vue`，不修改 `root`，`outDir` 指向 `dist`，移除 `external`）可以与 Tauri v2 良好协作。

4.  **获取初始路径**: 在 Rust 后端使用 `std::env::current_exe()` 获取程序运行目录，或结合 `app.path().home_dir()` / `app.path().app_data_dir()` 来动态确定一个比硬编码更合适的初始文件列表路径，可以提高跨平台兼容性。

希望这些记录能帮助后续开发避免重蹈覆辙。
