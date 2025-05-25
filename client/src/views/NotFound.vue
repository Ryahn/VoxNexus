<template>
  <div class="not-found">
    <h1>404</h1>
    <p>Page not found</p>
    <router-link to="/">Go Home</router-link>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'NotFoundView'
})
</script>

<style scoped>
.not-found {
  text-align: center;
  padding: 4rem;
}

h1 {
  font-size: 4rem;
  margin-bottom: 1rem;
}

a {
  color: #5865f2;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}
</style> 