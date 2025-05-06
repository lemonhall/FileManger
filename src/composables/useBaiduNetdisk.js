import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue'; // Import ref for potential internal state if needed later

export function useBaiduNetdisk(accessTokenRef) {

  const isLoading = ref(false); // Example: shared loading state for this composable instance
  const error = ref(null);     // Example: shared error state

  async function getUserInfo(tokenToUse) {
    // Prefers tokenToUse if provided, otherwise falls back to accessTokenRef.value
    const token = tokenToUse || accessTokenRef.value;
    if (!token) {
      throw new Error('Access Token is required for getUserInfo');
    }
    isLoading.value = true;
    error.value = null;
    try {
      return await invoke('get_baidu_user_info', { accessToken: token });
    } catch (err) {
      error.value = err;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  async function getQuota(tokenToUse) {
    const token = tokenToUse || accessTokenRef.value;
    if (!token) {
      throw new Error('Access Token is required for getQuota');
    }
    isLoading.value = true;
    error.value = null;
    try {
      return await invoke('get_baidu_quota', { accessToken: token });
    } catch (err) {
      error.value = err;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  async function uploadFile(localPath, remoteDir, tokenToUse) {
    const token = tokenToUse || accessTokenRef.value;
    if (!token) {
      throw new Error('Access Token is required for uploadFile');
    }
    if (!localPath || !remoteDir) {
        throw new Error('Local path and remote directory are required for uploadFile');
    }
    // isLoading and error for upload might be handled per-file or globally,
    // for now, this composable's isLoading/error will reflect the last operation.
    isLoading.value = true;
    error.value = null;
    try {
      return await invoke('upload_file_to_baidupan', {
        localPath: localPath,
        remoteDir: remoteDir,
        accessToken: token
      });
    } catch (err) {
      error.value = err;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    getUserInfo,
    getQuota,
    uploadFile,
    isLoading, // Expose if components want to react to this composable's loading state
    error      // Expose if components want to react to this composable's error state
  };
} 