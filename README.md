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

## 前端代码结构

为了更好地组织和维护前端代码，项目采用了以下结构和模块化方式：

*   **核心组件**:
    *   `src/components/FileManager.vue`: 作为应用的顶层协调组件，负责整合 `Toolbar`、`FileList` 和 `SettingsModal`。它负责初始化组合式函数 (`useFileSystem`, `useBaiduNetdisk`)，响应来自子组件的事件（如导航、文件操作触发、打开设置），并调用组合式函数执行相应的逻辑。
    *   `src/components/Toolbar.vue`: 负责展示文件管理器的工具栏，包括当前路径显示、向上导航按钮、同步到网盘按钮以及打开设置的按钮。它通过 props 接收状态（如当前路径、是否能向上、是否能同步），并通过 emits 将用户操作（如点击按钮）通知给父组件 `FileManager.vue`。
    *   `src/components/FileList.vue`: 负责展示文件和文件夹列表。它接收文件数据 (`items`)、加载状态 (`loading`) 和错误信息 (`error`) 作为 props。它内部使用 `FileListItem.vue` 来渲染每一行，并处理用户的交互事件（如双击打开、勾选文件/文件夹），通过 emits 将这些事件冒泡给父组件 `FileManager.vue`。
    *   `src/components/FileListItem.vue`: 负责渲染文件列表中的**单行**。它接收单个文件/文件夹 `item` 作为 prop，展示其图标、名称、类型、大小等信息，并处理该行的双击和复选框状态变化事件，通过 emit 通知父组件。
    *   `src/components/SettingsModal.vue`: 一个独立的Vue组件，专门用于处理百度网盘的 Access Token 配置。用户可以在此弹窗中输入、保存 Access Token。它内部使用 `useBaiduNetdisk` 组合式函数来获取和设置 Token，并调用该 composable 中的方法来显示相关的用户和配额信息。它通过 props (`show`) 控制显隐，并通过 emits (`close`) 通知父组件关闭。

*   **组合式函数 (Composables)**:
    *   `src/composables/useFileSystem.js`: 封装了核心的文件系统浏览逻辑。它管理着当前路径 (`currentPath`)、文件/文件夹列表 (`items`)、加载状态 (`loading`) 和错误状态 (`error`)。提供了获取初始路径、列出目录内容、向上导航和打开目录的方法，供 `FileManager.vue` 调用。
    *   `src/composables/useBaiduNetdisk.js`: 封装了所有与百度网盘 API 交互的逻辑。它内部管理 Access Token（从 `localStorage` 初始化，并提供方法进行更新），提供判断网盘服务是否可用的状态 (`isAvailable`)。它还包含了获取用户信息、查询存储配额，以及将多个文件同步（上传）到网盘指定目录等核心功能。此 composable 被 `FileManager.vue` 用于触发同步操作，被 `SettingsModal.vue` 用于 Token 管理和信息展示。

*   **工具模块 (Utils)**:
    *   `src/utils/icons.js`: 此模块集中管理和导出应用中用到的所有SVG图标字符串，供 `FileListItem.vue` 使用。
    *   `src/utils/fileTypes.js`: 定义了文件扩展名到用户友好的文件类型描述的映射表，供 `FileListItem.vue` 使用。
    *   `src/utils/formatters.js`: 包含通用的格式化函数，例如将文件大小（字节）格式化为 KB/MB/GB (`formatSize`)，或将 VIP 类型数字转换为可读字符串 (`vipTypeToString`)，供 `FileListItem.vue` 和 `SettingsModal.vue` 使用。

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
*   **百度网盘上传 (实验性)**:
    *   通过界面右上角的齿轮设置图标，可以配置百度网盘的 Access Token (存储在浏览器的 localStorage 中)。
    *   在文件列表中勾选一个或多个文件后，点击工具栏上的 "同步到网盘" 按钮。
    *   选中的文件将被上传到百度网盘的固定目录 (目前为 `/来自FileManger同步`)。
    *   Rust 后端处理实际的文件上传逻辑，包括对大于 4MB 的文件进行分片上传和 MD5 校验。
    *   **上传时间戳记录**:
        *   应用会自动记录每个文件成功同步到百度网盘的时间。
        *   此信息显示在文件列表的 "上次上传" 列中，格式为 `YYYY-MM-DD HH:mm`。
        *   时间戳数据持久化存储在用户操作系统的标准应用数据目录下的 `upload_timestamps.json` 文件中。该文件内部使用文件的绝对路径作为键来关联时间戳。
        *   该功能为未来实现增量同步（例如，比较本地文件最后修改时间与记录的上次上传时间）提供了基础。

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

    **注意**: 如果要测试百度网盘上传功能，请先通过应用内的设置界面配置有效的 Access Token。

## 开发过程中的关键点与学习

*   **Tauri v2 API 导入**: Tauri v2 中，前端调用后端命令需要从 `@tauri-apps/api/core` 导入 `invoke`，而不是 v1 的 `@tauri-apps/api/tauri` 或全局 `window.__TAURI__` 对象。
*   **Vite 与 Tauri 集成**:
    *   当使用正确的 API 导入路径 (`@tauri-apps/api/core`) 时，通常**不需要**在 `vite.config.js` 的 `build.rollupOptions.external` 中排除 `@tauri-apps/api`。错误地添加 `external` 可能导致构建成功但运行时 `Failed to resolve module specifier` 错误。
    *   需要配置 `tauri.conf.json` 中的 `frontendDist` 指向 Vite 的输出目录（默认为 `dist`），并确保移除或注释掉 `devUrl` 以使用 Tauri 的内建开发服务器（如果 `beforeDevCommand` 设置为构建命令如 `"npm run build"`）。如果使用 Vite 的开发服务器，则需设置 `devUrl` 指向 Vite 启动的地址 (如 `http://localhost:5173`) 并将 `beforeDevCommand` 设置为 `"npm run dev"` 或类似的启动命令。本项目目前使用 Vite 开发服务器。
*   **初始路径**: 获取可靠的跨平台初始路径比较复杂。`env::current_exe().parent()` 提供了一个相对稳定的起点（程序可执行文件所在目录），比单纯的 `home_dir` 或 `app_data_dir` 更适合此场景。
*   **文件列表排序**: 使用 JavaScript 的 `Array.prototype.sort` 配合 `String.prototype.localeCompare('zh-CN', { sensitivity: 'base', numeric: true })` 可以实现健壮的、支持中文拼音和数字的自然排序。
*   **隐藏文件/文件夹**: 在无法直接从所有平台 reliably 获取 "hidden" 属性时，通过检查名称是否以 `.` 开头是一种常见的跨平台近似处理方式，适用于 Linux, macOS 和 Git 仓库。
*   **百度网盘集成 (Rust + Vue)**:
    *   **后端 (Rust)**: 使用 `reqwest` crate 进行 HTTP API 调用，`md5` crate 进行校验和计算。实现了文件分片上传逻辑，区分处理大小文件，遵循百度网盘的上传流程（预创建、分片上传、创建文件）。Tauri command (`upload_file_to_baidupan`) 接收前端传递的 Access Token 和文件信息。
    *   **前端 (Vue)**: 设计了设置模态框，允许用户输入并保存 Access Token 到浏览器的 `localStorage`。文件列表支持多选，通过 "同步到网盘" 按钮触发上传流程，调用 Rust 后端命令。
    *   **Access Token 管理**: 为了方便测试和基本可用性，Access Token 通过前端界面配置并存储在 `localStorage`。在生产环境中，可能需要考虑更安全的 Token 存储和管理机制。
    *   **上传状态持久化**: 为了记录文件的上次上传时间，项目通过 Rust 后端在应用数据目录中维护一个 `upload_timestamps.json` 文件。前端通过 Tauri 命令调用相应接口进行读写，并通过 Vue Composable (`useUploadTimestamps.js`) 进行状态管理和UI更新。这为后续实现更智能的同步策略（如基于修改时间判断是否需要同步）奠定了基础。

## 未来可能的改进

*   实现真正的文件隐藏属性检测 (需要 Rust 后端针对不同平台处理)。
*   添加文件/文件夹操作：创建、删除、重名、复制、粘贴。
*   实现多选功能。
*   添加右键菜单。
*   优化大文件/目录的加载性能。
*   更完善的错误处理和用户反馈 (例如，上传进度、Toast 通知替代 alert)。
*   打包和测试 Android/iOS 目标。
*   添加更多文件类型的图标和预览。
*   实现可配置的排序方式（按大小、修改日期等）。
*   **百度网盘功能增强**:
    *   允许用户配置上传到网盘的指定目录。
    *   实现从网盘下载文件或同步文件夹的功能。
    *   更稳健的并发上传和错误重试机制。
    *   上传进度显示。
