# Tauri V2 文件管理器 (FileManger)

一个使用 Tauri V2、Rust 和 Vue 3 构建的跨平台文件管理器概念验证项目。

## 目标

创建一个简单但功能齐全的文件管理器组件或应用，能够：

*   跨平台运行 (主要验证 Windows，考虑 Android/iOS)。
*   浏览文件和文件夹。
*   导航：进入子文件夹、返回上一级。
*   显示文件/文件夹的基本信息（名称、类型、大小、只读属性）。
*   为不同类型的文件和文件夹（包括隐藏文件夹）显示图标。
*   按类型（文件夹优先）和名称（支持中文）排序。
*   (未来) 支持文件操作（创建、删除、重命名等）。
*   (未来) 支持多选。

## 技术栈

*   **核心框架**: [Tauri v2](https://beta.tauri.app/) - 使用 Rust 构建跨平台应用的框架。
*   **后端**: Rust - 处理文件系统交互、获取初始路径等。
*   **前端**: [Vue 3](https://vuejs.org/) (Composition API with `<script setup>`) - 构建用户界面。
*   **构建工具**: [Vite](https://vitejs.dev/) - 前端开发服务器和构建工具。
*   **包管理器**: npm

## 当前功能

*   **目录列表**: 显示指定路径下的文件和文件夹。
*   **导航**:
    *   双击文件夹进入。
    *   点击 "向上" 按钮返回上一级目录。
*   **信息展示**:
    *   名称 (带图标)。
    *   类型 (根据文件扩展名生成的用户友好描述)。
    *   大小 (格式化为 B, KB, MB, GB)。
    *   属性 (只读标记 'R')。
*   **图标**:
    *   区分文件夹和文件。
    *   隐藏文件夹 (名称以 `.` 开头) 显示为半透明。
    *   为常见文件类型（图片、音视频、代码、压缩包、PDF等）显示不同图标。
*   **排序**: 列表默认按 文件夹优先 -> 文件 -> 按名称 (支持中文拼音及数字) 升序排列。
*   **初始路径**: 应用启动时自动获取并加载用户可执行文件所在的目录。

## 如何运行

**环境要求**:

1.  **Node.js**: [下载并安装](https://nodejs.org/) (自带 npm)。
2.  **Rust**: [安装 Rust](https://www.rust-lang.org/tools/install)。
3.  **Tauri 依赖**: 根据你的操作系统，参照 [Tauri v2 Prerequisites](https://beta.tauri.app/start/prerequisites/) 安装必要的系统依赖（如 C++ 构建工具、WebView2 等）。

**步骤**:

1.  **克隆或下载仓库**:
    ```bash
    git clone <repository-url>
    cd FileManger
    ```
2.  **安装 Node.js 依赖**:
    ```bash
    npm install
    ```
3.  **运行开发环境**:
    ```bash
    npm run tauri dev
    ```
    这将启动 Vite 开发服务器和 Tauri 应用窗口。

## 开发过程中的关键点与学习

*   **Tauri v2 API 导入**: Tauri v2 中，前端调用后端命令需要从 `@tauri-apps/api/core` 导入 `invoke`，而不是 v1 的 `@tauri-apps/api/tauri` 或全局 `window.__TAURI__` 对象。
*   **Vite 与 Tauri 集成**:
    *   当使用正确的 API 导入路径 (`@tauri-apps/api/core`) 时，通常**不需要**在 `vite.config.js` 的 `build.rollupOptions.external` 中排除 `@tauri-apps/api`。错误地添加 `external` 可能导致构建成功但运行时 `Failed to resolve module specifier` 错误。
    *   需要配置 `tauri.conf.json` 中的 `frontendDist` 指向 Vite 的输出目录（默认为 `dist`），并确保移除或注释掉 `devUrl` 以使用 Tauri 的内建开发服务器（如果 `beforeDevCommand` 设置为构建命令如 `"npm run build"`）。如果使用 Vite 的开发服务器，则需设置 `devUrl` 指向 Vite 启动的地址 (如 `http://localhost:5173`) 并将 `beforeDevCommand` 设置为 `"npm run dev"` 或类似的启动命令。本项目目前使用 Vite 开发服务器。
*   **初始路径**: 获取可靠的跨平台初始路径比较复杂。`env::current_exe().parent()` 提供了一个相对稳定的起点（程序可执行文件所在目录），比单纯的 `home_dir` 或 `app_data_dir` 更适合此场景。
*   **文件列表排序**: 使用 JavaScript 的 `Array.prototype.sort` 配合 `String.prototype.localeCompare('zh-CN', { sensitivity: 'base', numeric: true })` 可以实现健壮的、支持中文拼音和数字的自然排序。
*   **隐藏文件/文件夹**: 在无法直接从所有平台 reliably 获取 "hidden" 属性时，通过检查名称是否以 `.` 开头是一种常见的跨平台近似处理方式，适用于 Linux, macOS 和 Git 仓库。

## 未来可能的改进

*   实现真正的文件隐藏属性检测 (需要 Rust 后端针对不同平台处理)。
*   添加文件/文件夹操作：创建、删除、重名、复制、粘贴。
*   实现多选功能。
*   添加右键菜单。
*   优化大文件/目录的加载性能。
*   更完善的错误处理和用户反馈。
*   打包和测试 Android/iOS 目标。
*   添加更多文件类型的图标和预览。
*   实现可配置的排序方式（按大小、修改日期等）。
