<script setup lang="ts">
import { useToast } from '../composables/useToast'
const { toasts } = useToast()
</script>

<template>
  <div class="fixed top-4 right-4 z-50 flex flex-col gap-2">
    <transition-group name="toast-fade" tag="div">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="[
          'px-4 py-2 rounded shadow-lg text-white min-w-[200px] max-w-xs',
          toast.type === 'success' ? 'bg-green-600' : toast.type === 'error' ? 'bg-red-600' : 'bg-gray-800'
        ]"
      >
        {{ toast.message }}
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: opacity 0.3s, transform 0.3s;
}
.toast-fade-enter-from,
.toast-fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style> 