<script setup lang="ts">
/**
 * Sidebar (Server List) - Discord style
 * - Discord logo at top
 * - Divider
 * - Server icons (colored circles with letters)
 * - Add server button
 * - Download button at bottom
 * - Active indicator and Discord-like hover effects
 */
import { ref } from 'vue'

interface Server {
  id: number
  name: string
  color: string
  letter: string
}

const servers: Server[] = [
  { id: 1, name: 'Server One', color: 'bg-indigo-600', letter: 'S' },
  { id: 2, name: 'Server Two', color: 'bg-green-500', letter: 'A' },
  { id: 3, name: 'Server Three', color: 'bg-red-500', letter: 'B' },
  { id: 4, name: 'Server Four', color: 'bg-yellow-400', letter: 'C' },
]

const activeServerId = ref(servers[0].id)

function setActive(id: number) {
  activeServerId.value = id
}
</script>

<template>
  <aside class="w-16 bg-gray-900 flex flex-col justify-between items-center h-screen">
    <!-- Top section: Discord logo and divider -->
    <div class="flex flex-col items-center space-y-3 w-full pt-3">
      <!-- Custom logo.svg -->
      <div class="w-12 h-12 flex items-center justify-center">
        <img src="/src/assets/logo.png" alt="App Logo" class="w-10 h-10 object-contain" />
      </div>
     
      <!-- Divider -->
      <div class="w-8 h-0.5 bg-gray-700 rounded my-1"></div>
      <!-- Server icons -->
      <div class="flex flex-col items-center space-y-3 w-full mt-2">
        <div
          v-for="server in servers"
          :key="server.id"
          :title="server.name"
          class="relative flex items-center justify-center w-full"
        >
          <!-- Active indicator -->
          <div
            v-if="activeServerId === server.id"
            class="absolute -left-2 h-10 w-1.5 bg-white rounded-r-full shadow-lg z-10 transition-all duration-200"
          ></div>
          <button
            :class="[
              server.color,
              'w-12 h-12 rounded-full flex items-center justify-center text-lg font-bold cursor-pointer transition-all duration-150 outline-none',
              'hover:rounded-2xl hover:bg-white hover:text-gray-900 hover:shadow-xl',
              activeServerId === server.id ? 'rounded-2xl ring-4 ring-white scale-110 shadow-xl' : '',
              'focus:ring-2 focus:ring-blue-400',
              'select-none'
            ]"
            @click="setActive(server.id)"
            tabindex="0"
          >
            {{ server.letter }}
          </button>
        </div>
      </div>
      <!-- Add Server Button -->
      <button
        class="w-12 h-12 bg-gray-700 rounded-full flex items-center justify-center text-2xl font-bold mt-2 transition-all duration-150 hover:rounded-2xl hover:bg-green-500 hover:text-white hover:scale-110 focus:outline-none"
        title="Add a Server"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
      <!-- Download button -->
      <div class="flex flex-col items-center w-full">
        <button
          class="w-12 h-12 bg-gray-700 rounded-full flex items-center justify-center text-xl font-bold mb-2 transition-all duration-150 hover:rounded-2xl hover:bg-blue-500 hover:text-white hover:scale-110 focus:outline-none"
          title="Download App"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v12m0 0l-4-4m4 4l4-4m-4 4V4" />
          </svg>
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
svg {
  width: 2.5rem !important;
  height: 2.5rem !important;
}
</style> 