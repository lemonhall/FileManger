<template>
  <div v-if="loading" class="loading-indicator">加载中...</div>
  <div v-if="error" class="error-message">错误: {{ error }}</div>
  <div class="file-list" v-if="!loading && !error">
    <table>
      <thead>
        <tr>
          <th></th> <!-- Checkbox -->
          <th>名称</th>
          <th>类型</th>
          <th>大小</th>
          <th>属性</th> 
        </tr>
      </thead>
      <tbody>
        <tr v-if="items.length === 0">
            <td colspan="5">文件夹为空</td>
        </tr>
        <FileListItem 
          v-else 
          v-for="item in items" 
          :key="item.path" 
          :item="item"
          @item-dblclick="handleItemDblClick"
          @update:selected="handleItemSelectionUpdate"
        />
      </tbody>
    </table>
  </div>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue';
import FileListItem from './FileListItem.vue'; // Import list item component

defineProps({
  items: {
    type: Array,
    required: true
  },
  loading: Boolean,
  error: String | null
});

const emit = defineEmits(['item-dblclick', 'update:item-selected']);

function handleItemDblClick(item) {
  emit('item-dblclick', item);
}

function handleItemSelectionUpdate(itemId, newSelectedValue) {
  emit('update:item-selected', itemId, newSelectedValue);
}
</script>

<style scoped>
.loading-indicator, .error-message {
    margin-top: 1rem;
    padding: 0.5rem;
    font-weight: bold;
}
.error-message {
    color: red;
    border: 1px solid red;
    background-color: #ffeeee;
}
.file-list table {
  width: 100%;
  border-collapse: collapse;
}
.file-list th, .file-list td { /* td styles might be overridden by FileListItem, but good defaults */
  border: 1px solid #ddd;
  padding: 8px;
  text-align: left;
  white-space: nowrap;
  vertical-align: middle;
}
.file-list th {
  background-color: #f2f2f2;
}
/* Hover effect can be applied here or in FileListItem */
/* .file-list tbody tr:hover {
  background-color: #f5f5f5;
  cursor: pointer;
} */

/* Column widths defined here */
.file-list th:first-child, .file-list td:first-child { width: 30px; text-align: center;}
.file-list th:nth-child(2), .file-list td:nth-child(2) { width: auto; white-space: normal; } /* Let name column take remaining space */
.file-list th:nth-child(3), .file-list td:nth-child(3) { width: 120px; }
.file-list th:nth-child(4), .file-list td:nth-child(4) { width: 100px; }
.file-list th:nth-child(5), .file-list td:nth-child(5) { width: 50px; text-align: center; }

</style> 