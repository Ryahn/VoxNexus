<script setup lang="ts">
import Sidebar from './components/layout/Sidebar.vue'
import ChannelList from './components/layout/ChannelList.vue'
// This is the main layout shell for the Discord clone UI
// Extract each section into its own component as you build out functionality
</script>

<template>
  <div class="flex h-screen w-screen bg-gray-900 text-white">
    <!-- Sidebar (Servers) -->
    <Sidebar class="flex-shrink-0 h-screen" />
    <!-- Channel List -->
    <ChannelList class="flex-shrink-0 h-screen" />

    <!-- Main content and User Bar side-by-side -->
    <div class="flex flex-1 h-screen min-w-0">
      <!-- Chat Area -->
      <main class="flex-1 flex flex-col h-screen min-w-0">
        <!-- Chat Header -->
        <header class="h-16 bg-gray-800 border-b border-gray-900 flex items-center px-6">
          <!-- TODO: Replace with ChatHeader component -->
          <span class="font-semibold text-xl"># general</span>
        </header>
        <!-- Message List -->
        <section class="flex-1 overflow-y-auto p-6 space-y-4 bg-gray-900">
          <!-- TODO: Replace with MessageList component -->
          <div class="flex items-start space-x-3">
            <div class="w-10 h-10 bg-gray-700 rounded-full"></div>
            <div>
              <div class="font-semibold">User1 <span class="text-xs text-gray-400 ml-2">09:00</span></div>
              <div>Hello! Welcome to the server.</div>
            </div>
          </div>
          <div class="flex items-start space-x-3">
            <div class="w-10 h-10 bg-gray-700 rounded-full"></div>
            <div>
              <div class="font-semibold">User2 <span class="text-xs text-gray-400 ml-2">09:01</span></div>
              <div>Hey there! 👋</div>
            </div>
          </div>
        </section>
        <!-- Message Input -->
        <footer class="h-20 bg-gray-800 border-t border-gray-900 flex items-center px-6">
          <!-- TODO: Replace with MessageInput component -->
          <input
            type="text"
            class="flex-1 bg-gray-700 rounded-full px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Message #general"
          />
        </footer>
      </main>

      <!-- User Bar -->
    </div>
  </div>
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
