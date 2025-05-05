<template>
  <div class="file-manager">
    <div class="toolbar">
      <button @click="goUp" :disabled="!canGoUp">向上</button>
      <span class="current-path">当前路径: {{ currentPath }}</span>
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
          </tr>
        </thead>
        <tbody>
          <tr v-if="items.length === 0">
             <td colspan="4">文件夹为空</td>
          </tr>
          <tr v-else v-for="item in items" :key="item.path" @dblclick="openItem(item)">
            <td><input type="checkbox" v-model="item.selected" @click.stop /></td> <!-- Prevent dblclick propagation -->
            <td>{{ item.name }}</td>
            <td>{{ item.is_dir ? '文件夹' : '文件' }}</td> <!-- Use is_dir from Rust -->
            <td>{{ formatSize(item.size) }}</td>
          </tr>
        </tbody>
      </table>
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

// --- 计算属性 ---
const canGoUp = computed(() => {
    // Basic check, needs improvement for root paths (C:\, /)
    return currentPath.value && currentPath.value !== '/' && !/^[a-zA-Z]:\\?$/.test(currentPath.value);
});

// --- 方法 ---
async function listDirectory(path) {
  loading.value = true;
  error.value = null;
  console.log(`尝试列出目录: ${path}`);

  try {
    const result = await invoke('list_directory', { path: path });
    items.value = result.map(item => ({ ...item, selected: false }));
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

    const parts = currentPath.value.replace(/\\$/, '').split(/[\/]/);
    if (parts.length <= 1 && !/^[a-zA-Z]:$/.test(parts[0])) {
        return;
    }
    if (parts.length === 1 && /^[a-zA-Z]:$/.test(parts[0])) {
        return;
    }

    parts.pop();
    let parentPath = parts.join(currentPath.value.includes('\\') ? '\\' : '/');

    if (parentPath === '') {
        parentPath = '/';
    }
    else if (/^[a-zA-Z]:$/.test(parentPath)) {
        parentPath += '\\';
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
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
    if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`;
    return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

// --- 生命周期钩子 ---
onMounted(() => {
  initializePath();
});

</script>

<style scoped>
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
}
.file-list th {
  background-color: #f2f2f2;
}
.file-list tbody tr:hover {
  background-color: #f5f5f5;
  cursor: pointer;
}
</style> 