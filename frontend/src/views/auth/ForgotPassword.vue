<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const email = ref('')
const error = ref('')
const router = useRouter()

function handleForgotPassword() {
  error.value = ''
  if (!email.value) {
    error.value = 'Please enter your email.'
    return
  }
  // TODO: Connect to backend
  // router.push('/login')
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-900">
    <div class="w-full max-w-md bg-gray-800 rounded-lg shadow-lg p-8">
      <h2 class="text-2xl font-bold text-white mb-6 text-center">Forgot Password</h2>
      <form @submit.prevent="handleForgotPassword" class="space-y-4">
        <div>
          <label class="block text-gray-300 mb-1" for="email">Email</label>
          <input v-model="email" id="email" type="email" class="w-full px-4 py-2 rounded bg-gray-700 text-white focus:outline-none focus:ring-2 focus:ring-blue-500" autocomplete="email" />
        </div>
        <div v-if="error" class="text-red-400 text-sm">{{ error }}</div>
        <button type="submit" class="w-full py-2 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded transition">Send Reset Link</button>
      </form>
      <div class="flex justify-between mt-4 text-sm">
        <router-link to="/login" class="text-blue-400 hover:underline">Back to Login</router-link>
      </div>
    </div>
  </div>
</template> 