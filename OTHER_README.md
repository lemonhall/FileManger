# 阿里云实时语音识别 (ASR) 应用 (Tauri + Rust + Vue)

这是一个使用 Tauri 框架构建的跨平台桌面和 Android 应用，集成了阿里云 Gummy 实时语音识别服务，提供从麦克风捕获音频、实时发送至云端、接收并展示识别和翻译（可选）结果的功能。

## 🎯 主要功能

- **实时语音识别**: 通过 WebSocket 连接阿里云 Gummy ASR 服务，实时处理音频流。
- **结果展示**: 在界面上实时展示识别出的文本结果。
- **可选翻译**: 支持配置是否启用实时翻译功能（目前示例目标语言为英语）。
- **音频可视化**: 提供简单的音频波形可视化效果。
- **状态反馈**: 清晰展示应用的连接状态、任务状态和错误信息。
- **跨平台**: 基于 Tauri，可打包为 Windows, macOS, Linux 桌面应用及 Android 应用。

## 📱 设备兼容

- **桌面端 (PC/Mac/Linux)**: 提供完整的桌面应用体验。
- **手机端 (Android)**: 优化适配 Android 设备，利用原生权限进行麦克风访问。

## 🛠️ 技术实现

- **核心框架**: [Tauri](https://tauri.app/) (v2) - 使用 Web 技术构建跨平台应用的框架。
- **前端**: 使用 Vue.js (或根据实际情况修改) 和原生 JavaScript/CSS 构建用户界面。
- **后端 (Tauri Core)**: 使用 Rust 语言编写，遵循模块化设计：
    - `src/lib.rs`: 作为库的根，负责设置 Tauri 应用、管理状态、注册命令和声明子模块。
    - `src/main.rs`: 最小化的二进制入口点，调用 `lib.rs` 中的启动函数。
    - `src/asr.rs`: 封装所有与阿里云 ASR WebSocket 通信相关的逻辑，包括状态定义、命令处理、异步任务等。
    - `src/llm_processor.rs`: 封装调用外部大模型 API 的逻辑 (参考阿里云流式输出文档：[https://help.aliyun.com/zh/model-studio/stream#493a8f482cfjs](https://help.aliyun.com/zh/model-studio/stream#493a8f482cfjs))。
    - 与前端通过 Tauri 命令 (IPC) 进行交互。
    - 状态管理和事件通知。
- **实时语音识别服务**: [阿里云 Gummy API](https://dashscope.console.aliyun.com/) (通过 WebSocket)。
- **外部 API 调用**: 使用 `reqwest` 库调用外部大模型 HTTP API。
- **异步处理**: 大量使用 Rust 的 `tokio` 运行时进行异步任务处理（WebSocket 通信、HTTP 请求、命令处理）。
- **状态管理**: 使用 `tokio::sync::Mutex` 和 Tauri 的状态管理机制在 Rust 端共享数据（如 API Key、任务句柄）。
- **依赖管理**: 使用 `npm` (前端) 和 `cargo` (Rust 后端)。
- **构建工具**: 使用 [Vite](https://vitejs.dev/) 进行前端开发和构建。

## 📦 项目结构

```
asr_android/                 # 项目根目录
├── dist/                    # 前端资源目录 (HTML, CSS, JS - Tauri frontendDist)
│   └── ...                  # 前端构建产物
├── src-tauri/               # Tauri 后端和配置目录
│   ├── src/                 # Rust 源代码
│   │   ├── main.rs          # 最小化的二进制入口点 (调用 lib.rs)
│   │   ├── lib.rs           # 库入口，设置 Tauri Builder, 声明模块
│   │   ├── asr.rs           # ASR 功能模块 (状态, 命令, WebSocket 逻辑)
│   │   └── llm_processor.rs # LLM API 调用模块
│   ├── Cargo.toml         # Rust 依赖 (crates) 配置文件
│   ├── tauri.conf.json    # Tauri 核心配置文件
│   ├── build.rs           # (可选) Tauri 构建脚本
│   ├── icons/               # 应用图标
│   └── gen/                 # Tauri 自动生成的平台项目文件
├── node_modules/            # Node.js 依赖包
├── scripts/                 # (可选) 辅助脚本目录
│   └── ...
├── .gitignore               # Git 忽略文件配置
├── package.json             # Node.js 项目配置和依赖
├── package-lock.json        # 锁定 Node.js 依赖版本
├── README.md                # 项目说明文档
└── vite.config.js           # Vite 配置文件
```

## 🚀 安装与使用

### 前提条件

**通用:**
- [Node.js](https://nodejs.org/) (推荐 LTS 版本)
- `npm` (通常随 Node.js 安装)
- [Rust](https://www.rust-lang.org/tools/install)
- **Tauri 系统依赖**: 根据你的操作系统，需要安装特定的依赖项 (如 WebView2, Build Tools, WebKitGTK 等)。请务必遵循 [Tauri 官方文档的 Prerequisites 指南](https://tauri.app/v2/guides/getting-started/prerequisites)。

**Android 开发/构建:**
- [Android Studio](https://developer.android.com/studio) (包含 Android SDK 和 Build-Tools)
- Android NDK (请确保版本与 `build-android-arm64.ps1` 脚本中 NDK 路径和 API Level 配置兼容)
- JDK 17 (或与 Android Gradle Plugin 兼容的版本)
- **推荐**: [vcpkg](https://vcpkg.io/) 用于管理 OpenSSL 等 C++ 依赖项，以便在 Windows 上进行 Android 交叉编译。需要设置 `VCPKG_ROOT` 环境变量。

### 设置与运行

1.  **克隆仓库**
    ```bash
    git clone <your-repo-url>
    cd asr_android
    ```

2.  **安装前端依赖**
    ```bash
    npm install
    ```

3.  **配置阿里云 Gummy API 凭证**
    - 首次启动应用时，或在应用的设置界面中，需要您**手动输入**您的阿里云 `AccessKey ID`, `AccessKey Secret`, 和 `AppKey`。
    - 这些凭证将被保存在浏览器的 `localStorage` 中，以便后续自动使用。
    - **注意**: 将敏感凭证存储在 `localStorage` 中方便了本地测试，但**安全性较低**。在生产环境或分发给他人使用时，请考虑更安全的凭证管理方式。

4.  **桌面端开发**
    - 确保已安装 Tauri 桌面环境依赖。
    - 运行开发模式：
      ```bash
      npm run tauri dev
      ```
    - 这会启动一个带热重载的开发窗口。

5.  **桌面端构建**
    - 运行构建命令：
      ```bash
      npm run tauri build
      ```
    - 生成的安装程序或可执行文件位于 `src-tauri/target/release/bundle/` 目录下。

6.  **Android 开发与调试**
    - **设置环境变量**: 确保 `ANDROID_HOME`, `ANDROID_NDK_HOME`, `JAVA_HOME`, `VCPKG_ROOT` (如果使用 vcpkg) 环境变量已正确设置。
    - **运行开发脚本**: 使用项目根目录下的 `dev-android.ps1` 脚本 (推荐)。此脚本会自动处理 OpenSSL 和 NDK Linker 等环境变量设置，并启动开发模式。
      ```powershell
      .\dev-android.ps1
      ```
    - **调试**: 应用启动后，使用 `chrome://inspect` 连接到应用的 WebView 进行调试 (详见"Android 开发与调试"章节)。

7.  **Android 构建 (Release AAB/APK)**
    - **设置环境变量**: 同上，确保环境变量正确。
    - **运行构建脚本**: 使用项目根目录下的 `build-android-arm64.ps1` 脚本 (推荐)。此脚本会自动处理环境变量并构建 Release 版本。
      ```powershell
      .\build-android-arm64.ps1
      ```
    - 构建产物 (通常是 `.aab` 或 `.apk`) 位于 `src-tauri/gen/android/app/build/outputs/` 下的相应目录中。

## 📱 Android APK构建指南

要将应用构建为Android APK，需要完成以下步骤：

### 1. 环境准备

构建Android APK需要以下环境：

- Android Studio (包含 Android SDK 和 Build-Tools)
- Android NDK (确保版本与项目兼容)
- JDK 17
- **重要**：除了设置 `ANDROID_HOME`, `NDK_HOME`, `JAVA_HOME` 环境变量外，请确保以下命令行工具也位于系统的 `PATH` 环境变量中，以便于构建和问题排查：
  - `keytool`: 通常位于 `$JAVA_HOME/bin` 目录下。
  - `apksigner`: 通常位于 `$ANDROID_HOME/build-tools/<version>` 目录下。

### 2. 设置环境变量

创建`setup-android.bat`（Windows）或`setup-android.sh`（Linux/macOS）脚本来设置必要的环境变量：

```bat
@echo off
REM 设置环境变量
set ANDROID_HOME=E:\Android\SDK
set NDK_HOME=E:\Android\SDK\ndk\29.0.13113456
set JAVA_HOME=E:\development\jdk17

REM 设置代理（可选，如果需要加速下载）
set HTTP_PROXY=http://127.0.0.1:7897
set HTTPS_PROXY=http://127.0.0.1:7897
set http_proxy=http://127.0.0.1:7897
set https_proxy=http://127.0.0.1:7897
set ALL_PROXY=http://127.0.0.1:7897

echo ANDROID_HOME设置为: %ANDROID_HOME%
echo NDK_HOME设置为: %NDK_HOME%
echo JAVA_HOME设置为: %JAVA_HOME%
echo 代理设置为: %HTTP_PROXY%

REM 运行tauri android init命令
call npm run tauri android init

REM 如果初始化成功，继续构建
if %ERRORLEVEL% EQU 0 (
  echo 初始化成功，开始构建Android APK...
  cd src-tauri
  call npm run tauri android build
)
```

### 3. 初始化Android支持

运行上面创建的脚本，或手动执行 `npm run tauri android init`。此步骤会在 `src-tauri/gen/android/` 目录下生成 Gradle 项目结构。

**注意**：此步骤后，签名配置的核心文件之一 `src-tauri/gen/android/app/build.gradle.kts` 会被创建或修改。

### 4. 配置签名密钥库

为了生成已签名的 Release APK，你需要：

1.  **生成密钥库文件**：如果还没有，请使用 `keytool` 生成一个 `.keystore` 文件（例如 `hanzi-writer.keystore`）并将其放置在 `src-tauri/` 目录下。记住你设置的**密钥库密码 (store password)** 和**密钥别名 (key alias)** 以及对应的**密钥密码 (key password)**。
    ```bash
    # 示例命令 (在 src-tauri 目录运行)
    keytool -genkey -v -keystore hanzi-writer.keystore -keyalg RSA -keysize 2048 -validity 10000 -alias hanziwriter
    ```
2.  **创建 `keystore.properties` 文件**：在 `src-tauri/gen/android/` 目录下（**注意**：不是 `app` 子目录）创建一个名为 `keystore.properties` 的文本文件。
3.  **编辑 `keystore.properties`**：填入以下内容，并替换为你自己的信息：
    ```properties
    keyAlias=你的密钥别名 # 例如 hanziwriter
    keyPassword=你的密钥密码 # 对应别名的密码
    # !!! 关键：相对于 src-tauri/gen/android/ 目录，指向 src-tauri/ 目录下的密钥库文件
    storeFile=../../../hanzi-writer.keystore
    storePassword=你的密钥库密码 # 整个 keystore 文件的密码
    ```
    **务必确保**：
    - 密码正确无误，且末尾没有多余空格。
    - `storeFile` 的相对路径正确指向你的 `.keystore` 文件。

### 5. 构建APK (检查 Gradle 配置)

现在可以运行构建命令：

```bash
# Windows
.\setup-android.bat
```
或者，如果您配置好了签名，可以直接运行：
```bash
# 在项目根目录运行
npm run tauri android build
```

**首次构建后检查**：Tauri 底层使用 Gradle 构建。构建过程依赖 `src-tauri/gen/android/app/build.gradle.kts` 文件。请打开此文件，检查以下关键配置是否存在且正确：

```kotlin
// src-tauri/gen/android/app/build.gradle.kts

import java.io.FileInputStream // 确保这个 import 存在
// ... 其他 imports ...

android {
    // ...
    signingConfigs {
        create("release") {
            // 检查这里的路径是否正确指向 src-tauri/gen/android/keystore.properties
            val keystorePropertiesFile = rootProject.file("keystore.properties")
            // ... 确保加载 keystore.properties 的逻辑存在 ...
        }
    }

    buildTypes {
        getByName("release") {
            // !!! 关键：确保这一行存在且没有被注释 !!!
            signingConfig = signingConfigs.getByName("release")
            // ... 其他 release 配置 ...
        }
        // ... debug 配置 ...
    }
    // ...
}
```
如果缺少 `signingConfigs` 块或 `release` 中的 `signingConfig = ...` 行，请参考上面示例手动添加或取消注释。

### 6. 常见问题与排查

#### 环境变量问题

*   **`keytool`/`apksigner` 找不到**：确认 JDK `bin` 目录和 Android SDK `build-tools` 目录已添加到系统 `PATH`。
*   **SDK/NDK/JDK 路径错误**：检查 `ANDROID_HOME`, `NDK_HOME`, `JAVA_HOME` 环境变量是否指向正确的安装位置。

#### 下载速度慢

*   **下载 Gradle 依赖或 Rust 工具链慢**：
    *   尝试设置 `HTTP_PROXY` 和 `HTTPS_PROXY` 环境变量。
    *   配置 Cargo 使用国内镜像源 (编辑 `~/.cargo/config` 或 `%USERPROFILE%\.cargo\config`)。

#### 构建失败

*   **生成 unsigned APK**：
    *   检查 `src-tauri/gen/android/app/build.gradle.kts` 文件，确认 `signingConfigs` 块存在，并且 `buildTypes.release` 中有 `signingConfig = signingConfigs.getByName("release")`。
    *   确认 `src-tauri/gen/android/keystore.properties` 文件存在且内容正确。
*   **`keystore.properties not found` 错误**：
    *   确认文件确实在 `src-tauri/gen/android/` 目录下。
    *   检查 `build.gradle.kts` 中 `rootProject.file(...)` 的路径是否正确（应为 `"keystore.properties"`）。
*   **`Keystore file '...' not found` 错误**：
    *   检查 `keystore.properties` 文件中的 `storeFile` 相对路径是否正确（通常是 `../../../hanzi-writer.keystore`）。
    *   确认 `.keystore` 文件本身存在于 `src-tauri/` 目录。
*   **`keystore password was incorrect` 或 `Failed to read key ...` 错误**：
    *   仔细检查 `keystore.properties` 文件中的 `storePassword` 和 `keyPassword` 是否与生成密钥库时设置的完全一致。
    *   确保密码末尾没有多余的空格。
*   **其他 Gradle 错误**：运行 `npm run tauri android build -- --verbose` 查看详细日志进行分析。

### 7. 安装和测试

构建成功后，**已签名**的APK文件将位于：`src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`。

您可以使用 `adb` 命令安装：
```powershell
# 确保 adb 在 PATH 中，且设备已连接并授权
adb install .\src-tauri\gen\android\app\build\outputs\apk\universal\release\app-universal-release.apk
```

您可以：
- 使用USB连接Android设备并直接安装APK

## 📱 Android 开发与调试 (Development & Debugging)

要在连接的 Android 设备或模拟器上运行应用的**开发版本**并启用 WebView 调试，请按照以下步骤操作：

### 1. 环境准备

- **确保设备/模拟器已连接**: 使用 `adb devices` 命令确认设备已连接并授权。
- **开启 USB 调试**: 在您的 Android 设备上启用开发者选项和 USB 调试。
- **设置环境变量**: 运行开发和构建模式**同样需要**正确的环境变量配置，特别是 NDK 链接器和 vcpkg 提供的 OpenSSL 路径。强烈建议使用类似 `build-android-arm64.ps1` 或 `dev-android.ps1` 的脚本来设置这些变量。
- **前端资源加载与安全上下文**: 
    - **问题**: `navigator.mediaDevices.getUserMedia` (用于麦克风、摄像头等) 等 Web API 需要在**安全上下文 (Secure Context)** 中运行。默认情况下，`https://` 或 `http://localhost` 被视为安全上下文，但**通过 IP 地址访问的 `http://` (例如 `http://192.168.x.x:port`) 则不被视为安全上下文**。
    - **错误**: 如果使用 `devUrl` 指向一个 HTTP IP 地址，WebView 加载后调用 `getUserMedia` 会因非安全上下文而失败，即使系统权限已授予，也可能报 `Permission denied` 或类似错误。
    - **推荐解决方案 (使用 Tauri 内建服务器)**: 
        1.  **修改 `src-tauri/tauri.conf.json`**: 
            *   **移除** `build.devUrl` 配置项。
            *   将 `build.frontendDist` 指向 Vite (或你的前端框架) **构建后的输出目录** (例如，根据 `vite.config.js` 的 `build.outDir`，可能是 `../dist_vite_build`)。
            *   将 `build.beforeDevCommand` 的值设为前端**构建命令** (例如 `"npm run build"`)。
            ```jsonc
            // tauri.conf.json (示例)
            {
              "build": {
                "frontendDist": "../dist_vite_build", // 指向构建产物
                // "devUrl" 已移除
                "beforeDevCommand": "npm run build", // 启动前先构建
                "beforeBuildCommand": "npm run build"
              },
              // ... 其他配置
            }
            ```
        2.  **工作原理**: 这样配置后，运行 `tauri android dev` (或相应的脚本) 时，会先执行 `npm run build` 将前端构建到 `dist_vite_build` 目录。然后 Tauri 会启动**内建的服务器**来加载这个目录下的静态资源，并通过其**自定义协议** (如 `tauri://localhost`) 提供服务。这个自定义协议被视为**安全上下文**，从而解决了 `getUserMedia` 的权限问题。
        3.  **缺点**: 每次启动开发都需要完整构建前端，启动速度变慢，且失去了 Vite Dev Server 提供的 HMR (热模块替换) 功能（Tauri 只提供基本的资源变更自动重载）。
    - **替代方案 (使用 HTTPS `devUrl`，不推荐)**: 理论上可以通过为 Vite Dev Server 配置 HTTPS 并使用 `https://` 的 `devUrl` 来创建安全上下文。但这通常涉及复杂的本地证书配置 (如使用 `mkcert`)，并且容易遇到证书信任、SSL 握手错误 (`ERR_SSL_VERSION_OR_CIPHER_MISMATCH`) 等问题，尤其是在移动设备上调试时。因此，对于需要访问敏感 Web API 的移动端开发，**优先推荐使用 Tauri 内建服务器方案**。
    - ~~WebSocket HMR 问题~~: (如果使用 Tauri 内建服务器方案，则之前关于 Vite HMR host 的问题通常不再相关，可以忽略或删除此条)

### 2. 启动开发模式

在设置好环境变量的终端中，运行：
```powershell
# 如果使用了脚本
.\dev-android.ps1

# 或者，如果手动设置了环境变量，直接运行：
npx tauri android dev
```
此命令会编译 Debug 版本的应用，将其安装到设备/模拟器并启动它。

### 3. 使用 Chrome DevTools 调试

1. 应用在设备/模拟器上运行后，在 PC 的 Chrome 浏览器中打开新标签页。
2. 地址栏输入 `chrome://inspect` 并回车。
3. 在 "Remote Target" 部分找到您的设备和应用对应的 WebView (例如 `com.hanzi.writer`)。
4. 点击 WebView 下方的 "inspect" 链接。
5. Chrome DevTools 窗口将打开，连接到应用的 WebView，您可以开始调试。

## 指定架构build

根据Tauri官方文档，可以通过 `--target` 参数在构建时指定目标架构。这比直接修改 Gradle 文件更推荐。

## 📄 许可证

[MIT License](LICENSE)

## 📞 联系方式

如有问题或建议，请通过以下方式联系：
- 电子邮件：[your-email@example.com]
- GitHub Issues：[项目Issues页面]

## ⚙️ 开发经验与注意事项

在本次开发和调试过程中，我们遇到了一些关于 Rust 项目结构、模块系统以及 Tauri 配置的问题，总结如下经验供参考：

1.  **项目结构 (lib.rs vs main.rs)**:
    *   **标准实践**: 对于同时包含库和二进制文件的 Rust 包 (由同一个 `Cargo.toml` 定义)，推荐将核心逻辑、数据结构、Tauri 命令和 Builder 配置等放在库文件 (`src/lib.rs`) 或其子模块中。
    *   二进制入口 (`src/main.rs`) 应保持简洁，其主要职责是调用库中定义的公共启动函数 (例如 `my_crate_name::run()` 或 `my_lib_name::run()`)。
    *   **避免反向依赖**: **切勿**在 `src/lib.rs` 中使用 `mod main;` 将二进制入口作为模块导入，这会破坏标准的依赖关系并导致编译/链接错误。

2.  **包内 Crate 引用 (`main.rs` 调用 `lib.rs`)**: 当 `src/main.rs` 需要调用 `src/lib.rs` 中的公共函数（如 `run`）时，引用方式取决于 `Cargo.toml` 是否显式定义了库名:
    *   **如果 `Cargo.toml` 未显式指定 `[lib] name`**: 使用**包名** (package name，`[package]` 下的 `name`，`-` 替换为 `_`) 进行引用。例如，如果包名是 `my-app`，则使用 `my_app::run()`。
    *   **如果 `Cargo.toml` 显式指定了 `[lib] name = "my_lib_name"`**: 则**必须**使用这个指定的**库名**进行引用。例如，使用 `my_lib_name::run()`。 (这是本项目遇到的情况，包名 `write-zh-char`，库名 `app_lib`，最终需使用 `app_lib::run()` 才成功)。

3.  **Tauri 命令注册**: 应将 `tauri::Builder::default()...invoke_handler(tauri::generate_handler![...])` 的配置集中在一处（推荐放在 `lib.rs` 的 `run` 函数中），并在 `generate_handler!` 宏中引用定义在各模块（如 `asr::`、`llm_processor::`）中的命令函数。这避免了重复注册 (E0255 错误)。

4.  **Tauri 状态管理**: 使用 `builder.manage(MyState(...))` 注入状态，并在命令函数签名中使用 `state: State<'_, MyState>` 来访问。这是在不同命令或任务间安全共享数据（如配置、API Key、任务句柄等）的标准方式。注意状态结构体需要 `Sync + Send`，通常需要 `Arc<Mutex<...>>` 或 `Arc<RwLock<...>>` 来包裹可变数据。状态的类型定义也应放在合适的模块中（如 `asr::ApiKeyState`）。

5.  **模块路径 (`mod`, `use`)**: 
    *   在 Crate 根 (`lib.rs`) 中使用 `mod module_name;` 来声明子模块。
    *   在不同模块间引用类型或函数时，使用 `use crate::module_name::Item;` 来导入，然后在代码中直接使用 `Item`。或者，直接使用完整路径 `crate::module_name::Item`。

希望这些记录能帮助后续开发避免类似的"坑"。
