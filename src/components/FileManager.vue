<template>
  <div class="file-manager">
    <Toolbar 
      :current-path="currentPath"
      :can-go-up="canGoUp"
      :can-sync-to-netdisk="canSyncToNetdisk"
      @go-up="goUp"
      @sync="syncSelectedToNetdisk"
      @open-settings="openSettingsModalView"
    />
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
          <FileListItem 
            v-else 
            v-for="item in items" 
            :key="item.path" 
            :item="item"
            @item-dblclick="openItem"
            @update:selected="updateItemSelected"
          />
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
import { formatSize } from '../utils/formatters'; // Import formatSize
import FileListItem from './FileListItem.vue'; // Import the new list item component
import { useFileSystem } from '../composables/useFileSystem'; // Import the file system composable
import Toolbar from './Toolbar.vue'; // Import the new Toolbar component

// --- 状态 ---
const showSettingsModal = ref(false); 
const storedAccessToken = ref(''); 

// Initialize the composable for File System operations
const { 
    currentPath,
    items,
    loading,
    error,
    canGoUp,
    initializePath,
    goUp,
    openItemDirectory
} = useFileSystem();

// Initialize the composable for Baidu Netdisk operations
const { 
    uploadFile, 
    // We can also get isLoading and error from the composable if needed for FileManager UI
    // isLoading: baiduUploadIsLoading, 
    // error: baiduUploadError 
} = useBaiduNetdisk(storedAccessToken);

// --- 计算属性 ---
const selectedFiles = computed(() => {
  return items.value.filter(item => item.selected && !item.is_dir);
});

const canSyncToNetdisk = computed(() => {
  return selectedFiles.value.length > 0;
});

// --- 方法 ---
function openItem(item) {
  if (item.is_dir) { 
    openItemDirectory(item);
  } else {
    console.log(`尝试打开文件: ${item.path}`);
    alert(`打开文件: ${item.name}`);
    // TODO: File opening logic
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

// New method to handle selection updates from FileListItem
function updateItemSelected(itemId, newSelectedValue) {
    const itemToUpdate = items.value.find(i => i.path === itemId);
    if (itemToUpdate) {
        itemToUpdate.selected = newSelectedValue;
    }
}

// --- 生命周期钩子 ---
onMounted(() => {
  initializePath();
  storedAccessToken.value = localStorage.getItem('BAIDU_NETDISK_ACCESS_TOKEN_VUE') || '';
});

// --- 新增方法: 同步选中文件到网盘 ---
async function syncSelectedToNetdisk() {
  console.log('syncSelectedToNetdisk called'); // Log: Function start
  const token = storedAccessToken.value; 
  if (!token) {
      alert("请先在设置中配置百度网盘Access Token!");
      openSettingsModalView();
      return;
  }
  console.log('Token check passed:', !!token); // Log: Token check

  if (!canSyncToNetdisk.value) {
    alert("请至少选择一个文件进行同步。");
    return;
  }
  console.log('canSyncToNetdisk check passed:', canSyncToNetdisk.value); // Log: Selection check

  const filesToSync = selectedFiles.value;
  const remoteBaseDir = "/来自FileManger同步";

  // Double-check this selector. Is it definitely the second button in the toolbar?
  const syncButton = document.querySelector('.toolbar button:nth-child(2)'); 
  let originalButtonText = '';
  if (syncButton) {
      console.log('Sync button found, updating UI.'); // Log: Button UI update start
      originalButtonText = syncButton.textContent;
      syncButton.textContent = '同步中...';
      syncButton.disabled = true;
  } else {
      console.warn(`Sync button not found with selector '.toolbar button:nth-child(2)'. UI feedback skipped.`); // Fixed string using template literal
  }

  let successCount = 0;
  let errorCount = 0;

  console.log('Starting upload loop for files:', filesToSync); // Log: Loop start
  for (const file of filesToSync) {
    console.log(`准备上传: ${file.path} 到 ${remoteBaseDir}`);
    try {
      const result = await uploadFile(file.path, remoteBaseDir, token);
      console.log(`上传成功: ${file.name}`, result);
      alert(`文件 '${file.name}' 上传成功!\n响应: ${JSON.stringify(result)}`); 
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

// async function listDirectory(path) { // Moved to useFileSystem
// ... existing code ...

</script>

<style scoped>
/* 微调样式以适应图标 */
.file-manager {
  padding: 1rem;
}
/* .toolbar {
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
  margin-left: auto; 
  background: none;
  border: none;
  cursor: pointer;
  padding: 5px; 
  display: flex;
  align-items: center;
  justify-content: center;
}
.settings-button svg {
  color: #555;
}
.settings-button:hover svg {
  color: #000;
} */

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