<template>
  <div class="server">
    <h1>Server View</h1>
    <p>Server ID: {{ serverId }}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useRoute } from 'vue-router'

export default defineComponent({
  name: 'ServerView',
  setup() {
    const route = useRoute()
    const serverId = route.params.serverId

    return {
      serverId
    }
  }
})
</script>

<style scoped>
.server {
  padding: 2rem;
}
</style> 