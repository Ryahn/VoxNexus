<script setup lang="ts">
/**
 * ChannelList - Discord style (refined)
 * - Text/Voice channel sections
 * - # icon for text, mic icon for voice
 * - Add channel button (plus icon) on section hover
 * - Unread indicator (dot)
 * - Improved spacing and styling
 */
import { ref } from 'vue'

interface Channel {
  id: number
  name: string
  type: 'text' | 'voice'
  unread?: boolean
}

const textChannels: Channel[] = [
  { id: 1, name: 'welcome', type: 'text', unread: false },
  { id: 2, name: 'general', type: 'text', unread: true },
  { id: 3, name: 'random', type: 'text', unread: false },
]

const voiceChannels: Channel[] = [
  { id: 4, name: 'General', type: 'voice', unread: false },
  { id: 5, name: 'AFK', type: 'voice', unread: false },
]

const activeChannelId = ref(textChannels[0].id)

function setActive(id: number) {
  activeChannelId.value = id
}
</script>

<template>
  <nav class="w-64 bg-gray-800 border-r border-gray-900 flex flex-col h-full">
    <div class="p-4 font-bold text-lg border-b border-gray-900 text-white"># General</div>
    <div class="flex-1 overflow-y-auto py-2">
      <!-- Text Channels Section -->
      <div class="group px-2 mb-2">
        <div class="flex items-center justify-between px-2 mb-1 text-xs text-gray-400 uppercase tracking-wide font-semibold">
          <span>Text Channels</span>
          <button class="opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-gray-700" title="Add Text Channel">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
        <ul>
          <li
            v-for="channel in textChannels"
            :key="channel.id"
            @click="setActive(channel.id)"
            :class="[
              'flex items-center px-3 py-1.5 cursor-pointer select-none transition-colors rounded-md relative',
              activeChannelId === channel.id
                ? 'bg-gray-700 text-white font-semibold'
                : 'text-gray-300 hover:bg-gray-700 hover:text-white'
            ]"
          >
            <span class="mr-2 text-base text-gray-400">#</span>
            <span class="flex-1 truncate">{{ channel.name }}</span>
            <span v-if="channel.unread && activeChannelId !== channel.id" class="ml-2 w-2 h-2 bg-blue-500 rounded-full"></span>
          </li>
        </ul>
      </div>
      <!-- Voice Channels Section -->
      <div class="group px-2 mt-4">
        <div class="flex items-center justify-between px-2 mb-1 text-xs text-gray-400 uppercase tracking-wide font-semibold">
          <span>Voice Channels</span>
          <button class="opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-gray-700" title="Add Voice Channel">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
        <ul>
          <li
            v-for="channel in voiceChannels"
            :key="channel.id"
            @click="setActive(channel.id)"
            :class="[
              'flex items-center px-3 py-1.5 cursor-pointer select-none transition-colors rounded-md relative',
              activeChannelId === channel.id
                ? 'bg-gray-700 text-white font-semibold'
                : 'text-gray-300 hover:bg-gray-700 hover:text-white'
            ]"
          >
            <svg class="mr-2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v18m7-7H5" />
              <path stroke-linecap="round" stroke-linejoin="round" d="M19 12a7 7 0 1 1-14 0 7 7 0 0 1 14 0z" />
            </svg>
            <span class="flex-1 truncate">{{ channel.name }}</span>
            <span v-if="channel.unread && activeChannelId !== channel.id" class="ml-2 w-2 h-2 bg-blue-500 rounded-full"></span>
          </li>
        </ul>
      </div>
    </div>
    <!-- User Info Bar (bottom of channel list panel) -->
    <div class="w-full flex items-center gap-2 px-3 py-3 bg-gray-900 border-t border-gray-700 cursor-pointer hover:bg-gray-800 transition">
      <div class="w-8 h-8 bg-gray-700 rounded-full"></div>
      <div class="flex flex-col">
        <span class="text-sm font-semibold text-white leading-none">Username</span>
        <span class="text-xs text-gray-400 leading-none">#1234</span>
      </div>
    </div>
  </nav>
</template>

<style scoped>
</style> 