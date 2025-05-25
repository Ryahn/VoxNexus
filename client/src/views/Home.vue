<template>
  <MainLayout>
    <HomeView />
  </MainLayout>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import MainLayout from '@/components/Layout/MainLayout.vue'
import HomeView from './HomeView.vue'

export default defineComponent({
  name: 'Home',
  components: { MainLayout, HomeView }
})
</script>

<style scoped>
.home {
  padding: 2rem;
}
</style> 