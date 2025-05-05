<template>
  <div class="file-manager">
    <div class="toolbar">
      <button @click="goUp" :disabled="!canGoUp">向上</button>
      <span class="current-path">当前路径: {{ currentPath }}</span>
    </div>
    <div class="file-list">
      <table>
        <thead>
          <tr>
            <th></th> <!-- Checkbox -->
            <th>名称</th>
            <th>类型</th>
            <th>大小</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="loading">
            <td colspan="4">加载中...</td>
          </tr>
          <tr v-else-if="error">
             <td colspan="4" class="error-message">错误: {{ error }}</td>
          </tr>
          <tr v-else v-for="item in items" :key="item.path" @dblclick="openItem(item)">
            <td><input type="checkbox" v-model="item.selected" /></td>
            <td>{{ item.name }}</td>
            <td>{{ item.isDir ? '文件夹' : '文件' }}</td>
            <td>{{ formatSize(item.size) }}</td>
          </tr>
          <tr v-if="!loading && !error && items.length === 0">
             <td colspan="4">文件夹为空</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
// 注意：invoke 函数需要从 '@tauri-apps/api/tauri' 导入
// import { invoke } from '@tauri-apps/api/tauri';

// --- 状态 ---
const currentPath = ref('/'); // 初始路径，后续需要根据平台调整
const items = ref([]); // 文件/文件夹列表
const loading = ref(false);
const error = ref(null);

// --- 计算属性 ---
const canGoUp = computed(() => {
  // 简单的判断，后续需要完善跨平台路径处理
  return currentPath.value !== '/' && currentPath.value.length > 1;
});

// --- 方法 ---
async function listDirectory(path) {
  loading.value = true;
  error.value = null;
  items.value = []; // 清空旧列表
  console.log(`尝试列出目录: ${path}`); // 调试信息

  try {
    // --- TODO: 调用 Tauri 命令 ---
    // const result = await invoke('list_directory', { path: path });
    // items.value = result.map(item => ({ ...item, selected: false })); // 假设后端返回 { name, path, isDir, size } 结构

    // --- 临时 Mock 数据 ---
    await new Promise(resolve => setTimeout(resolve, 500)); // 模拟网络延迟
     if (path === '/') {
         items.value = [
             { name: 'Documents', path: '/Documents', isDir: true, size: null, selected: false },
             { name: 'Downloads', path: '/Downloads', isDir: true, size: null, selected: false },
             { name: 'image.png', path: '/image.png', isDir: false, size: 1024 * 5, selected: false },
             { name: 'config.txt', path: '/config.txt', isDir: false, size: 256, selected: false },
         ];
     } else if (path === '/Documents') {
         items.value = [
             { name: 'report.docx', path: '/Documents/report.docx', isDir: false, size: 1024 * 120, selected: false },
             { name: 'notes', path: '/Documents/notes', isDir: true, size: null, selected: false },
         ];
     } else if (path === '/Downloads') {
         items.value = []; // 模拟空文件夹
     } else if (path === '/Documents/notes') {
         items.value = [
              { name: 'meeting.txt', path: '/Documents/notes/meeting.txt', isDir: false, size: 512, selected: false }
         ];
     } else {
         // 模拟找不到路径或非文件夹路径
         error.value = `路径 "${path}" 无法访问或不是文件夹。`;
         console.error(error.value); // 调试输出
     }

  } catch (e) {
    error.value = `无法加载目录: ${e}`;
    console.error(error.value); // 调试输出
  } finally {
    loading.value = false;
  }
}

function goUp() {
  if (!canGoUp.value) return;
  // 简单的向上逻辑，后续需要替换为可靠的路径处理
  const parts = currentPath.value.split(/[\/]/).filter(p => p); // 分割路径并移除空部分
  parts.pop(); // 移除最后一部分
  const parentPath = parts.length === 0 ? '/' : '/' + parts.join('/'); // 重新组合，根目录特殊处理
  currentPath.value = parentPath;
  listDirectory(currentPath.value);
}

function openItem(item) {
  if (item.isDir) {
    currentPath.value = item.path; // 直接使用项的路径
    listDirectory(currentPath.value);
  } else {
    // TODO: 处理打开文件的逻辑 (例如，调用 Tauri 命令)
    console.log(`尝试打开文件: ${item.path}`);
    alert(`打开文件: ${item.name}`);
  }
}

function formatSize(size) {
    if (size === null || size === undefined) return '-';
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
    if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`;
    return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

// --- 生命周期钩子 ---
onMounted(() => {
  listDirectory(currentPath.value); // 组件挂载后加载初始路径
});

</script>

<style scoped>
.file-manager {
  font-family: sans-serif;
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

.file-list table {
  width: 100%;
  border-collapse: collapse;
}

.file-list th, .file-list td {
  border: 1px solid #ddd;
  padding: 8px;
  text-align: left;
}

.file-list th {
  background-color: #f2f2f2;
}

.file-list tbody tr:hover {
  background-color: #f5f5f5;
  cursor: pointer;
}

.error-message {
    color: red;
    font-weight: bold;
}
</style> 