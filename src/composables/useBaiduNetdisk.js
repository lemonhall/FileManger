import { invoke } from '@tauri-apps/api/core';
import { ref, computed, readonly } from 'vue'; // Import ref, computed, readonly
import { push } from 'notivue'; // Import Notivue push
import { useUploadTimestamps } from './useUploadTimestamps'; // Import the new composable

const BAIDU_TOKEN_STORAGE_KEY = 'BAIDU_NETDISK_ACCESS_TOKEN_VUE';

// Make the composable a singleton or ensure state is shared if multiple instances are created unintentionally
// For simplicity, assume a single instance for now.

const accessToken = ref('');
const isLoading = ref(false);
const error = ref(null);
const isSyncing = ref(false);
const syncError = ref(null);

// Get the timestamp update function from the other composable
// We call useUploadTimestamps() here at the module level to get a shared instance of its state and methods.
const { updateFileTimestamp: recordFileUploadedTimestamp } = useUploadTimestamps();

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
        push.success('Access Token 已保存!'); 
    } else {
        localStorage.removeItem(BAIDU_TOKEN_STORAGE_KEY);
        accessToken.value = '';
        console.log('[useBaiduNetdisk] Token cleared.');
        push.info('Access Token 已清除!'); 
    }
}

// --- API Call Wrappers ---

async function getUserInfo(tokenOverride) {
    const token = tokenOverride || accessToken.value;
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

async function getQuota(tokenOverride) {
    const token = tokenOverride || accessToken.value;
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
        push.warning("请先在设置中配置百度网盘Access Token!"); 
        return { successCount: 0, errorCount: filesToSync?.length || 0 }; // Return status
    }
    if (!filesToSync || filesToSync.length === 0) {
        push.warning("没有选中任何文件进行同步。");
        return { successCount: 0, errorCount: 0 }; // Return status
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
        console.log(`[useBaiduNetdisk] Syncing file ${i + 1}/${totalFiles}: ${file.name} (Path: ${file.path})`);
        try {
            await _uploadSingleFile(file.path, remoteBaseDir);
            console.log(`[useBaiduNetdisk] Upload success: ${file.name}`);
            successCount++;
            // After successful upload, record the timestamp
            await recordFileUploadedTimestamp(file.path, Date.now()); 
        } catch (err) {
            console.error(`[useBaiduNetdisk] Upload failed for ${file.name} (Path: ${file.path}):`, err);
            syncError.value = `文件 '${file.name}' 上传失败: ${err}`;
            errorCount++;
            // Decide whether to continue or stop on first error?
            // For now, continue and report summary.
        }
    }

    isSyncing.value = false;
    console.log(`[useBaiduNetdisk] Sync finished. Success: ${successCount}, Failed: ${errorCount}`);

    if (errorCount === 0 && successCount > 0) {
        push.success(`同步完成! ${successCount}个文件全部成功。`);
    } else if (successCount > 0 && errorCount > 0) {
        push.info(`同步部分完成。成功: ${successCount}，失败: ${errorCount}。详情请查看控制台。`);
    } else if (errorCount > 0 && successCount === 0) {
        push.error(`同步失败! ${errorCount}个文件全部失败。详情请查看控制台。`);
    } else if (successCount === 0 && errorCount === 0 && totalFiles > 0) {
        // This case might occur if pre-flight checks failed for all, though unlikely with current structure
        push.warning("没有文件成功同步，也未报告错误。"); 
    } else {
        // This case (no files to sync) is handled by the initial check, but as a fallback:
        // push.info("没有文件被同步。"); // Already handled
    }

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