# 文档地址：

https://pan.baidu.com/union/doc/Wm9sl0i0j


# 流程总结：

1、登录注册百度

2、访问地址获得个人授权

https://openapi.baidu.com/oauth/2.0/authorize?response_type=token&client_id=QHOuRXiepJBMjtk0esLhrPoNlQyYd0mF&redirect_uri=oob&scope=basic,netdisk

3、跳转到地址后：

copy拿到ak

https://openapi.baidu.com/oauth/2.0/login_success#expires_in=2592000&access_token=123.e4d76b3fdc9be6b67f99bf71650223e0.YDhd2cm70urDCjfNae4vaDx3ufgc-VEA0o1Fvo-.JFL5yw&session_secret=&session_key=&scope=basic+netdisk


比如如上地址：

123.e4d76b3fdc9be6b67f99bf71650223e0.YDhd2cm70urDCjfNae4vaDx3ufgc-VEA0o1Fvo-.JFL5yw

4、进入授权管理页面管理授权

https://pan.baidu.com/union/doc/Wm9sl0i0j


# 具体安装与使用：

https://pan.baidu.com/union/doc/Nm9sni0ju


简单的说就是要上传的话，需要它的python sdk

通过Stdio接入

一、Stdio源码获取
本地文件上传到百度网盘云端文件可通过Stdio源码接入，Stdio模式的Python SDK详见 【Python SDK-Stdio模式】

-----------------------

好的，这个计划听起来很有挑战性也很有趣！将 netdisk.py 中的文件上传逻辑用 Rust 重写并集成到您的 Tauri 文件管理器项目中，最终实现与百度网盘的同步功能，这是一个很棒的目标。

根据我的理解，主要步骤如下：
分析 Python 实现: 我们需要仔细研究 netdisk.py 中的 upload_file, upload_small_file, 和 upload_large_file 函数，理解它们与百度网盘 API 的交互方式，

包括：
API 端点 (如 xpanfileprecreate, pcssuperfile2, xpanfilecreate)。

请求参数和认证方式 (access_token)。
文件 MD5 的计算和使用。
分片上传的逻辑（分片大小、block_list 的构建等）。
错误处理和重试逻辑。

设计 Rust 实现:

HTTP 客户端: Rust 中流行的选择是 reqwest，它支持异步请求和文件上传。
JSON 解析: serde 和 serde_json 是标准选择。
MD5 计算: 可以使用 md5 crate。
异步处理: 由于涉及到网络和文件 I/O，使用 async/await 和 tokio 作为异步运行时会非常合适，以避免阻塞 Tauri 的主线程。
文件操作: Rust 的标准库 std::fs 和 std::io。
错误处理: 利用 Rust 的 Result 类型进行健壮的错误处理。
代码结构:
可以创建一个新的 Rust 模块（例如 baidu_netdisk_api.rs 或 uploader.rs）来放置这部分逻辑。
定义与百度网盘 API 交互所需的结构体（用于请求体和响应体）。
实现核心的上传函数，同样区分小文件和分片上传。
与 Tauri 集成:
您提到将代码与 main.rs 放在一起。更准确地说，这部分逻辑应该成为 Tauri 后端的一部分。如果 main.rs 中的 app_lib::run() 指向的是 src-tauri/src/lib.rs 或者一个同级的 app_lib.rs 文件，那么新的上传功能应该整合到那个库/模块中。
您需要创建 Tauri command，这些命令是前端 Vue 组件可以调用的异步 Rust 函数。例如，可以有一个 #[tauri::command] async fn upload_file_to_baidupan(local_path: String, remote_path: Option<String>) -> Result<String, String> 这样的命令。
这个命令会调用您在第二步中实现的 Rust 上传逻辑。
access_token 的处理方式需要考虑：是从环境变量读取，还是通过前端配置传递。Python 脚本中是从环境变量读取的。
我的建议步骤是：
确认 app_lib 的位置：您提到 main.rs 调用 app_lib::run()。我们需要确定 app_lib 是 src-tauri/src/lib.rs 文件，还是 src-tauri/src/app_lib.rs 文件，或者是 src-tauri/src/app_lib/mod.rs 这种形式的模块。这将决定我们将新代码集成到哪里。
添加依赖: 在 src-tauri/Cargo.toml 文件中添加必要的 Rust crates：
reqwest (启用 json, multipart 特性)
tokio (启用 macros, rt-multi-thread 特性)
serde, serde_json
md5
hex (用于将 MD5 字节转换为十六进制字符串)
anyhow 或 thiserror (可选，用于更方便的错误处理)
创建新的 Rust 文件: 在 src-tauri/src/ 目录下创建一个新文件，比如 baidu_uploader.rs。
逐步实现上传逻辑:
先定义好与百度网盘 API 交互所需的请求和响应结构体。
实现一个基础的 HTTP POST/GET 函数，用于调用百度 API。
实现小文件上传逻辑。
实现大文件（分片）上传逻辑。
确保正确计算和使用 MD5。
创建并暴露 Tauri Command:
在 app_lib.rs (或 lib.rs等) 中，引入新创建的 baidu_uploader 模块。
定义 Tauri command 函数，该函数接收前端传来的参数（如本地文件路径、目标网盘路径），调用 baidu_uploader 中的核心上传函数，并处理结果返回给前端。
在 app_lib::run() (或类似的地方，通常是 tauri::Builder::default().invoke_handler(...)) 中注册这个新的 command。
这是一个比较大的任务，我们可以一步一步来。
首先，为了明确代码应该放在哪里，我需要了解 src-tauri/src/ 目录的结构，特别是 app_lib 的具体形式。我将先列出 src-tauri/src/ 目录的内容。
好的，src-tauri/src/ 目录下有 lib.rs 和 main.rs。
这通常意味着 app_lib 就是指 lib.rs 文件（或者说，lib.rs 是名为 app_lib 这个库的根文件，库名在 src-tauri/Cargo.toml 中定义）。main.rs 编译成可执行文件，它会链接并调用 app_lib 库中的 run 函数。
因此，我们的新百度网盘上传逻辑可以这样做：
在 src-tauri/src/ 目录下创建一个新的文件，例如 baidu_uploader.rs。
在这个新文件中实现上传的核心逻辑。
在 src-tauri/src/lib.rs 中：
声明并引入 baidu_uploader 模块 (例如 mod baidu_uploader;)。
定义 Tauri command，这些 command 会调用 baidu_uploader 中的函数。
在 run 函数内（或者 tauri::Builder 的配置链中）注册这些 command。
接下来，我将为您在 src-tauri/Cargo.toml 中添加必要的依赖。请注意，我需要读取 Cargo.toml 的内容，然后提出修改。
好的，这是 src-tauri/Cargo.toml 的内容。
我注意到 [lib] 部分定义了 name = "app_lib"，这证实了我们的推断，lib.rs 就是 app_lib 库的根。
现在，我将为您建议对 src-tauri/Cargo.toml 的修改，以添加所需的依赖。
我将添加：
reqwest (with json and multipart features) for HTTP requests.
tokio (with macros and rt-multi-thread features) for the async runtime.
md5 for calculating MD5 hashes.
hex for converting byte arrays (like MD5 hashes) to hex strings.
anyhow for simpler error handling (optional, but often useful).




src-tauri/src/baidu_uploader.rs 文件已创建，其中包含了上传逻辑的初步框架。
这个框架包括：
BaiduUploader 结构体，用于持有 reqwest::Client 和 access_token。
new 构造函数。
upload_file 公共异步方法，它会根据文件大小判断调用 upload_small_file 还是 upload_large_file。
upload_small_file 和 upload_large_file 的基本实现，包括了对预创建（precreate）、分片上传（pcssuperfile2，仅大文件循环）、最终创建（create）这三个主要步骤的模拟调用。
MD5 计算逻辑的初步实现（小文件完整计算，大文件分片计算）。
使用了 anyhow::Result 进行错误处理，并用 log crate 记录信息。
API URL 和参数名基于 Python 脚本和常见的百度网盘 API 文档。
请注意：代码中包含了一些基于 Python 脚本的假设，例如 rtype=3 和 autoinit=1 的使用。API 的确切行为（尤其是错误代码和响应结构）可能需要通过实际测试或查阅最新的官方 API 文档来验证和调整。urlencoding::encode 用于对 URL 中的路径参数进行编码，这是必要的。pcssuperfile2 的 path 参数在 Python 脚本中没有显式 URL 编码，但在 Rust 的 reqwest 中作为查询参数构建 URL 时，最好进行编码。
重要提示和下一步:
错误处理和重试: 当前的实现是基础的，真正的生产级代码需要更完善的错误处理（例如，解析百度 API 返回的具体错误码和消息）和自动重试机制（reqwest 本身不直接提供像 Python requests 的 Retry 适配器那样的开箱即用方案，但可以手动实现或使用 backoff 等 crate）。
API 响应结构体: 我使用了 serde_json::Value 作为很多 API 调用的返回类型，以增加灵活性。在实际开发中，您可能希望为每个 API 端点的成功响应定义更精确的 Rust 结构体，以便更好地利用类型系统。我已经为 PrecreateResponse 和 CreateResponse 添加了示例结构体，但它们可能不完整或与实际 API 不完全匹配。
大文件上传中的 TokioFile::read: 在 upload_large_file 中，我使用了 TokioFile::read 来读取分片。请注意，file_for_upload.read(&mut chunk_buffer).await? 会从文件的当前位置开始读取，这对于顺序上传分片是合适的。如果需要支持分片上传的断点续传或更复杂的重试（例如只重试失败的分片），则需要更精细地控制文件指针，可能需要结合 std::fs::File 的 seek 方法（并在异步上下文中适当地处理它，或者确保每次重试时重新打开并定位文件）。
URL 编码: 我添加了 urlencoding::encode 来处理 remote_path 参数，当它作为 URL 的一部分时。这是很重要的。
Content-Type for pcssuperfile2: multipart 请求中的 Part 通常会自动推断 Content-Type，但对于文件上传，显式设置为 application/octet-stream 是一个好习惯。
block_list 的格式: 对于 precreate 和 create，block_list 参数是一个包含 MD5 字符串的 JSON 数组。代码中已按此实现。
现在，您需要将这个 baidu_uploader.rs 模块集成到 src-tauri/src/lib.rs 中，并创建 Tauri command 来调用它。
您想继续进行这一步吗？即修改 lib.rs 来：
声明 baidu_uploader 模块。
创建一个 Tauri command (例如 upload_to_baidu)，它会实例化 BaiduUploader 并调用其 upload_file 方法。
在 run() 函数中注册这个 command。




好的，我们继续。现在我们将修改 src-tauri/src/lib.rs 文件，以便集成 baidu_uploader 模块并暴露一个 Tauri command。
我将执行以下操作：
在 lib.rs 中添加 mod baidu_uploader; 来声明新模块。
添加 use baidu_uploader::BaiduUploader;。
定义一个新的异步 Tauri command upload_file_to_baidupan，它会：
接收 local_path: String 和 remote_dir: String 作为参数。
从环境变量 BAIDU_NETDISK_ACCESS_TOKEN 读取 access token。这是一个简化的做法，用于最小实验。在实际应用中，您可能需要更安全和灵活的方式来管理 token (例如，通过前端配置，或 Tauri 的 store API)。
创建 BaiduUploader 实例。
调用 uploader.upload_file()。
返回一个 Result<String, String> 给前端，成功时是包含上传信息的 JSON 字符串，失败时是错误信息。
修改 run() 函数中的 tauri::Builder，使用 .invoke_handler(tauri::generate_handler![...]) 来注册这个新的 command 以及任何已有的 command。