<template>
  <div class="flex flex-col h-full bg-[#2f3136] w-60">
    <!-- Server Header -->
    <div class="h-12 px-4 flex items-center border-b border-[#232428]">
      <h2 class="text-white font-semibold truncate">{{ currentServer?.name || 'Select a Server' }}</h2>
    </div>

    <!-- Channel List -->
    <div class="flex-1 p-4 text-[#b9bbbe]">
      <div class="font-bold mb-2">Text Channels</div>
      <div 
        v-for="channel in channels" 
        :key="channel.id"
        class="mb-2 px-2 py-1 rounded hover:bg-[#36393f] cursor-pointer"
        :class="{ 'bg-[#36393f]': currentChannel?.id === channel.id }"
        @click="selectChannel(channel)"
      >
        # {{ channel.name }}
      </div>
    </div>

    <!-- User Profile -->
    <div class="p-2 border-t border-[#232428]">
      <slot name="user-profile"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useServerStore } from '@/stores/server'
import type { Channel } from '@/types'

const router = useRouter()
const route = useRoute()
const serverStore = useServerStore()

// Computed
const currentServer = computed(() => serverStore.currentServer)
const channels = computed(() => currentServer.value?.channels || [])
const currentChannel = computed(() => {
  const channelId = route.params.channelId as string
  return channels.value.find(channel => channel.id === channelId)
})

// Methods
const selectChannel = (channel: Channel): void => {
  if (currentServer.value) {
    router.push(`/servers/${currentServer.value.id}/channels/${channel.id}`)
  }
}
</script> 