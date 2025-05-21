<template>
  <div class="flex flex-col items-center gap-2">
    <!-- Home Button -->
    <div class="w-12 h-12 rounded-full bg-[#5865f2] flex items-center justify-center cursor-pointer transition-all duration-200 hover:rounded-2xl">
      <HomeIcon class="w-6 h-6 text-white" />
    </div>
    
    <!-- Separator -->
    <div class="w-8 h-0.5 bg-[#36393f] my-1"></div>
    
    <!-- Server Icons -->
    <div class="flex flex-col gap-2">
      <div 
        v-for="server in servers" 
        :key="server.id" 
        class="w-12 h-12 rounded-full bg-[#36393f] flex items-center justify-center cursor-pointer transition-all duration-200 hover:rounded-2xl hover:bg-[#5865f2] overflow-hidden"
      >
        <img 
          v-if="server.icon" 
          :src="server.icon" 
          :alt="server.name"
          class="w-full h-full object-cover"
        >
        <span v-else class="text-[#dcddde] text-xl font-medium">
          {{ server.name.charAt(0) }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { HomeIcon } from '@heroicons/vue/24/solid'

const servers = ref([
  { id: 1, name: 'Gaming', icon: null },
  { id: 2, name: 'Music', icon: null },
  { id: 3, name: 'Coding', icon: null }
])
</script>

<style scoped>
.server-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.server-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background-color: #36393f;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: border-radius 0.2s;
  overflow: hidden;
}

.server-icon:hover {
  border-radius: 16px;
  background-color: #5865f2;
}

.server-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.server-initial {
  color: #dcddde;
  font-size: 20px;
  font-weight: 500;
}

.separator {
  width: 32px;
  height: 2px;
  background-color: #36393f;
  margin: 4px 0;
}

.home-icon {
  background-color: #5865f2;
  color: white;
}
</style> 