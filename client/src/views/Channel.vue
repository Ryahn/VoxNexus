<template>
  <div class="channel">
    <h1>Channel View</h1>
    <p>Server ID: {{ serverId }}</p>
    <p>Channel ID: {{ channelId }}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useRoute } from 'vue-router'

export default defineComponent({
  name: 'ChannelView',
  setup() {
    const route = useRoute()
    const serverId = route.params.serverId
    const channelId = route.params.channelId

    return {
      serverId,
      channelId
    }
  }
})
</script>

<style scoped>
.channel {
  padding: 2rem;
}
</style> 