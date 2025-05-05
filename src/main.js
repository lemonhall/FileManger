import { invoke } from '@tauri-apps/api/core'; // Correct import path for Tauri v2

let currentPath = '';
const fileListBody = document.getElementById('file-list-body');
const currentPathSpan = document.getElementById('current-path');
const goUpButton = document.getElementById('go-up');
const loadingIndicator = document.getElementById('loading');
const errorMessageElement = document.getElementById('error-message');

function setLoading(isLoading) {
    if (loadingIndicator) {
        loadingIndicator.style.display = isLoading ? 'block' : 'none';
    }
    if (fileListBody && !isLoading) {
         // Hide loading row if present
        const loadingRow = fileListBody.querySelector('tr[data-loading="true"]');
        if (loadingRow) loadingRow.remove();
    }
}

function setError(message) {
    if (errorMessageElement) {
        errorMessageElement.textContent = message || '';
        errorMessageElement.style.display = message ? 'block' : 'none';
    }
     if (fileListBody) {
        fileListBody.innerHTML = ''; // Clear list on error
        if (message) {
             const row = fileListBody.insertRow();
             const cell = row.insertCell();
             cell.colSpan = 4;
             cell.className = 'error-message';
             cell.textContent = `错误: ${message}`;
        }
    }
}

function formatSize(size) {
    if (size === null || size === undefined) return '-';
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
    if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`;
    return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function renderFileList(items) {
    if (!fileListBody) return;
    fileListBody.innerHTML = ''; // Clear previous entries
    setError(null); // Clear any previous error

    if (items.length === 0) {
        const row = fileListBody.insertRow();
        const cell = row.insertCell();
        cell.colSpan = 4;
        cell.textContent = '文件夹为空';
        return;
    }

    items.forEach(item => {
        const row = fileListBody.insertRow();
        row.dataset.path = item.path;
        row.dataset.isDir = item.is_dir;
        row.style.cursor = 'pointer';

        row.addEventListener('dblclick', () => openItem(item));

        const selectCell = row.insertCell();
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.dataset.path = item.path; // Add path for selection tracking
        selectCell.appendChild(checkbox);

        const nameCell = row.insertCell();
        nameCell.textContent = item.name;

        const typeCell = row.insertCell();
        typeCell.textContent = item.is_dir ? '文件夹' : '文件';

        const sizeCell = row.insertCell();
        sizeCell.textContent = formatSize(item.size);
    });
}

async function listDirectory(path) {
    setLoading(true);
    setError(null);
    console.log(`尝试列出目录: ${path}`);

    // Update UI
    if (currentPathSpan) {
        currentPathSpan.textContent = path;
    }
    if (goUpButton) {
        // Basic check, needs improvement for root paths (C:\, /)
        goUpButton.disabled = !path || path === '/' || /^[a-zA-Z]:\\?$/.test(path);
    }

    try {
        // Use the correctly imported invoke
        const result = await invoke('list_directory', { path: path });
        currentPath = path; // Update current path only on success
        renderFileList(result);
    } catch (e) {
        console.error(`无法加载目录: ${e}`);
        setError(`无法加载目录: ${e}`);
        // Optionally reset path or show previous state?
        // currentPathSpan.textContent = currentPath; // Revert path display?
    } finally {
        setLoading(false);
    }
}

async function initializePath() {
    setLoading(true);
    setError(null);
    try {
        // Use the correctly imported invoke
        const initialPath = await invoke('get_initial_path');
        await listDirectory(initialPath);
    } catch (e) {
        console.error(`无法获取初始路径: ${e}`);
        setError(`无法获取初始路径: ${e}`);
        if (currentPathSpan) currentPathSpan.textContent = '?';
        setLoading(false);
    }
}

function goUp() {
    if (!currentPath) return;

    // Basic up logic, needs refinement
    const parts = currentPath.replace(/\\$/, '').split(/[\/]/);
    if (parts.length <= 1 && !/^[a-zA-Z]:$/.test(parts[0])) {
        console.log("已经在根目录 (/) 或无法解析，无法向上。");
        return;
    }
     // Special case for windows root C:\ -> C:
     if (parts.length === 1 && /^[a-zA-Z]:$/.test(parts[0])) {
         console.log("已经在驱动器根目录，无法向上。");
         return;
     }

    parts.pop();
    let parentPath = parts.join(currentPath.includes('\\') ? '\\' : '/'); // Try to preserve separator

    // Handle root cases
    if (parentPath === '') {
        parentPath = '/';
    }
    // Handle Windows drive letter case C: -> C:\
    else if (/^[a-zA-Z]:$/.test(parentPath)) {
        parentPath += '\\';
    }
     // Ensure trailing separator for directories unless it's the root drive letter
    // This might not be strictly necessary depending on how list_directory handles it
    // if (parentPath.length > 0 && !parentPath.endsWith('\\') && !parentPath.endsWith('/') && !/^[a-zA-Z]:$/.test(parentPath)) {
    //    parentPath += (currentPath.includes('\\') ? '\\' : '/');
    // }


    console.log("向上导航到:", parentPath);
    listDirectory(parentPath);
}

function openItem(item) {
    if (item.is_dir) {
        listDirectory(item.path);
    } else {
        console.log(`尝试打开文件: ${item.path}`);
        alert(`打开文件: ${item.name}`);
        // TODO: Implement file opening logic (e.g., using tauri-plugin-shell)
    }
}

// --- Event Listeners ---
document.addEventListener('DOMContentLoaded', () => {
    // No need for delay or __TAURI__ check now, assuming correct import works
    initializePath();
    if (goUpButton) {
        goUpButton.addEventListener('click', goUp);
    }
}); 