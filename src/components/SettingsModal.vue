<template>
  <div v-if="show" class="modal-overlay" @click.self="closeModal">
    <div class="modal-content">
      <h3>百度网盘设置</h3>
      <div class="form-group">
        <label for="baidu-token">Access Token:</label>
        <input type="password" id="baidu-token" v-model="accessTokenInput" placeholder="请输入百度网盘Access Token">
      </div>

      <!-- User Info Display Area -->
      <div v-if="baiduApiIsLoading" class="info-loading">正在获取信息...</div>
      <div v-if="baiduApiError" class="info-error">错误: {{ baiduApiError }}</div>
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
        <button @click="checkTokenAndFetchInfo" :disabled="!accessTokenInput.trim() || baiduApiIsLoading">检查Token并获取信息</button>
        <button @click="handleSave">保存</button>
        <button @click="closeModal">取消</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, defineProps, defineEmits, onMounted } from 'vue';
// import { invoke } from '@tauri-apps/api/core'; // No longer directly used here
// import { useBaiduNetdisk } from '../composables/useBaiduNetdisk'; // Now use it internally
import { useBaiduNetdisk } from '../composables/useBaiduNetdisk';
import { formatSize, vipTypeToString } from '../utils/formatters'; // Import formatters

// const props = defineProps({ // No longer need initialAccessToken
//   show: Boolean,
//   initialAccessToken: String 
// });
const props = defineProps({
  show: Boolean
});

// const emit = defineEmits(['close', 'save-token']); // No longer emit save-token
const emit = defineEmits(['close']);

const accessTokenInput = ref('');
const userInfo = ref(null);
const quotaInfo = ref(null);
// isLoading and error state will now come from the composable instance

// Use the BaiduNetdisk composable internally
const { 
    accessToken, // Get the reactive token state
    setAccessToken, // Get the method to update the token
    getUserInfo, 
    getQuota, 
    isLoading: baiduApiIsLoading, 
    error: baiduApiError
} = useBaiduNetdisk();

// Initialize input field when the component mounts or when the token changes
// Watch the token from the composable
watch(accessToken, (newValue) => {
    accessTokenInput.value = newValue || '';
}, { immediate: true }); 

// Optionally reset local UI state when modal becomes visible
watch(() => props.show, (newVal) => {
    if (newVal) {
        // Reset potential stale info from previous opening
        userInfo.value = null;
        quotaInfo.value = null;
        // Clear error from composable if it was set previously
        if (baiduApiError.value) baiduApiError.value = null; 
    } else {
         // Maybe reset input field to current token value when closing?
         // Or just let it reflect the last input until next open.
         // Let's reset it to the actual stored value on close for consistency.
         accessTokenInput.value = accessToken.value || '';
    }
});


function closeModal() {
  emit('close');
}

async function checkTokenAndFetchInfo() {
  const token = accessTokenInput.value.trim();
  if (!token) {
    alert("请输入有效的 Access Token!");
    return;
  }

  userInfo.value = null; // Still need to clear local display data
  quotaInfo.value = null; // Still need to clear local display data
  if (baiduApiError.value) baiduApiError.value = null; // Clear previous error from composable before new calls

  try {
    // Now using the composable methods. 
    // The composable's isLoading and error refs will be updated automatically.
    // We pass the token explicitly here to ensure the most current input value is used for the check.
    const [infoResult, quotaResult] = await Promise.allSettled([
        getUserInfo(token),
        getQuota(token)
    ]);

    let errors = []; // Keep local error accumulation for this specific operation
    if (infoResult.status === 'fulfilled') {
      userInfo.value = infoResult.value;
    } else {
      errors.push(`获取用户信息失败: ${infoResult.reason}`);
    }

    if (quotaResult.status === 'fulfilled') {
      quotaInfo.value = quotaResult.value;
    } else {
      errors.push(`获取配额信息失败: ${quotaResult.reason}`);
    }

    if (errors.length > 0) {
      // error.value = errors.join('; \n'); // This would set the local error ref if we still had one.
                                          // Instead, we should set the error ref from the composable if we want to display these combined errors.
                                          // However, the composable's error ref is typically for errors from the invoke calls themselves.
                                          // For now, let's display these specific errors in the UI if they occur, perhaps by setting baiduApiError directly
                                          // or by having a separate local error display for this aggregate operation.
      // For simplicity, if any part fails, the individual error is already in baiduApiError from the composable.
      // If we want a combined message, we can assign to baiduApiError here:
      if (errors.length > 0 && baiduApiError) { // Check if baiduApiError is writable
        baiduApiError.value = errors.join('; \n');
      }
    }

  } catch (e) {
    // error.value = `获取信息时发生意外错误: ${e}`; // This catch is for the Promise.allSettled itself or other synchronous errors.
                                                  // The composable functions will catch their own invoke errors and set baiduApiError.
    // So, if an error occurs here, it's likely an unexpected issue outside the API calls.
    if (baiduApiError) baiduApiError.value = `获取信息时发生意外错误: ${e}`;
  }
}

function handleSave() { // Renamed from saveToken
  const currentToken = accessTokenInput.value.trim();
  // Call the composable's method to save the token
  setAccessToken(currentToken);
  // Feedback alert is now inside setAccessToken in the composable
  // emit('save-token', currentToken); // No longer needed
  closeModal(); // Close modal after saving
}

// Helper functions (can be moved to utils if used elsewhere)
// function vipTypeToString(vipType) { // Moved to formatters.js
//   switch (vipType) {
//     case 0: return '普通用户';
//     case 1: return '普通会员';
//     case 2: return '超级会员';
//     default: return `未知 (${vipType})`;
//   }
// }

// function formatSize(size) { // Moved to formatters.js
//     if (size === null || size === undefined) return '-';
//     const numSize = Number(size);
//     if (isNaN(numSize)) return '-';
//     if (numSize < 1024) return `${numSize} B`;
//     if (numSize < 1024 * 1024) return `${(numSize / 1024).toFixed(1)} KB`;
//     if (numSize < 1024 * 1024 * 1024) return `${(numSize / (1024 * 1024)).toFixed(1)} MB`;
//     return `${(numSize / (1024 * 1024 * 1024)).toFixed(1)} GB`;
// }

</script>

<style scoped>
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
  min-width: 300px;
  max-width: 500px;
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

.form-group input[type="password"], .form-group input[type="text"] {
  width: calc(100% - 22px); 
  padding: 8px 10px;
  border: 1px solid #ccc;
  border-radius: 4px;
  box-sizing: border-box; 
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

.modal-actions button:nth-child(1) { 
  background-color: #2196F3; 
  color: white;
  margin-right: auto; 
}
.modal-actions button:nth-child(2) { 
  background-color: #4CAF50;
  color: white;
}

.modal-actions button:nth-child(3) { 
  background-color: #f44336;
  color: white;
}

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