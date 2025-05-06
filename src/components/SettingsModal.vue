<template>
  <div v-if="show" class="modal-overlay" @click.self="closeModal">
    <div class="modal-content">
      <h3>百度网盘设置</h3>
      <div class="form-group">
        <label for="baidu-token">Access Token:</label>
        <input type="password" id="baidu-token" v-model="accessTokenInput" placeholder="请输入百度网盘Access Token">
      </div>

      <div v-if="isCheckingOrRefreshing || baiduApiOverallLoading" class="info-loading">正在获取信息...</div>
      <!-- Display composable's error if it's set (e.g. from invoke failure) -->
      <div v-if="!isCheckingOrRefreshing && baiduApiOverallError" class="info-error">获取信息时出错: {{ baiduApiOverallError }}</div>
      
      <div v-if="!isCheckingOrRefreshing && !baiduApiOverallError && (userInfo || quotaInfo)" class="user-info-display">
        <h4 v-if="userInfo">用户信息</h4>
        <p v-if="userInfo">用户名: {{ userInfo.baidu_name }} ({{ userInfo.netdisk_name }})</p>
        <p v-if="userInfo">VIP类型: {{ vipTypeToString(userInfo.vip_type) }}</p>
        <p v-if="userInfo"><img :src="userInfo.avatar_url" alt="avatar" width="30" height="30" style="vertical-align: middle; border-radius: 50%;"></p>
        <h4 v-if="quotaInfo">存储配额</h4>
        <p v-if="quotaInfo">{{ formatSize(quotaInfo.used) }} / {{ formatSize(quotaInfo.total) }}</p>
        <progress v-if="quotaInfo" :value="quotaInfo.used" :max="quotaInfo.total" style="width: 100%;"></progress>
      </div>
      <div v-if="!isCheckingOrRefreshing && !baiduApiOverallError && !userInfo && !quotaInfo && accessTokenInput.trim()" class="info-hint">
        输入Token后，请点击下方按钮获取信息。
      </div>

      <div class="modal-actions">
        <button @click="handleCheckTokenFromInput" :disabled="!accessTokenInput.trim() || isCheckingOrRefreshing || baiduApiOverallLoading">
          {{ (isCheckingOrRefreshing || baiduApiOverallLoading) ? '检查中...' : '检查Token并获取信息' }}
        </button>
        <button @click="handleSave">保存</button>
        <button @click="closeModal">取消</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, defineProps, defineEmits } from 'vue';
import { useBaiduNetdisk } from '../composables/useBaiduNetdisk';
import { formatSize, vipTypeToString } from '../utils/formatters';
import { push } from 'notivue';

const USER_INFO_CACHE_KEY = 'BAIDU_USER_INFO_CACHE';
const QUOTA_INFO_CACHE_KEY = 'BAIDU_QUOTA_INFO_CACHE';

const props = defineProps({
  show: Boolean
});
const emit = defineEmits(['close']);

const accessTokenInput = ref('');
const userInfo = ref(null);
const quotaInfo = ref(null);
const isCheckingOrRefreshing = ref(false); // For button loading state

const { 
    accessToken, 
    setAccessToken, 
    getUserInfo, 
    getQuota, 
    isLoading: baiduApiOverallLoading, // Renamed to distinguish from local loading
    error: baiduApiOverallError      // Renamed to distinguish
} = useBaiduNetdisk();

function clearDisplayAndCache() {
  console.log('[SettingsModal] Clearing display and cache for user/quota info.');
  userInfo.value = null;
  quotaInfo.value = null;
  localStorage.removeItem(USER_INFO_CACHE_KEY);
  localStorage.removeItem(QUOTA_INFO_CACHE_KEY);
}

function loadInfoFromCache() {
  console.log('[SettingsModal] Attempting to load user/quota info from cache.');
  try {
    const cachedUserInfo = localStorage.getItem(USER_INFO_CACHE_KEY);
    if (cachedUserInfo) userInfo.value = JSON.parse(cachedUserInfo);
    const cachedQuotaInfo = localStorage.getItem(QUOTA_INFO_CACHE_KEY);
    if (cachedQuotaInfo) quotaInfo.value = JSON.parse(cachedQuotaInfo);
    console.log('[SettingsModal] Loaded from cache - UserInfo:', userInfo.value, 'QuotaInfo:', quotaInfo.value);
  } catch (e) {
    console.error('[SettingsModal] Error loading info from cache:', e);
    clearDisplayAndCache(); // Clear if cache is corrupted
  }
}

async function fetchAndDisplayOnlineInfo(tokenToUse) {
  if (!tokenToUse) {
    console.warn('[SettingsModal] fetchAndDisplayOnlineInfo called without a token.');
    // If no token, ensure UI is clear, might have been called by auto-refresh when token was removed.
    clearDisplayAndCache();
    return;
  }
  console.log(`[SettingsModal] Fetching online info with token: ${tokenToUse ? '***' : '(empty)'}`);
  isCheckingOrRefreshing.value = true;
  if (baiduApiOverallError.value) baiduApiOverallError.value = null; // Clear previous general API error

  let fetchErrorOccurred = false;

  try {
    const [infoResult, quotaResult] = await Promise.allSettled([
        getUserInfo(tokenToUse),
        getQuota(tokenToUse)
    ]);

    if (infoResult.status === 'fulfilled') {
      userInfo.value = infoResult.value;
      localStorage.setItem(USER_INFO_CACHE_KEY, JSON.stringify(infoResult.value));
    } else {
      console.error('[SettingsModal] Fetch user info failed:', infoResult.reason);
      userInfo.value = null;
      localStorage.removeItem(USER_INFO_CACHE_KEY);
      fetchErrorOccurred = true;
    }

    if (quotaResult.status === 'fulfilled') {
      quotaInfo.value = quotaResult.value;
      localStorage.setItem(QUOTA_INFO_CACHE_KEY, JSON.stringify(quotaResult.value));
    } else {
      console.error('[SettingsModal] Fetch quota info failed:', quotaResult.reason);
      quotaInfo.value = null;
      localStorage.removeItem(QUOTA_INFO_CACHE_KEY);
      fetchErrorOccurred = true;
    }

    if (fetchErrorOccurred) {
      push.error("获取部分或全部账户信息失败。");
      // baiduApiOverallError will be set by the composable if the API call itself failed at invoke level
    } else {
      push.success("账户信息已刷新。");
    }

  } catch (e) {
    // This catch is for unexpected errors in Promise.allSettled or subsequent logic
    console.error('[SettingsModal] Unexpected error in fetchAndDisplayOnlineInfo:', e);
    push.error("刷新信息时发生意外错误。");
    clearDisplayAndCache(); // Clear everything on unexpected error
    if (baiduApiOverallError) baiduApiOverallError.value = `刷新信息时发生意外错误: ${e.toString()}`;
  }
  isCheckingOrRefreshing.value = false;
}

// Renamed from checkTokenAndFetchInfo to be more specific for the button's action
async function handleCheckTokenFromInput() {
  const tokenFromInput = accessTokenInput.value.trim();
  if (!tokenFromInput) {
    push.warning("请输入有效的 Access Token!");
    return;
  }
  await fetchAndDisplayOnlineInfo(tokenFromInput);
}

watch(() => props.show, (isVisible) => {
  console.log(`[SettingsModal] Show changed: ${isVisible}`);
  if (isVisible) {
    accessTokenInput.value = accessToken.value || '';
    loadInfoFromCache();
    // Auto-fetch if token exists (from composable) and cache is incomplete, or always refresh if desired.
    // For now, let's refresh if token exists and any part of the cache is missing.
    if (accessToken.value && (!userInfo.value || !quotaInfo.value)) {
        console.log('[SettingsModal] Modal visible, token exists, cache incomplete. Auto-fetching info.');
        fetchAndDisplayOnlineInfo(accessToken.value);
    }
  } else {
    isCheckingOrRefreshing.value = false; // Reset button loading state if modal is closed
  }
});

watch(accessToken, (newAuthToken, oldAuthToken) => {
  console.log(`[SettingsModal] Watched accessToken changed. New: ${newAuthToken ? '***' : '(empty)'}, Old: ${oldAuthToken ? '***' : '(empty)'}`);
  accessTokenInput.value = newAuthToken || '';
  // Avoid clearing and re-fetching on initial component setup if oldAuthToken is undefined
  if (oldAuthToken !== undefined && newAuthToken !== oldAuthToken) {
    console.log('[SettingsModal] AccessToken effectively changed. Clearing cache and potentially re-fetching.');
    clearDisplayAndCache();
    if (newAuthToken && props.show) {
      fetchAndDisplayOnlineInfo(newAuthToken);
    }
  }
});

function handleSave() {
  const tokenToSave = accessTokenInput.value.trim();
  setAccessToken(tokenToSave); // This will trigger the accessToken watcher if the token value changes
  closeModal();
}

function closeModal() {
  emit('close');
}

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