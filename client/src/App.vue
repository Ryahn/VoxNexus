<template>
  <div class="app">
    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
    <notifications position="top right" />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { websocketService } from '@/services/websocket/websocketService'
import { Notifications } from '@kyvg/vue3-notification'

export default defineComponent({
  name: 'App',
  components: {
    Notifications
  },
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
