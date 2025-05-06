import { ref, readonly } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { push } from 'notivue';

const timestampsMap = ref({}); // Store: { [filePath: string]: number (timestamp) }
const isLoading = ref(false);
const error = ref(null);

export function useUploadTimestamps() {

    async function loadAllTimestamps() {
        console.log('[useUploadTimestamps] Loading all timestamps...');
        isLoading.value = true;
        error.value = null;
        try {
            const allTimestamps = await invoke('get_all_upload_timestamps');
            timestampsMap.value = allTimestamps || {}; // Ensure it's an object even if null/undefined is returned
            console.log('[useUploadTimestamps] Timestamps loaded:', Object.keys(allTimestamps || {}).length, 'entries');
            // push.success('上传时间记录已加载。'); // Optional: notification for successful load
        } catch (err) {
            console.error('[useUploadTimestamps] Error loading all timestamps:', err);
            error.value = `加载上传时间记录失败: ${err}`;
            push.error(error.value);
            timestampsMap.value = {}; // Reset on error
        }
        isLoading.value = false;
    }

    async function updateFileTimestamp(filePath, timestamp) {
        if (!filePath || typeof timestamp !== 'number') {
            console.error('[useUploadTimestamps] Invalid arguments for updateFileTimestamp:', filePath, timestamp);
            push.error('更新文件上传时间戳时参数无效。');
            return;
        }
        console.log(`[useUploadTimestamps] Updating timestamp for ${filePath} to ${timestamp}`);
        // No separate loading/error for this individual update, assume it's quick
        try {
            await invoke('set_upload_timestamp', { filePath, timestamp });
            // Update local cache optimistically or after confirmation
            timestampsMap.value = {
                ...timestampsMap.value,
                [filePath]: timestamp,
            };
            console.log(`[useUploadTimestamps] Timestamp updated for ${filePath}`);
            // push.success(`文件 ${filePath.split('/').pop()} 的上传时间已记录。`); // This might be too noisy
        } catch (err) {
            console.error(`[useUploadTimestamps] Error setting timestamp for ${filePath}:`, err);
            // Decide if a global error ref for this composable should be set
            push.error(`记录文件 ${filePath.split('/').pop() || ''} 的上传时间失败。`);
        }
    }

    // Function to get a specific timestamp, reacting to changes in the map
    function getTimestampForFile(filePath) {
        return timestampsMap.value[filePath] || null;
    }

    return {
        timestampsMap: readonly(timestampsMap), // filePath -> timestamp
        isLoading: readonly(isLoading),
        error: readonly(error),
        loadAllTimestamps,
        updateFileTimestamp,
        getTimestampForFile
    };
} 