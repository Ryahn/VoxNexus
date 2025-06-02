<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const email = ref('')
const password = ref('')
const error = ref('')
const router = useRouter()

function handleLogin() {
  error.value = ''
  if (!email.value || !password.value) {
    error.value = 'Please enter both email and password.'
    return
  }
  // TODO: Connect to backend
  // router.push('/')
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-900">
    <div class="w-full max-w-md bg-gray-800 rounded-lg shadow-lg p-8">
      <h2 class="text-2xl font-bold text-white mb-6 text-center">Login to VoxNexus</h2>
      <form @submit.prevent="handleLogin" class="space-y-4">
        <div>
          <label class="block text-gray-300 mb-1" for="email">Email or Username</label>
          <input v-model="email" id="email" type="text" class="w-full px-4 py-2 rounded bg-gray-700 text-white focus:outline-none focus:ring-2 focus:ring-blue-500" autocomplete="username" />
        </div>
        <div>
          <label class="block text-gray-300 mb-1" for="password">Password</label>
          <input v-model="password" id="password" type="password" class="w-full px-4 py-2 rounded bg-gray-700 text-white focus:outline-none focus:ring-2 focus:ring-blue-500" autocomplete="current-password" />
        </div>
        <div v-if="error" class="text-red-400 text-sm">{{ error }}</div>
        <button type="submit" class="w-full py-2 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded transition">Login</button>
      </form>
      <div class="flex justify-between mt-4 text-sm">
        <router-link to="/register" class="text-blue-400 hover:underline">Register</router-link>
        <router-link to="/forgot-password" class="text-blue-400 hover:underline">Forgot Password?</router-link>
      </div>
    </div>
  </div>
</template> 