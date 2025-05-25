<template>
  <div class="app">
    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { websocketService } from '@/services/websocket/websocketService'

export default defineComponent({
  name: 'App',
  setup() {
    // Initialize WebSocket connection when app starts
    const token = localStorage.getItem('token')
    if (token) {
      websocketService.connect(token)
    }

    return {}
  }
})
</script>

<style>
.app {
  @apply h-screen w-screen bg-gray-900 text-white;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
