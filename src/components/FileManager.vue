<template>
  <div class="file-manager">
    <div class="toolbar">
      <button @click="goUp" :disabled="!canGoUp">向上</button>
      <button @click="syncSelectedToNetdisk" :disabled="!canSyncToNetdisk">同步到网盘</button>
      <span class="current-path">当前路径: {{ currentPath }}</span>
      <button @click="openSettingsModal" class="settings-button" title="设置百度网盘Access Token">
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

    <!-- Settings Modal -->
    <div v-if="showSettingsModal" class="modal-overlay" @click.self="closeSettingsModal">
      <div class="modal-content">
        <h3>百度网盘设置</h3>
        <div class="form-group">
          <label for="baidu-token">Access Token:</label>
          <input type="password" id="baidu-token" v-model="baiduAccessTokenInput" placeholder="请输入百度网盘Access Token">
        </div>

        <!-- User Info Display Area -->
        <div v-if="userInfoLoading" class="info-loading">正在获取信息...</div>
        <div v-if="userInfoError" class="info-error">错误: {{ userInfoError }}</div>
        <div v-if="userInfo || quotaInfo" class="user-info-display">
          <h4 v-if="userInfo">用户信息</h4>
          <p v-if="userInfo">用户名: {{ userInfo.baidu_name }} ({{ userInfo.netdisk_name }})</p>
          <p v-if="userInfo">VIP类型: {{ vipTypeToString(userInfo.vip_type) }}</p>
          <p v-if="userInfo"><img :src="userInfo.avatar_url" alt="avatar" width="30" height="30" style="vertical-align: middle; border-radius: 50%;"></p>
          <h4 v-if="quotaInfo">存储配额</h4>
          <p v-if="quotaInfo">{{ formatSize(quotaInfo.used) }} / {{ formatSize(quotaInfo.total) }}</p>
          <progress v-if="quotaInfo" :value="quotaInfo.used" :max="quotaInfo.total" style="width: 100%;"></progress>
        </div>
        <!-- End User Info Display Area -->

        <div class="modal-actions">
          <button @click="fetchAndDisplayUserInfo" :disabled="!baiduAccessTokenInput.trim() || userInfoLoading">检查Token并获取信息</button> <!-- New Button -->
          <button @click="saveSettings">保存</button>
          <button @click="closeSettingsModal">取消</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core'; // Correct v2 import

// --- 状态 ---
const currentPath = ref('');
const items = ref([]);
const loading = ref(true);
const error = ref(null);
const showSettingsModal = ref(false);
const baiduAccessTokenInput = ref('');
const storedAccessToken = ref(''); // To store the token loaded from localStorage
const userInfo = ref(null);
const quotaInfo = ref(null);
const userInfoLoading = ref(false);
const userInfoError = ref(null);

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

// --- SVG 图标定义 ---
// (使用内联SVG简化处理，颜色和样式可以稍后调整)

// 基础样式，可用于所有图标
const svgStyle = 'display: inline-block; vertical-align: middle; margin-right: 5px; width: 16px; height: 16px;';

// 文件夹图标 (普通)
const folderIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #FFC107;">
  <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
</svg>`;

// 文件夹图标 (隐藏 - 半透明)
const hiddenFolderIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #FFC107; opacity: 0.6;">
  <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
</svg>`;

// 通用文件图标
const fileIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #90A4AE;">
  <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 14h-8v-2h8v2zm0-4h-8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
</svg>`;

// 图片文件图标
const imageIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #4CAF50;">
 <path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"/>
</svg>`;

// 音频文件图标
const audioIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #2196F3;">
  <path d="M12 3v9.28c-.47-.17-.97-.28-1.5-.28C8.01 12 6 14.01 6 16.5S8.01 21 10.5 21c2.31 0 4.2-1.75 4.45-4H15V6h4V3h-7z"/>
</svg>`;

// 视频文件图标
const videoIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #F44336;">
 <path d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z"/>
</svg>`;

// PDF 文件图标
const pdfIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #E91E63;">
 <path d="M20 2H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-8.5 7.5c0 .83-.67 1.5-1.5 1.5H9v2H7.5V7H10c.83 0 1.5.67 1.5 1.5v1zm5 2c0 .83-.67 1.5-1.5 1.5h-2.5V7H15c.83 0 1.5.67 1.5 1.5v3zm4-3H19v1h1.5V11H19v2h-1.5V7h3v1.5zM9 9.5h1v-1H9v1z"/>
</svg>`;

// 代码文件图标
const codeIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #00BCD4;">
 <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/>
</svg>`;

// 压缩文件图标
const archiveIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #FF9800;">
  <path d="M20 6h-8l-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm-1 8h-3v3h-2v-3h-3v-2h3V9h2v3h3v2z"/>
</svg>`;

// 可执行文件图标
const executableIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" style="${svgStyle} color: #795548;">
  <path d="M13.89 8.7L12 10.59 10.11 8.7a.996.996 0 10-1.41 1.41L10.59 12 8.7 13.89a.996.996 0 101.41 1.41L12 12.41l1.89 1.89a.996.996 0 101.41-1.41L13.41 12l1.89-1.89a.996.996 0 000-1.41c-.39-.38-1.03-.38-1.41 0zM19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14z"/>
</svg>`;

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
  const token = storedAccessToken.value;
  if (!token) {
      alert("请先在设置中配置百度网盘Access Token!");
      openSettingsModal();
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
      const result = await invoke('upload_file_to_baidupan', {
        localPath: file.path,
        remoteDir: remoteBaseDir,
        accessToken: token // Pass the correct token
      });
      console.log(`上传成功: ${file.name}`, result);
      alert(`文件 '${file.name}' 上传成功!\n响应: ${result}`);
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

    const parts = currentPath.value.replace(/\\\\$/, '').split(/[\\/]/);
    if (parts.length <= 1 && !/^[a-zA-Z]:$/.test(parts[0])) {
        // Already at root (e.g., '/') or invalid path
        return;
    }
     if (parts.length === 1 && /^[a-zA-Z]:$/.test(parts[0])) {
         // Already at drive root (e.g., C:)
         return;
     }


    parts.pop();
    let parentPath = parts.join(currentPath.value.includes('\\\\') ? '\\\\' : '/');

    // Handle going up from root directory (e.g., C:\Users -> C:\)
    // Or handle root like /home -> /
    if (/^[a-zA-Z]:$/.test(parts[0]) && parts.length === 1) {
        parentPath += '\\\\'; // Append backslash for drive root C:\
    } else if (parentPath === '' && currentPath.value.startsWith('/')) {
        parentPath = '/'; // Set to root for Unix-like paths
    } else if (parentPath === '' && /^[a-zA-Z]:/.test(currentPath.value)) {
        // This case might occur if path was "C:", should already be handled above
         console.warn("Unexpected empty parent path for drive:", currentPath.value);
         return; // Avoid navigating to empty string
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

// --- 新增方法: 获取文件类型描述 ---

// 定义扩展名到描述的映射
const fileTypeDescriptions = {
  // 文档
  'txt': { full: '文本文档', short: '文本' },
  'doc': { full: 'Microsoft Word 97-2003 文档', short: 'Word 文档' },
  'docx': { full: 'Microsoft Word 文档', short: 'Word 文档' },
  'pdf': { full: '可移植文档格式 (PDF)', short: 'PDF' },
  'rtf': { full: '富文本格式文档', short: '富文本' },
  'odt': { full: 'OpenDocument 文本', short: 'ODT 文档' },
  'md': { full: 'Markdown 文档', short: 'Markdown' },
  'html': { full: '超文本标记语言页面', short: 'HTML' },
  'htm': { full: '超文本标记语言页面', short: 'HTML' },
  'xml': { full: '可扩展标记语言文件', short: 'XML' },
  'csv': { full: '逗号分隔值文件', short: 'CSV' },
  'log': { full: '日志文件', short: '日志' },
  'ini': { full: '初始化配置文件', short: '配置' },
  'json': { full: 'JavaScript 对象表示法文件', short: 'JSON' },

  // 电子表格
  'xls': { full: 'Microsoft Excel 97-2003 工作表', short: 'Excel 工作表' },
  'xlsx': { full: 'Microsoft Excel 工作表', short: 'Excel 工作表' },
  'ods': { full: 'OpenDocument 电子表格', short: 'ODS 表格' },

  // 演示文稿
  'ppt': { full: 'Microsoft PowerPoint 97-2003 演示文稿', short: 'PPT 演示' },
  'pptx': { full: 'Microsoft PowerPoint 演示文稿', short: 'PPT 演示' },
  'odp': { full: 'OpenDocument 演示文稿', short: 'ODP 演示' },

  // 图像
  'jpg': { full: 'JPEG 图像', short: '图片' },
  'jpeg': { full: 'JPEG 图像', short: '图片' },
  'png': { full: '可移植网络图形', short: '图片' },
  'gif': { full: '图形交换格式', short: '图片' },
  'bmp': { full: '位图图像', short: '图片' },
  'tiff': { full: '标记图像文件格式', short: '图片' },
  'tif': { full: '标记图像文件格式', short: '图片' },
  'svg': { full: '可缩放矢量图形', short: '矢量图' },
  'ico': { full: '图标文件', short: '图标' },
  'webp': { full: 'WebP 图像', short: '图片' },
  'psd': { full: 'Adobe Photoshop 文档', short: 'PSD' },
  'ai': { full: 'Adobe Illustrator 文档', short: 'AI' },
  'raw': { full: '相机原始图像文件', short: '原始图像' },
  'cr2': { full: '佳能相机原始图像', short: '原始图像' },
  'nef': { full: '尼康相机原始图像', short: '原始图像' },
  'orf': { full: '奥林巴斯相机原始图像', short: '原始图像' },
  'arw': { full: '索尼相机原始图像', short: '原始图像' },

  // 音频
  'mp3': { full: 'MP3 音频文件', short: '音频' },
  'wav': { full: '波形音频文件', short: '音频' },
  'ogg': { full: 'Ogg Vorbis 音频文件', short: '音频' },
  'flac': { full: '免费无损音频编解码器文件', short: '无损音频' },
  'aac': { full: '高级音频编码文件', short: '音频' },
  'wma': { full: 'Windows Media 音频文件', short: '音频' },
  'm4a': { full: 'MPEG-4 音频文件', short: '音频' },
  'ape': { full: "Monkey's Audio 无损音频", short: '无损音频' },
  'mid': { full: 'MIDI 序列', short: 'MIDI' },
  'midi': { full: 'MIDI 序列', short: 'MIDI' },

  // 视频
  'mp4': { full: 'MPEG-4 视频文件', short: '视频' },
  'avi': { full: '音频视频交错文件', short: '视频' },
  'mkv': { full: 'Matroska 视频文件', short: '视频' },
  'mov': { full: 'Apple QuickTime 电影', short: '视频' },
  'wmv': { full: 'Windows Media 视频文件', short: '视频' },
  'flv': { full: 'Flash 视频文件', short: 'Flash 视频' },
  'webm': { full: 'WebM 视频文件', short: 'WebM 视频' },
  'mpg': { full: 'MPEG 视频文件', short: '视频' },
  'mpeg': { full: 'MPEG 视频文件', short: '视频' },
  'm4v': { full: 'MPEG-4 视频文件 (Apple)', short: '视频' },
  '3gp': { full: '3GPP 多媒体文件', short: '3GP 视频' },

  // 压缩文件
  'zip': { full: 'Zip 压缩文件', short: '压缩包' },
  'rar': { full: 'RAR 压缩文件', short: '压缩包' },
  '7z': { full: '7-Zip 压缩文件', short: '压缩包' },
  'gz': { full: 'GNU Zip 压缩文件', short: '压缩包' },
  'tar': { full: 'Tar 归档文件', short: '归档包' },
  'bz2': { full: 'Bzip2 压缩文件', short: '压缩包' },
  'xz': { full: 'XZ 压缩文件', short: '压缩包' },
  'iso': { full: '光盘映像文件', short: '光盘映像' },
  'img': { full: '磁盘映像文件', short: '磁盘映像' },

  // 可执行与系统文件
  'exe': { full: 'Windows 可执行文件', short: '应用程序' },
  'dll': { full: '动态链接库', short: '库文件' },
  'bat': { full: 'Windows 批处理文件', short: '批处理' },
  'sh': { full: 'Shell 脚本', short: '脚本' },
  'msi': { full: 'Windows Installer 包', short: '安装包' },
  'sys': { full: '系统文件', short: '系统文件' },
  'jar': { full: 'Java 归档文件', short: 'JAR 包' },
  'apk': { full: 'Android 应用包', short: 'APK' },
  'app': { full: 'macOS 应用程序', short: '应用' }, // Might be a folder on macOS

  // 代码与开发
  'js': { full: 'JavaScript 文件', short: 'JS 代码' },
  'ts': { full: 'TypeScript 文件', short: 'TS 代码' },
  'jsx': { full: 'JavaScript XML (React)', short: 'JSX 代码' },
  'tsx': { full: 'TypeScript XML (React)', short: 'TSX 代码' },
  'css': { full: '层叠样式表', short: 'CSS' },
  'scss': { full: 'Sass 层叠样式表', short: 'Sass' },
  'less': { full: 'Less 层叠样式表', short: 'Less' },
  'py': { full: 'Python 脚本', short: 'Python' },
  'java': { full: 'Java 源代码', short: 'Java' },
  'c': { full: 'C 源代码', short: 'C 代码' },
  'cpp': { full: 'C++ 源代码', short: 'C++ 代码' },
  'h': { full: 'C/C++ 头文件', short: '头文件' },
  'hpp': { full: 'C++ 头文件', short: '头文件' },
  'cs': { full: 'C# 源代码', short: 'C#' },
  'go': { full: 'Go 源代码', short: 'Go' },
  'php': { full: 'PHP 脚本', short: 'PHP' },
  'rb': { full: 'Ruby 脚本', short: 'Ruby' },
  'swift': { full: 'Swift 源代码', short: 'Swift' },
  'kt': { full: 'Kotlin 源代码', short: 'Kotlin' },
  'rs': { full: 'Rust 源代码', short: 'Rust' },
  'vue': { full: 'Vue.js 组件', short: 'Vue 组件' },
  'sql': { full: 'SQL 查询文件', short: 'SQL' },

  // 其他
  'ttf': { full: 'TrueType 字体', short: '字体' },
  'otf': { full: 'OpenType 字体', short: '字体' },
  'woff': { full: 'Web 开放字体格式', short: '字体' },
  'woff2': { full: 'Web 开放字体格式 2.0', short: '字体' },
  'eot': { full: '嵌入式 OpenType 字体', short: '字体' },
  'ics': { full: 'iCalendar 文件', short: '日历' },
  'vcf': { full: 'vCard 文件', short: '联系人' },
  'url': { full: 'Internet 快捷方式', short: '快捷方式' },
  'lnk': { full: 'Windows 快捷方式', short: '快捷方式' },
};

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

// --- Settings Modal Methods ---
function openSettingsModal() {
  baiduAccessTokenInput.value = storedAccessToken.value; // Use stored token when opening
  // Reset info display when opening
  userInfo.value = null;
  quotaInfo.value = null;
  userInfoError.value = null;
  userInfoLoading.value = false;
  showSettingsModal.value = true;
}

async function fetchAndDisplayUserInfo() {
  const token = baiduAccessTokenInput.value.trim();
  if (!token) {
    alert("请输入有效的 Access Token!");
    return;
  }

  userInfoLoading.value = true;
  userInfoError.value = null;
  userInfo.value = null; // Clear previous results
  quotaInfo.value = null; // Clear previous results

  try {
    // Fetch both concurrently
    const [infoResult, quotaResult] = await Promise.allSettled([
      invoke('get_baidu_user_info', { accessToken: token }),
      invoke('get_baidu_quota', { accessToken: token })
    ]);

    let errors = [];
    if (infoResult.status === 'fulfilled') {
      userInfo.value = infoResult.value;
      console.log("User Info Fetched:", infoResult.value);
    } else {
      errors.push(`获取用户信息失败: ${infoResult.reason}`);
      console.error("User Info Fetch Error:", infoResult.reason);
    }

    if (quotaResult.status === 'fulfilled') {
      quotaInfo.value = quotaResult.value;
      console.log("Quota Info Fetched:", quotaResult.value);
    } else {
      errors.push(`获取配额信息失败: ${quotaResult.reason}`);
      console.error("Quota Info Fetch Error:", quotaResult.reason);
    }

    if (errors.length > 0) {
        userInfoError.value = errors.join('; \n');
    }

  } catch (e) {
    userInfoError.value = `获取信息时发生意外错误: ${e}`;
    console.error("Unexpected error during fetch:", e);
  } finally {
    userInfoLoading.value = false;
  }
}

function saveSettings() {
  const currentToken = baiduAccessTokenInput.value.trim();
  if (currentToken) {
    localStorage.setItem('BAIDU_NETDISK_ACCESS_TOKEN_VUE', currentToken);
    storedAccessToken.value = currentToken;
    alert('Access Token 已保存!');
    // Fetch info immediately after saving a non-empty token
    fetchAndDisplayUserInfo(); 
  } else {
    localStorage.removeItem('BAIDU_NETDISK_ACCESS_TOKEN_VUE');
    storedAccessToken.value = '';
    userInfo.value = null; // Clear info if token is removed
    quotaInfo.value = null;
    alert('Access Token 已清除!');
  }
  // Keep modal open after save to see the result, user can close manually
  // closeSettingsModal(); 
}

function closeSettingsModal() {
  showSettingsModal.value = false;
}

// --- Helper Functions ---
function vipTypeToString(vipType) {
  switch (vipType) {
    case 0: return '普通用户';
    case 1: return '普通会员';
    case 2: return '超级会员';
    default: return `未知 (${vipType})`;
  }
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
  /* margin-left: auto; Removed to allow settings button to be on the far right */
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

/* Modal Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background-color: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 4px 6px rgba(0,0,0,0.1);
  min-width: 300px; /* Or your preferred width */
  max-width: 500px; /* Or your preferred max-width */
  /* z-index: 1001; /* Generally not needed if overlay has a higher z-index */
}

.modal-content h3 {
  margin-top: 0;
  margin-bottom: 15px;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
}

/* Ensure input takes full width within padding */
.form-group input[type="password"], .form-group input[type="text"] {
  width: calc(100% - 22px); /* 2px for border, 20px for padding (10px each side) */
  padding: 8px 10px;
  border: 1px solid #ccc;
  border-radius: 4px;
  box-sizing: border-box; /* Important for width calculation with padding and border */
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 20px;
}

.modal-actions button {
  padding: 8px 15px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.modal-actions button:nth-child(1) { /* Check Token button */
  background-color: #2196F3; /* Blue */
  color: white;
  margin-right: auto; /* Pushes this button to the left */
}
.modal-actions button:nth-child(2) { /* Save button */
  background-color: #4CAF50;
  color: white;
}

.modal-actions button:nth-child(3) { /* Cancel button */
  background-color: #f44336;
  color: white;
}

/* Add styles for user info display */
.user-info-display {
  margin-top: 15px;
  padding-top: 10px;
  border-top: 1px solid #eee;
}
.user-info-display h4 {
  margin-top: 0;
  margin-bottom: 8px;
  font-size: 0.9em;
  color: #333;
}
.user-info-display p {
  margin: 4px 0;
  font-size: 0.85em;
  color: #555;
}

.info-loading, .info-error {
    margin-top: 10px;
    padding: 8px;
    border-radius: 4px;
    font-size: 0.9em;
}
.info-loading {
    background-color: #e3f2fd;
    color: #1e88e5;
}
.info-error {
    background-color: #ffcdd2;
    color: #c62828;
}
</style> 