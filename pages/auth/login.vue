<template>
  <div class="flex items-center justify-center min-h-screen bg-nightgray">
    <form @submit.prevent="handleLogin" class="bg-gray-800 p-8 rounded shadow-md w-96">
      <h2 class="text-2xl font-bold mb-6 text-white">Login</h2>
      <div class="mb-4">
        <label class="block text-gray-300 mb-1">Email</label>
        <input v-model="email" type="email" class="w-full p-2 rounded bg-gray-700 text-white" required />
      </div>
      <div class="mb-6">
        <label class="block text-gray-300 mb-1">Password</label>
        <input v-model="password" type="password" class="w-full p-2 rounded bg-gray-700 text-white" required />
      </div>
      <div v-if="error" class="mb-4 text-red-500">{{ error }}</div>
      <button type="submit" class="w-full py-2 bg-green-600 text-white rounded hover:bg-green-700">Login</button>
      <div class="mt-4 text-gray-400 text-sm">
        Don't have an account?
        <NuxtLink to="/auth/register" class="text-green-400 hover:underline">Register</NuxtLink>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useUserStore } from '~/store/user-store'

const email = ref('')
const password = ref('')
const error = ref('')
const router = useRouter()
const userStore = useUserStore()

async function handleLogin() {
  error.value = ''
  try {
    const res = await $fetch('/api/auth/login', {
      method: 'POST',
      body: { email: email.value, password: password.value },
    })
    if (res && res.token && res.user) {
      userStore.setUser(res.user, res.token)
      await userStore.fetchProfile()
      router.push('/')
    } else {
      error.value = res.error || 'Login failed.'
    }
  } catch (err: any) {
    error.value = err.data?.error || 'Login failed.'
  }
}
</script> 