<template>
  <div class="flex flex-col h-full bg-[#2f3136] w-60">
    <!-- Channel list placeholder -->
    <div class="flex-1 p-4 text-[#b9bbbe]">
      <div class="font-bold mb-2">New Room</div>
      <div class="mb-4"># general</div>
      <!-- Add more channels here -->
    </div>
    <div class="p-2 border-t border-[#232428]">
      <slot name="user-profile"></slot>
    </div>
  </div>
</template>

<script setup>
// Channel list logic will go here
</script> 