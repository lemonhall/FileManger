import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export function useFileSystem() {
    const currentPath = ref('');
    const items = ref([]);
    const loading = ref(true);
    const error = ref(null);

    const canGoUp = computed(() => {
        // This logic depends on currentPath, so it fits well within this composable
        return currentPath.value && currentPath.value !== '/' && !/^[a-zA-Z]:\\\\?$/.test(currentPath.value);
    });

    async function listDirectory(pathToList) {
        loading.value = true;
        error.value = null;
        console.log(`[useFileSystem] 尝试列出目录: ${pathToList}`);

        try {
            const result = await invoke('list_directory', { path: pathToList });

            const sortedResult = result.sort((a, b) => {
                if (a.is_dir && !b.is_dir) return -1;
                if (!a.is_dir && b.is_dir) return 1;
                return a.name.localeCompare(b.name, 'zh-CN', { sensitivity: 'base', numeric: true });
            });

            items.value = sortedResult.map(item => ({ ...item, selected: false })); // Add selected property here
            currentPath.value = pathToList;
        } catch (e) {
            error.value = `[useFileSystem] 无法加载目录: ${e}`;
            console.error(error.value);
            items.value = [];
            // Optionally reset currentPath or set it to a specific error state?
            // currentPath.value = '?'; // Keep previous path or set to error indicator?
        } finally {
            loading.value = false;
        }
    }

    async function initializePath() {
        // Note: loading/error state is handled by listDirectory called within
        console.log('[useFileSystem] 初始化路径...');
        try {
            const initialPath = await invoke('get_initial_path');
            await listDirectory(initialPath);
        } catch (e) {
            error.value = `[useFileSystem] 无法获取初始路径: ${e}`;
            console.error(error.value);
            currentPath.value = '?'; // Set path to ? on init error
            items.value = [];
            loading.value = false; // Ensure loading is false if init fails here
        }
    }

    function goUp() {
        if (!canGoUp.value) {
            console.warn('[useFileSystem] Cannot go up from:', currentPath.value);
            return;
        }

        const current = currentPath.value;
        const isWindowsPath = current.includes('\\');
        let pathToProcess = current;

        if (pathToProcess.match(/^[a-zA-Z]:\\+$/)) {
            console.warn("[useFileSystem] Already at drive root:", current);
            return;
        } else if (pathToProcess !== '/' && (pathToProcess.endsWith('\\') || pathToProcess.endsWith('/'))) {
            pathToProcess = pathToProcess.substring(0, pathToProcess.length - 1);
        }

        const parts = pathToProcess.split(isWindowsPath ? /\\/ : /\//);

        if (parts.length <= 1 && isWindowsPath && /^[a-zA-Z]:$/.test(parts[0])) {
            if (parts[0].endsWith(':')) {
                listDirectory(parts[0] + '\\');
                return;
            }
        } else if (parts.length === 1 && !isWindowsPath && parts[0] === '') {
            listDirectory('/');
            return;
        }

        parts.pop();

        let parentPath;
        if (isWindowsPath) {
            if (parts.length === 1 && /^[a-zA-Z]:$/.test(parts[0])) {
                parentPath = parts[0] + '\\';
            } else {
                parentPath = parts.join('\\');
            }
        } else {
            if (parts.length === 1 && parts[0] === '') {
                parentPath = '/';
            } else {
                parentPath = parts.join('/');
                if (parentPath === '' && current.startsWith('/')) {
                    parentPath = '/';
                }
            }
        }

        if (!parentPath) {
            if (!isWindowsPath && current.startsWith('/')) {
                parentPath = '/';
            } else {
                console.warn("[useFileSystem] Could not determine parent path for:", current, "Resulting parts:", parts);
                return;
            }
        }
        listDirectory(parentPath);
    }

    function openItemDirectory(item) {
        if (item && item.is_dir) {
            listDirectory(item.path);
        }
    }

    // Expose reactive state and methods
    return {
        currentPath,
        items,
        loading,
        error,
        canGoUp, // Expose computed property
        listDirectory, // Expose listDirectory if needed externally (e.g., refresh button)
        initializePath,
        goUp,
        openItemDirectory
    };
} 