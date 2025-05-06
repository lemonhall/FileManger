import { invoke } from '@tauri-apps/api/core';
import { ref, computed, readonly } from 'vue'; // Import ref, computed, readonly

const BAIDU_TOKEN_STORAGE_KEY = 'BAIDU_NETDISK_ACCESS_TOKEN_VUE';

// Make the composable a singleton or ensure state is shared if multiple instances are created unintentionally
// For simplicity, assume a single instance for now.

const accessToken = ref('');
const isLoading = ref(false);
const error = ref(null);
const isSyncing = ref(false);
const syncError = ref(null);

// Function to initialize token from localStorage
function initAccessToken() {
    accessToken.value = localStorage.getItem(BAIDU_TOKEN_STORAGE_KEY) || '';
    console.log('[useBaiduNetdisk] Initialized token:', accessToken.value ? '***' : '(empty)');
}

// Function to set the access token
function setAccessToken(newToken) {
    const tokenToSave = newToken ? newToken.trim() : '';
    if (tokenToSave) {
        localStorage.setItem(BAIDU_TOKEN_STORAGE_KEY, tokenToSave);
        accessToken.value = tokenToSave;
        console.log('[useBaiduNetdisk] Token saved.');
        alert('Access Token 已保存!'); // Keep feedback for now
    } else {
        localStorage.removeItem(BAIDU_TOKEN_STORAGE_KEY);
        accessToken.value = '';
        console.log('[useBaiduNetdisk] Token cleared.');
        alert('Access Token 已清除!'); // Keep feedback for now
    }
}

// --- API Call Wrappers ---

async function getUserInfo() {
    const token = accessToken.value;
    if (!token) throw new Error('Access Token is required for getUserInfo');
    isLoading.value = true;
    error.value = null;
    try {
        console.log('[useBaiduNetdisk] Fetching user info...');
        return await invoke('get_baidu_user_info', { accessToken: token });
    } catch (err) {
        console.error('[useBaiduNetdisk] Error fetching user info:', err);
        error.value = err;
        throw err;
    } finally {
        isLoading.value = false;
    }
}

async function getQuota() {
    const token = accessToken.value;
    if (!token) throw new Error('Access Token is required for getQuota');
    isLoading.value = true;
    error.value = null;
    try {
        console.log('[useBaiduNetdisk] Fetching quota...');
        return await invoke('get_baidu_quota', { accessToken: token });
    } catch (err) {
        console.error('[useBaiduNetdisk] Error fetching quota:', err);
        error.value = err;
        throw err;
    } finally {
        isLoading.value = false;
    }
}

// Internal helper for single file upload, might not need to be exposed
async function _uploadSingleFile(localPath, remoteDir) {
    const token = accessToken.value;
    if (!token) throw new Error('Access Token is required for uploadFile');
    if (!localPath || !remoteDir) throw new Error('Local path and remote directory are required');
    
    // Use a local loading/error for this specific upload if needed, 
    // or rely on the syncFiles loop's overall state.
    // For now, just invoke.
    console.log(`[useBaiduNetdisk] Uploading: ${localPath} to ${remoteDir}`);
    return await invoke('upload_file_to_baidupan', {
        localPath: localPath,
        remoteDir: remoteDir,
        accessToken: token
    });
}

// --- Main Functionality ---

async function syncFiles(filesToSync, remoteBaseDir) {
    if (!accessToken.value) {
        alert("请先在设置中配置百度网盘Access Token!"); 
        // Maybe emit an event or return a specific status instead of alert?
        // For now, keep alert, but ideally component handles this via isAvailable check.
        return; 
    }
    if (!filesToSync || filesToSync.length === 0) {
        alert("没有选中任何文件进行同步。");
        return;
    }

    isSyncing.value = true;
    syncError.value = null;
    let successCount = 0;
    let errorCount = 0;
    const totalFiles = filesToSync.length;

    console.log(`[useBaiduNetdisk] Starting sync for ${totalFiles} files to ${remoteBaseDir}`);

    // TODO: Implement UI update for button state externally using isSyncing

    for (let i = 0; i < totalFiles; i++) {
        const file = filesToSync[i];
        // TODO: Update progress (e.g., current file index / total files)
        console.log(`[useBaiduNetdisk] Syncing file ${i + 1}/${totalFiles}: ${file.name}`);
        try {
            const result = await _uploadSingleFile(file.path, remoteBaseDir);
            console.log(`[useBaiduNetdisk] Upload success: ${file.name}`, result);
            // Maybe emit progress event?
            successCount++;
        } catch (err) {
            console.error(`[useBaiduNetdisk] Upload failed: ${file.name}`, err);
            syncError.value = `文件 '${file.name}' 上传失败: ${err}`; // Store the last error
            errorCount++;
            // Decide whether to continue or stop on first error?
            // For now, continue and report summary.
        }
    }

    isSyncing.value = false;
    console.log(`[useBaiduNetdisk] Sync finished. Success: ${successCount}, Failed: ${errorCount}`);

    // TODO: Replace alert with better notification system (e.g., Toast)
    alert(`同步完成! 成功: ${successCount}，失败: ${errorCount}。${errorCount > 0 ? '详情请查看控制台。' : ''}`);

    // Returning status might be useful
    return { successCount, errorCount };
}

// Initialize the token once when the module is first imported
initAccessToken();

// --- Composable Export ---

export function useBaiduNetdisk() {

    // Expose state and methods
    return {
        // State (Readonly recommended for external use where modification isn't intended via direct ref mutation)
        accessToken: readonly(accessToken), // Expose token for display/init purposes if needed
        isAvailable: computed(() => !!accessToken.value), // Is the feature configured?
        isLoading: readonly(isLoading), // Loading state for getUserInfo/getQuota
        error: readonly(error),         // Error state for getUserInfo/getQuota
        isSyncing: readonly(isSyncing), // Sync specific loading state
        syncError: readonly(syncError), // Sync specific error state

        // Methods
        setAccessToken, // Method to update the token
        getUserInfo,
        getQuota,
        syncFiles       // The main method to trigger sync
        // _uploadSingleFile is internal, not exposed
    };
} 