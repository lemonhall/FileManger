<template>
  <div class="file-manager">
    <Toolbar 
      :current-path="currentPath"
      :can-go-up="canGoUp"
      :can-sync-to-netdisk="canSyncToNetdisk"
      @go-up="goUp"
      @sync="handleFileSync" 
      @open-settings="openSettingsModalView"
    />
    <FileList 
      :items="items"
      :loading="loading"
      :error="error"
      @item-dblclick="openItem"
      @update:item-selected="updateItemSelected"
    />

    <SettingsModal 
      :show="showSettingsModal" 
      @close="handleCloseSettingsModal"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
// import { invoke } from '@tauri-apps/api/core'; // No longer directly used here for FS ops
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
import FileList from './FileList.vue'; // Import the new FileList component

// --- 状态 ---
const showSettingsModal = ref(false); 
// const storedAccessToken = ref(''); // No longer managed here

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
    // uploadFile, // uploadFile seems not used directly here, syncFiles is preferred
    isAvailable: isNetdiskAvailable, 
    syncFiles: baiduSyncFiles, // Renamed to avoid potential naming clashes if we had a local syncFiles
} = useBaiduNetdisk();

// --- 计算属性 ---
const selectedFiles = computed(() => {
  return items.value.filter(item => item.selected && !item.is_dir);
});

const canSyncToNetdisk = computed(() => {
  return isNetdiskAvailable.value && selectedFiles.value.length > 0;
});

// --- 方法 ---
function openItem(item) {
  if (item.is_dir) { 
    openItemDirectory(item);
  } else {
    console.log(`尝试打开文件: ${item.path}`);
    // TODO: File opening logic
  }
}

async function handleFileSync() {
  const filesToProcess = [...selectedFiles.value]; // Create a snapshot of currently selected files

  // The check for empty filesToProcess is already handled inside baiduSyncFiles,
  // which will show a warning notification. So, no need to duplicate it here unless
  // we want different behavior.

  const remoteDir = '/来自FileManger同步'; // Currently hardcoded, as it was in the template
  const result = await baiduSyncFiles(filesToProcess, remoteDir);

  if (result && result.successCount > 0 && result.errorCount === 0) {
    // Only deselect if there were successes AND no errors.
    // This implies all files in filesToProcess were successful if result.successCount == filesToProcess.length
    if (result.successCount === filesToProcess.length) {
      const pathsToDeselect = new Set(filesToProcess.map(f => f.path));
      items.value.forEach(item => {
        if (pathsToDeselect.has(item.path)) {
          item.selected = false;
        }
      });
    }
  }
  // If there are errors, or partial success, items remain selected.
  // The user gets a toast notification about the outcome from useBaiduNetdisk.
}

// --- Settings Modal Methods ---
function openSettingsModalView() {
  showSettingsModal.value = true;
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
});

</script>

<style scoped>
/* 微调样式以适应图标 */
.file-manager {
  padding: 1rem;
}

</style> 