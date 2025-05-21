<template>
  <TransitionRoot as="template" :show="open">
    <Dialog as="div" class="fixed inset-0 z-50 flex items-center justify-center" @close="close">
      <div class="fixed inset-0 bg-black bg-opacity-60" aria-hidden="true" />
      <div class="relative bg-[#23272a] rounded-xl shadow-xl w-full max-w-xs mx-auto p-6">
        <!-- Avatar and Username -->
        <div class="flex flex-col items-center mb-4">
          <div class="w-20 h-20 rounded-full bg-[#5865f2] flex items-center justify-center mb-2">
            <span class="text-white text-3xl font-bold">{{ username.charAt(0) }}</span>
          </div>
          <div class="text-lg font-bold text-white">{{ username }}</div>
          <div class="text-sm text-[#b9bbbe] mb-2">{{ userId }}</div>
        </div>
        <!-- Presence Selector -->
        <div class="mb-4">
          <label class="block text-xs text-[#b9bbbe] mb-1">Set Status</label>
          <div class="flex gap-2">
            <button v-for="option in presenceOptions" :key="option.value"
              @click="setPresence(option.value)"
              :class="[
                'flex items-center gap-1 px-2 py-1 rounded transition',
                presence === option.value ? option.activeClass : 'hover:bg-[#2c2f33]'
              ]"
            >
              <span :class="['w-3 h-3 rounded-full', option.dotClass]"></span>
              <span class="text-xs text-white">{{ option.label }}</span>
            </button>
          </div>
        </div>
        <!-- Copy User ID -->
        <button @click="copyUserId" class="w-full flex items-center gap-2 px-3 py-2 rounded bg-[#2c2f33] hover:bg-[#5865f2] text-[#b9bbbe] hover:text-white transition mb-2">
          <span class="font-bold text-xs">ID</span>
          <span class="flex-1 text-left truncate">Copy User ID</span>
        </button>
        <!-- Close -->
        <button @click="close" class="absolute top-2 right-2 text-[#b9bbbe] hover:text-white">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
        </button>
      </div>
    </Dialog>
  </TransitionRoot>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue'
import { Dialog, TransitionRoot } from '@headlessui/vue'

const props = defineProps({
  open: Boolean,
  username: String,
  userId: String,
  presence: String
})
const emit = defineEmits(['close', 'update:presence'])

const presenceOptions = [
  { label: 'Online', value: 'online', dotClass: 'bg-green-500', activeClass: 'bg-green-600' },
  { label: 'Idle', value: 'idle', dotClass: 'bg-yellow-400', activeClass: 'bg-yellow-500' },
  { label: 'Do Not Disturb', value: 'dnd', dotClass: 'bg-red-500', activeClass: 'bg-red-600' },
  { label: 'Invisible', value: 'invisible', dotClass: 'bg-gray-400', activeClass: 'bg-gray-500' },
]

function close() {
  emit('close')
}

function setPresence(val) {
  emit('update:presence', val)
}

function copyUserId() {
  navigator.clipboard.writeText(props.userId)
}
</script> 