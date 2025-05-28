<template>
  <div class="flex justify-between items-center p-2 text-[#dcddde]">
    <div class="flex items-center gap-2">
      <div class="w-8 h-8 rounded-full bg-[#5865f2] flex items-center justify-center">
        <span class="text-white font-medium">{{ username.charAt(0) }}</span>
      </div>
      <div class="flex flex-col cursor-pointer" @click="openModal">
        <span class="font-medium text-sm">{{ username }}</span>
        <span
          class="text-xs text-[#b9bbbe]"
          :class="`presence-${presenceLabel}`"
        >
          {{ presenceLabel }}
        </span>
      </div>
    </div>
    
    <div class="flex gap-1">
      <button 
        @click="toggleMic" 
        :class="[
          'p-1.5 rounded transition-colors',
          isMicMuted
            ? 'bg-red-500 hover:bg-red-600 text-white'
            : 'text-[#b9bbbe] hover:bg-[#36393f] hover:text-[#dcddde]'
        ]"
      >
        <MicrophoneIcon v-if="!isMicMuted" class="w-5 h-5" />
        <font-awesome-icon v-else icon="microphone-slash" class="w-5 h-5" />
      </button>
      
      <button 
        @click="toggleSpeaker" 
        :class="[
          'p-1.5 rounded transition-colors',
          isSpeakerMuted
            ? 'bg-red-500 hover:bg-red-600 text-white'
            : 'text-[#b9bbbe] hover:bg-[#36393f] hover:text-[#dcddde]'
        ]"
      >
        <SpeakerWaveIcon v-if="!isSpeakerMuted" class="w-5 h-5" />
        <SpeakerXMarkIcon v-else class="w-5 h-5" />
      </button>
      
      <button 
        @click="openSettings"
        class="p-1.5 rounded text-[#b9bbbe] hover:bg-[#36393f] hover:text-[#5865f2] transition-colors"
      >
        <Cog6ToothIcon class="w-5 h-5" />
      </button>
    </div>
    <UserProfileModal
      :open="modalOpen"
      :username="username"
      :user-id="userId"
      :presence="presence"
      @close="modalOpen = false"
      @update:presence="setPresence"
    />
    <Settings
      v-if="settingsOpen"
      @close="settingsOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { 
  MicrophoneIcon, 
  SpeakerWaveIcon,
  SpeakerXMarkIcon,
  Cog6ToothIcon 
} from '@heroicons/vue/24/solid'
import UserProfileModal from './UserProfileModal.vue'
import Settings from '../Settings.vue'

type PresenceType = 'online' | 'idle' | 'dnd' | 'invisible'

// State
const username = ref<string>('User')
const userId = ref<string>('1234567890')
const isMicMuted = ref<boolean>(false)
const isSpeakerMuted = ref<boolean>(false)
const modalOpen = ref<boolean>(false)
const settingsOpen = ref<boolean>(false)
const presence = ref<PresenceType>('online')

// Computed
const presenceLabel = computed<string>(() => {
  switch (presence.value) {
    case 'online': return 'Online'
    case 'idle': return 'Idle'
    case 'dnd': return 'DND'
    case 'invisible': return 'Invisible'
    default: return 'Online'
  }
})

// Methods
const toggleMic = (): void => {
  isMicMuted.value = !isMicMuted.value
}

const toggleSpeaker = (): void => {
  isSpeakerMuted.value = !isSpeakerMuted.value
}

const openModal = (): void => {
  modalOpen.value = true
}

const openSettings = (): void => {
  settingsOpen.value = true
}

const setPresence = (val: PresenceType): void => {
  presence.value = val
}
</script>

<style scoped>
.user-profile {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  color: #dcddde;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: #5865f2;
  display: flex;
  align-items: center;
  justify-content: center;
}

.avatar-initial {
  color: white;
  font-weight: 500;
}

.user-details {
  display: flex;
  flex-direction: column;
}

.username {
  font-weight: 500;
  font-size: 14px;
}

.status {
  font-size: 12px;
  color: #b9bbbe;
}

.controls {
  display: flex;
  gap: 4px;
}

.control-btn {
  background: none;
  border: none;
  color: #b9bbbe;
  padding: 6px;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.2s, color 0.2s;
}

.control-btn:hover {
  background-color: #36393f;
  color: #dcddde;
}

.control-btn.muted {
  color: #ed4245;
}

.control-btn.muted:hover {
  background-color: #ed4245;
  color: white;
}

.settings:hover {
  color: #5865f2;
}

.presence-Online {
    background-color: #16a34a;
    color: white;
    padding-top: 2px;
    padding-bottom: 2px;
    padding-left: 4px;
    padding-right: 4px;
    border-radius: 3px;
    border: 1px solid #16a34a;
}

.presence-Idle {
    background-color: #f59e0b;
    color: white;
    padding-top: 2px;
    padding-bottom: 2px;
    padding-left: 4px;
    padding-right: 4px;
    border-radius: 3px;
    border: 1px solid #f59e0b;
}

.presence-DND {
    background-color: #ed4245;
    color: white;
    padding-top: 2px;
    padding-bottom: 2px;
    padding-left: 4px;
    padding-right: 4px;
    border-radius: 3px;
    border: 1px solid #ed4245;
}

.presence-Invisible {
    background-color: #6b7280;
    color: white;
    padding-top: 2px;
    padding-bottom: 2px;
    padding-left: 4px;
    padding-right: 4px;
    border-radius: 3px;
    border: 1px solid #6b7280;
}
</style> 