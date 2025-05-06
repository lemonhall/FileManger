<template>
  <div class="file-manager">
    <div class="toolbar">
      <button @click="goUp" :disabled="!canGoUp">向上</button>
      <button @click="syncSelectedToNetdisk" :disabled="!canSyncToNetdisk">同步到网盘</button>
      <span class="current-path">当前路径: {{ currentPath }}</span>
      <button @click="openSettingsModalView" class="settings-button" title="设置百度网盘Access Token">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="18px" height="18px">
          <path d="M19.43 12.98c.04-.32.07-.64.07-.98s-.03-.66-.07-.98l2.11-1.65c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.3-.61-.22l-2.49 1c-.52-.4-1.08-.73-1.69-.98l-.38-2.65C14.46 2.18 14.25 2 14 2h-4c-.25 0-.46.18-.49.42l-.38 2.65c-.61.25-1.17.59-1.69.98l-2.49-1c-.23-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64l2.11 1.65c-.04.32-.07.65-.07.98s.03.66.07.98l-2.11 1.65c-.19.15-.24.42.12.64l2 3.46c.12.22.39.3.61.22l2.49-1c.52.4 1.08.73 1.69.98l.38 2.65c.03.24.24.42.49.42h4c.25 0 .46-.18.49-.42l.38-2.65c.61-.25 1.17-.59 1.69-.98l2.49 1c.23.09.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.65zM12 15.5c-1.93 0-3.5-1.57-3.5-3.5s1.57-3.5 3.5-3.5 3.5 1.57 3.5 3.5-1.57 3.5-3.5 3.5z"/>
        </svg>
      </button>
    </div>
    <div v-if="loading" class="loading-indicator">加载中...</div>
    <div v-if="error" class="error-message">错误: {{ error }}</div>
    <div class="file-list" v-if="!loading && !error">
      <table>
        <thead>
          <tr>
            <th></th> <!-- Checkbox -->
            <th>名称</th>
            <th>类型</th>
            <th>大小</th>
            <th>属性</th> <!-- Added Attributes header -->
          </tr>
        </thead>
        <tbody>
          <tr v-if="items.length === 0">
             <td colspan="5">文件夹为空</td> <!-- Adjusted colspan -->
          </tr>
          <tr v-else v-for="item in items" :key="item.path" @dblclick="openItem(item)">
            <td><input type="checkbox" v-model="item.selected" @click.stop /></td> <!-- Prevent dblclick propagation -->
            <td>
              <span v-html="getFileIcon(item)"></span> <!-- Icon -->
              {{ item.name }}
            </td>
            <td>{{ getFileTypeDescription(item) }}</td> <!-- Call new method -->
            <td>{{ formatSize(item.size) }}</td>
            <td>{{ item.readonly ? 'R' : '' }}</td> <!-- Display R if readonly -->
          </tr>
        </tbody>
      </table>
    </div>

    <SettingsModal 
      :show="showSettingsModal" 
      :initial-access-token="storedAccessToken"
      @close="handleCloseSettingsModal"
      @save-token="handleSaveToken"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core'; // Correct v2 import
import { 
    folderIcon, 
    hiddenFolderIcon, 
    fileIcon, 
    imageIcon, 
    audioIcon, 
    videoIcon, 
    pdfIcon, 
    codeIcon, 
    archiveIcon, 
    executableIcon 
} from '../utils/icons';
import { fileTypeDescriptions } from '../utils/fileTypes';
import SettingsModal from './SettingsModal.vue'; // Import the new modal component
import { useBaiduNetdisk } from '../composables/useBaiduNetdisk'; // Import the composable

// --- 状态 ---
const currentPath = ref('');
const items = ref([]);
const loading = ref(true);
const error = ref(null);
const showSettingsModal = ref(false); 
const storedAccessToken = ref(''); 

// Initialize the composable for Baidu Netdisk operations
const { 
    uploadFile, 
    // We can also get isLoading and error from the composable if needed for FileManager UI
    // isLoading: baiduUploadIsLoading, 
    // error: baiduUploadError 
} = useBaiduNetdisk(storedAccessToken);

// --- 计算属性 ---
const canGoUp = computed(() => {
    // Basic check, needs improvement for root paths (C:\\, /)
    return currentPath.value && currentPath.value !== '/' && !/^[a-zA-Z]:\\\\?$/.test(currentPath.value);
});

const selectedFiles = computed(() => {
  return items.value.filter(item => item.selected && !item.is_dir);
});

const canSyncToNetdisk = computed(() => {
  return selectedFiles.value.length > 0;
});

// --- 新增方法: 获取文件/文件夹图标 ---
function getFileIcon(item) {
  // 1. 检查是否是文件夹
  if (item.is_dir) {
    // 检查是否是隐藏文件夹 (以 '.' 开头)
    // TODO: 未来可以依赖后端传来的 is_hidden 属性
    if (item.name.startsWith('.')) {
      return hiddenFolderIcon;
    } else {
      return folderIcon;
    }
  }

  // 2. 处理文件
  const name = item.name || '';
  const lastDotIndex = name.lastIndexOf('.');
  if (lastDotIndex === -1 || lastDotIndex === 0 || lastDotIndex === name.length - 1) {
    return fileIcon; // 没有有效扩展名，返回通用文件图标
  }

  const extension = name.substring(lastDotIndex + 1).toLowerCase();

  // 3. 根据扩展名返回特定图标
  switch (extension) {
    // 图片
    case 'jpg': case 'jpeg': case 'png': case 'gif': case 'bmp': case 'tiff': case 'tif': case 'svg': case 'ico': case 'webp': case 'psd': case 'ai': case 'raw': case 'cr2': case 'nef': case 'orf': case 'arw':
      return imageIcon;
    // 音频
    case 'mp3': case 'wav': case 'ogg': case 'flac': case 'aac': case 'wma': case 'm4a': case 'ape': case 'mid': case 'midi':
      return audioIcon;
    // 视频
    case 'mp4': case 'avi': case 'mkv': case 'mov': case 'wmv': case 'flv': case 'webm': case 'mpg': case 'mpeg': case 'm4v': case '3gp':
      return videoIcon;
    // PDF
    case 'pdf':
      return pdfIcon;
    // 代码
    case 'js': case 'ts': case 'jsx': case 'tsx': case 'css': case 'scss': case 'less': case 'py': case 'java': case 'c': case 'cpp': case 'h': case 'hpp': case 'cs': case 'go': case 'php': case 'rb': case 'swift': case 'kt': case 'rs': case 'vue': case 'sql': case 'html': case 'htm': case 'xml': case 'json': case 'md': case 'sh': case 'bat':
      return codeIcon;
    // 压缩文件
    case 'zip': case 'rar': case '7z': case 'gz': case 'tar': case 'bz2': case 'xz': case 'iso': case 'img':
      return archiveIcon;
    // 可执行文件
    case 'exe': case 'dll': case 'msi': case 'jar': case 'apk': case 'app': // .app is tricky
      return executableIcon;
    // 文本 (如果不是其他特定类型)
    case 'txt': case 'log': case 'ini': case 'csv':
       return fileIcon; // 可以用通用文件图标，或专门的文本图标
    // 字体
    case 'ttf': case 'otf': case 'woff': case 'woff2': case 'eot':
       // 暂时用通用图标
       return fileIcon;
    // 其他已知类型但无特定图标 (如Office文档)
    case 'doc': case 'docx': case 'xls': case 'xlsx': case 'ppt': case 'pptx': case 'odt': case 'ods': case 'odp': case 'rtf':
       // 可以用通用图标，或文档图标
       return fileIcon;

    // 默认通用文件图标
    default:
      return fileIcon;
  }
}

// --- 新增方法: 同步选中文件到网盘 ---
async function syncSelectedToNetdisk() {
  const token = storedAccessToken.value; // Token is now managed by the composable via storedAccessToken ref
  if (!token) {
      alert("请先在设置中配置百度网盘Access Token!");
      openSettingsModalView();
      return;
  }
  if (!canSyncToNetdisk.value) {
    alert("请至少选择一个文件进行同步。");
    return;
  }

  const filesToSync = selectedFiles.value;
  const remoteBaseDir = "/来自FileManger同步";

  const syncButton = document.querySelector('.toolbar button:nth-child(2)');
  let originalButtonText = '';
  if (syncButton) {
      originalButtonText = syncButton.textContent;
      syncButton.textContent = '同步中...';
      syncButton.disabled = true;
  }

  let successCount = 0;
  let errorCount = 0;

  for (const file of filesToSync) {
    console.log(`准备上传: ${file.path} 到 ${remoteBaseDir}`);
    try {
      // const result = await invoke('upload_file_to_baidupan', {
      //   localPath: file.path,
      //   remoteDir: remoteBaseDir,
      //   accessToken: token // Pass the correct token
      // });
      // Use the uploadFile method from the composable
      // The composable uses storedAccessToken internally, but we can also pass it explicitly if preferred
      // For consistency with how other methods in the composable can override the ref, let's pass it.
      const result = await uploadFile(file.path, remoteBaseDir, token);
      console.log(`上传成功: ${file.name}`, result);
      alert(`文件 '${file.name}' 上传成功!\n响应: ${JSON.stringify(result)}`); // Stringify result for better alert display
      successCount++;
    } catch (err) {
      console.error(`上传失败: ${file.name}`, err);
      alert(`文件 '${file.name}' 上传失败!\n错误: ${err}`);
      errorCount++;
    }
  }

  if (syncButton) {
      syncButton.textContent = originalButtonText;
      syncButton.disabled = !canSyncToNetdisk.value;
  }

  alert(`同步完成! 成功: ${successCount}，失败: ${errorCount}。详情请查看控制台。`);

  items.value.forEach(item => item.selected = false);
}

// --- 方法 ---
async function listDirectory(path) {
  loading.value = true;
  error.value = null;
  console.log(`尝试列出目录: ${path}`);

  try {
    const result = await invoke('list_directory', { path: path });

    // --- 开始排序 ---
    const sortedResult = result.sort((a, b) => {
      // 1. 类型比较 (文件夹优先)
      if (a.is_dir && !b.is_dir) {
        return -1; // a (文件夹) 在前
      }
      if (!a.is_dir && b.is_dir) {
        return 1; // b (文件夹) 在前
      }

      // 2. 同类型按名称排序 (使用 localeCompare 支持中文)
      // 'zh-CN' 确保中文按拼音排序
      // sensitivity: 'base' 可以忽略大小写差异，如果需要区分大小写可以去掉或改为 'variant'
      // numeric: true 让名称中的数字按数值排序 (如 'folder10' 在 'folder2' 之后)
      return a.name.localeCompare(b.name, 'zh-CN', { sensitivity: 'base', numeric: true });
    });
    // --- 结束排序 ---

    items.value = sortedResult.map(item => ({ ...item, selected: false }));
    currentPath.value = path; // Update path on success
  } catch (e) {
    error.value = `无法加载目录: ${e}`;
    console.error(error.value);
    items.value = []; // Clear items on error
  } finally {
    loading.value = false;
  }
}

async function initializePath() {
  loading.value = true;
  error.value = null;
  try {
    const initialPath = await invoke('get_initial_path');
    await listDirectory(initialPath);
  } catch (e) {
    error.value = `无法获取初始路径: ${e}`;
    console.error(error.value);
    currentPath.value = '?';
    loading.value = false;
  }
}

function goUp() {
  if (!canGoUp.value || !currentPath.value) return;

    const isWindowsPath = currentPath.value.includes('\\');
    // Normalize path by removing trailing slashes before splitting, 
    // but be careful not to remove the slash from a root like "C:\\" or "/"
    let pathToProcess = currentPath.value;
    if (pathToProcess.match(/^[a-zA-Z]:\\+$/)) { // Matches C:\, C:\\, etc.
        // If it's just the drive root, cannot go up further with this logic
        // This case should ideally be caught by canGoUp, but double check here.
        console.warn("Already at drive root or invalid state for goUp:", currentPath.value);
        return;
    } else if (pathToProcess !== '/' && (pathToProcess.endsWith('\\') || pathToProcess.endsWith('/'))) {
        pathToProcess = pathToProcess.substring(0, pathToProcess.length - 1);
    }

    const parts = pathToProcess.split(isWindowsPath ? /\\/ : /\//);

    // If original path was something like "C:\Users" or "/home/user"
    // after pop, parts would be ["C:", "Users"] -> ["C:"] or ["", "home", "user"] -> ["", "home"]
    if (parts.length <= 1 && isWindowsPath && /^[a-zA-Z]:$/.test(parts[0])) {
        // This means we are at a path like C:\file or C:\folder, and going up should lead to C:\
        // or we are at C: (which should have been caught by trailing slash normalization or canGoUp)
        // For C:\folder -> C:\
        if (parts[0].endsWith(':')) {
            listDirectory(parts[0] + '\\');
            return;
        }
        // Other cases like single part being just "C:" should ideally not happen if path is well-formed like C:\foo
    } else if (parts.length === 1 && !isWindowsPath && parts[0] === '') {
        // This means we are at /file or /folder, and going up should lead to /
        listDirectory('/');
        return;
    }

    parts.pop();

    let parentPath;
    if (isWindowsPath) {
        if (parts.length === 1 && /^[a-zA-Z]:$/.test(parts[0])) {
            // Drive letter, e.g., C:, parent is C:\\
            parentPath = parts[0] + '\\';
        } else {
            parentPath = parts.join('\\');
            // If the original path was a root of a share or a deeper path that becomes empty
            // this join might be problematic. E.g. \\server\share -> parts ['', '', 'server', 'share']
            // For now, assuming simple C:\path structures.
        }
    } else {
        // POSIX paths
        if (parts.length === 1 && parts[0] === '') {
            // Parent is the root directory "/"
            parentPath = '/';
        } else {
            parentPath = parts.join('/');
            if (parentPath === '' && currentPath.value.startsWith('/')) {
                // This can happen if currentPath was like /foo, parts becomes [''] after pop, join is ''
                parentPath = '/';
            }
        }
    }
    
    // Final check for empty or invalid parentPath before listing
    if (!parentPath) {
        // This might happen for Unix root if not handled above, or other edge cases
        if (!isWindowsPath && currentPath.value.startsWith('/')) {
            parentPath = '/'; // Default to root if all else fails for POSIX
        } else {
            console.warn("Could not determine parent path for:", currentPath.value, "Resulting parts:", parts);
            return; // Avoid listing an empty or invalid path
        }
    }

    listDirectory(parentPath);
}

function openItem(item) {
  if (item.is_dir) { // Use is_dir from Rust
    listDirectory(item.path);
  } else {
    console.log(`尝试打开文件: ${item.path}`);
    alert(`打开文件: ${item.name}`);
    // TODO: File opening logic
  }
}

function formatSize(size) {
    if (size === null || size === undefined) return '-';
    const numSize = Number(size); // Ensure it's a number
    if (isNaN(numSize)) return '-';
    if (numSize < 1024) return `${numSize} B`;
    if (numSize < 1024 * 1024) return `${(numSize / 1024).toFixed(1)} KB`;
    if (numSize < 1024 * 1024 * 1024) return `${(numSize / (1024 * 1024)).toFixed(1)} MB`;
    return `${(numSize / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function getFileTypeDescription(item, useShortDescription = true) { // 添加 useShortDescription 参数
  if (item.is_dir) {
    return '文件夹';
  }

  const name = item.name || '';
  const lastDotIndex = name.lastIndexOf('.');
  if (lastDotIndex === -1 || lastDotIndex === 0 || lastDotIndex === name.length - 1) {
    return '文件'; // 没有扩展名或点在开头/结尾
  }

  const extension = name.substring(lastDotIndex + 1).toLowerCase();
  const descriptionData = fileTypeDescriptions[extension];

  if (descriptionData) {
    return useShortDescription ? descriptionData.short : descriptionData.full;
  } else {
    // 对于未知类型，返回大写的扩展名
    return `${extension.toUpperCase()} 文件`;
  }
}

// --- Settings Modal Methods (Now handled by SettingsModal.vue or through events) ---
function openSettingsModalView() {
  showSettingsModal.value = true;
}

function handleSaveToken(newToken) {
  if (newToken) {
    localStorage.setItem('BAIDU_NETDISK_ACCESS_TOKEN_VUE', newToken);
    storedAccessToken.value = newToken;
    alert('Access Token 已保存!');
    // Optionally, trigger info fetch in modal if it's still open, 
    // or let the modal handle it internally upon saving.
    // For now, SettingsModal handles its own info fetching.
  } else {
    localStorage.removeItem('BAIDU_NETDISK_ACCESS_TOKEN_VUE');
    storedAccessToken.value = '';
    alert('Access Token 已清除!');
  }
  showSettingsModal.value = false; // Close modal on save
}

function handleCloseSettingsModal() {
  showSettingsModal.value = false;
}

// --- 生命周期钩子 ---
onMounted(() => {
  initializePath();
  storedAccessToken.value = localStorage.getItem('BAIDU_NETDISK_ACCESS_TOKEN_VUE') || '';
});

</script>

<style scoped>
/* 微调样式以适应图标 */
.file-manager {
  padding: 1rem;
}
.toolbar {
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
  gap: 1rem;
}
.current-path {
  font-style: italic;
  color: #555;
}
.settings-button {
  margin-left: auto; /* Pushes to the far right */
  background: none;
  border: none;
  cursor: pointer;
  padding: 5px; /* Adjust padding as needed */
  display: flex;
  align-items: center;
  justify-content: center;
}
.settings-button svg {
  color: #555;
}
.settings-button:hover svg {
  color: #000;
}

.loading-indicator, .error-message {
    margin-top: 1rem;
    padding: 0.5rem;
    font-weight: bold;
}
.error-message {
    color: red;
    border: 1px solid red;
    background-color: #ffeeee;
}
.file-list table {
  width: 100%;
  border-collapse: collapse;
}
.file-list th, .file-list td {
  border: 1px solid #ddd;
  padding: 8px;
  text-align: left;
  white-space: nowrap;
  vertical-align: middle;
}
.file-list th {
  background-color: #f2f2f2;
}
.file-list tbody tr:hover {
  background-color: #f5f5f5;
  cursor: pointer;
}

.file-list th:first-child, .file-list td:first-child { width: 30px; text-align: center;}
.file-list th:nth-child(2), .file-list td:nth-child(2) { width: auto; white-space: normal; }
.file-list th:nth-child(3), .file-list td:nth-child(3) { width: 120px; }
.file-list th:nth-child(4), .file-list td:nth-child(4) { width: 100px; }
.file-list th:nth-child(5), .file-list td:nth-child(5) { width: 50px; text-align: center; }

</style> 