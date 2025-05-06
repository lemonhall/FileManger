<script setup>
import FileManager from './components/FileManager.vue';
import { Notivue, Notification } from 'notivue';
</script>

<template>
  <FileManager />
  <Notivue v-slot="item">
    <Notification :item="item" />
  </Notivue>
</template>

<style scoped>
/* Add styles for App.vue if needed */
/* Ensure Notivue is positioned correctly if needed, e.g., if it conflicts with other global styles */
</style> 