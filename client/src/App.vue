<template>
  <div id="app">
    <component :is="isAuthRoute ? 'div' : 'MainLayout'">
      <router-view></router-view>
    </component>
  </div>
</template>

<script setup>
import { useRoute } from 'vue-router'
import { computed } from 'vue'
// eslint-disable-next-line no-unused-vars
import MainLayout from './components/Layout/MainLayout.vue'

const route = useRoute()
const isAuthRoute = computed(() => route.path === '/auth')
</script>

<style>
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  font-family: 'Whitney', 'Helvetica Neue', Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  @apply bg-[#36393f] text-[#dcddde];
}
</style>
