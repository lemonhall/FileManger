<template>
  <tr @dblclick="onDblClick">
    <td><input type="checkbox" :checked="item.selected" @change="toggleSelection" @click.stop /></td>
    <td>
      <span v-html="currentFileIcon"></span>
      {{ item.name }}
    </td>
    <td>{{ currentFileTypeDescription }}</td>
    <td>{{ formattedSize }}</td>
    <td>{{ item.readonly ? 'R' : '' }}</td>
    <td>{{ formattedLastUploaded }}</td>
  </tr>
</template>

<script setup>
import { computed, defineProps, defineEmits } from 'vue';
import { formatSize as importedFormatSize } from '../utils/formatters'; // Renamed to avoid conflict
import { 
    folderIcon, 
    hiddenFolderIcon, 
    fileIcon as genericFileIcon, // Renamed for clarity
    imageIcon, 
    audioIcon, 
    videoIcon, 
    pdfIcon, 
    codeIcon, 
    archiveIcon, 
    executableIcon 
} from '../utils/icons';
import { fileTypeDescriptions } from '../utils/fileTypes';
import { useUploadTimestamps } from '../composables/useUploadTimestamps'; // Import composable

const props = defineProps({
  item: {
    type: Object,
    required: true
  }
});

const emit = defineEmits(['item-dblclick', 'update:selected']);

// Get timestamp for the current file
const { getTimestampForFile } = useUploadTimestamps();
const lastUploadedTimestamp = computed(() => getTimestampForFile(props.item.path));

const formattedLastUploaded = computed(() => {
  const ts = lastUploadedTimestamp.value;
  if (!ts) return '-';
  try {
    // Format the timestamp (e.g., YYYY-MM-DD HH:mm)
    const date = new Date(ts);
    // Basic formatting, can be improved with a date formatting library if needed
    const year = date.getFullYear();
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const day = date.getDate().toString().padStart(2, '0');
    const hours = date.getHours().toString().padStart(2, '0');
    const minutes = date.getMinutes().toString().padStart(2, '0');
    return `${year}-${month}-${day} ${hours}:${minutes}`;
  } catch (e) {
    console.error("Error formatting timestamp:", ts, e);
    return '日期无效';
  }
});

const onDblClick = () => {
  emit('item-dblclick', props.item);
};

const toggleSelection = (event) => {
  emit('update:selected', props.item.path, event.target.checked);
};

const currentFileIcon = computed(() => {
  if (props.item.is_dir) {
    if (props.item.name.startsWith('.')) {
      return hiddenFolderIcon;
    } else {
      return folderIcon;
    }
  }
  const name = props.item.name || '';
  const lastDotIndex = name.lastIndexOf('.');
  if (lastDotIndex === -1 || lastDotIndex === 0 || lastDotIndex === name.length - 1) {
    return genericFileIcon;
  }
  const extension = name.substring(lastDotIndex + 1).toLowerCase();
  switch (extension) {
    case 'jpg': case 'jpeg': case 'png': case 'gif': case 'bmp': case 'tiff': case 'tif': case 'svg': case 'ico': case 'webp': case 'psd': case 'ai': case 'raw': case 'cr2': case 'nef': case 'orf': case 'arw': return imageIcon;
    case 'mp3': case 'wav': case 'ogg': case 'flac': case 'aac': case 'wma': case 'm4a': case 'ape': case 'mid': case 'midi': return audioIcon;
    case 'mp4': case 'avi': case 'mkv': case 'mov': case 'wmv': case 'flv': case 'webm': case 'mpg': case 'mpeg': case 'm4v': case '3gp': return videoIcon;
    case 'pdf': return pdfIcon;
    case 'js': case 'ts': case 'jsx': case 'tsx': case 'css': case 'scss': case 'less': case 'py': case 'java': case 'c': case 'cpp': case 'h': case 'hpp': case 'cs': case 'go': case 'php': case 'rb': case 'swift': case 'kt': case 'rs': case 'vue': case 'sql': case 'html': case 'htm': case 'xml': case 'json': case 'md': case 'sh': case 'bat': return codeIcon;
    case 'zip': case 'rar': case '7z': case 'gz': case 'tar': case 'bz2': case 'xz': case 'iso': case 'img': return archiveIcon;
    case 'exe': case 'dll': case 'msi': case 'jar': case 'apk': case 'app': return executableIcon;
    case 'txt': case 'log': case 'ini': case 'csv': return genericFileIcon;
    case 'ttf': case 'otf': case 'woff': case 'woff2': case 'eot': return genericFileIcon;
    case 'doc': case 'docx': case 'xls': case 'xlsx': case 'ppt': case 'pptx': case 'odt': case 'ods': case 'odp': case 'rtf': return genericFileIcon;
    default: return genericFileIcon;
  }
});

const currentFileTypeDescription = computed(() => {
  if (props.item.is_dir) {
    return '文件夹';
  }
  const name = props.item.name || '';
  const lastDotIndex = name.lastIndexOf('.');
  if (lastDotIndex === -1 || lastDotIndex === 0 || lastDotIndex === name.length - 1) {
    return '文件';
  }
  const extension = name.substring(lastDotIndex + 1).toLowerCase();
  const descriptionData = fileTypeDescriptions[extension];
  // Defaulting to short description, could be made a prop if needed
  if (descriptionData) {
    return descriptionData.short;
  } else {
    return `${extension.toUpperCase()} 文件`;
  }
});

const formattedSize = computed(() => {
  return importedFormatSize(props.item.size);
});

</script>

<style scoped>
/* Minimal styling, inherits from parent or can be expanded */
td {
  vertical-align: middle; /* Ensure icons and text align nicely */
  white-space: nowrap; /* Prevent wrapping for most columns */
}
td:nth-child(2) { /* Name column */
  white-space: normal; /* Allow name to wrap if very long */
}
/* Ensure the new column has reasonable style if needed */
td:last-child {
  /* text-align: right; */ /* Example if you want to right-align dates */
}
</style> 