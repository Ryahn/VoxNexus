<template>
  <div class="min-h-screen bg-gray-900 flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8">
      <div>
        <img
          class="mx-auto h-24 w-auto"
          src="/assets/logo.png"
          alt="VoxNexus Logo"
        />
        <h2 class="mt-6 text-center text-3xl font-extrabold text-white">
          Reset your password
        </h2>
        <p class="mt-2 text-center text-sm text-gray-400">
          Enter your email address and we'll send you a link to reset your password.
        </p>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="handleSubmit">
        <div>
          <label for="email" class="sr-only">Email address</label>
          <input
            id="email"
            v-model="email"
            name="email"
            type="email"
            required
            class="appearance-none rounded-md relative block w-full px-3 py-2 border border-gray-700 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
            placeholder="Email address"
          />
        </div>

        <div>
          <button
            type="submit"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            :disabled="loading"
          >
            <span v-if="loading">Sending reset link...</span>
            <span v-else>Send reset link</span>
          </button>
        </div>

        <div v-if="error" class="text-red-500 text-sm text-center">
          {{ error }}
        </div>

        <div v-if="success" class="text-green-500 text-sm text-center">
          {{ success }}
        </div>

        <div class="text-center">
          <router-link
            to="/login"
            class="text-indigo-400 hover:text-indigo-300"
          >
            Back to login
          </router-link>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()
const email = ref('')
const loading = ref(false)
const error = ref('')
const success = ref('')

const handleSubmit = async (): Promise<void> => {
  try {
    loading.value = true
    error.value = ''
    success.value = ''

    await authStore.requestPasswordReset(email.value)
    success.value = 'Password reset link has been sent to your email'
    email.value = ''
  } catch (err: any) {
    error.value = err?.message || 'Failed to send reset link'
  } finally {
    loading.value = false
  }
}
</script> 