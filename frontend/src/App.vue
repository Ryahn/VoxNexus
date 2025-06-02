<script setup lang="ts">
import Sidebar from './components/layout/Sidebar.vue'
import ChannelList from './components/layout/ChannelList.vue'
// This is the main layout shell for the Discord clone UI
// Extract each section into its own component as you build out functionality
</script>

<template>
  <router-view />
</template>

<style scoped>
/* Custom scrollbar for chat area */
::-webkit-scrollbar {
  width: 8px;
  background: #23272a;
}
::-webkit-scrollbar-thumb {
  background: #2c2f33;
  border-radius: 4px;
}
</style>
