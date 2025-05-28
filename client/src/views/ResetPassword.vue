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
          Set new password
        </h2>
        <p class="mt-2 text-center text-sm text-gray-400">
          Please enter your new password below.
        </p>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="handleSubmit">
        <div class="rounded-md shadow-sm -space-y-px">
          <div>
            <label for="password" class="sr-only">New Password</label>
            <input
              id="password"
              v-model="form.password"
              name="password"
              type="password"
              required
              minlength="12"
              maxlength="36"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-700 bg-gray-800 text-white placeholder-gray-400 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
              placeholder="New Password"
              @input="validatePassword"
            />
          </div>
          <div>
            <label for="confirmPassword" class="sr-only">Confirm Password</label>
            <input
              id="confirmPassword"
              v-model="form.confirmPassword"
              name="confirmPassword"
              type="password"
              required
              minlength="12"
              maxlength="36"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-700 bg-gray-800 text-white placeholder-gray-400 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
              placeholder="Confirm Password"
              @input="validatePassword"
            />
          </div>
        </div>

        <!-- Password Requirements Display -->
        <div class="mt-4 space-y-2">
          <p class="text-sm text-gray-400">Password must meet the following requirements:</p>
          <ul class="text-sm space-y-1">
            <li :class="{'text-red-400': !passwordValidation.length, 'text-green-400': passwordValidation.length}">
              • 12-36 characters long
            </li>
            <li :class="{'text-red-400': !passwordValidation.number, 'text-green-400': passwordValidation.number}">
              • At least 1 number
            </li>
            <li :class="{'text-red-400': !passwordValidation.special, 'text-green-400': passwordValidation.special}">
              • At least 1 special character (e.g., !, @, #, $, %, ^, &, *, etc.)
            </li>
            <li :class="{'text-red-400': !passwordValidation.uppercase, 'text-green-400': passwordValidation.uppercase}">
              • At least 2 uppercase letters
            </li>
            <li :class="{'text-red-400': !passwordValidation.lowercase, 'text-green-400': passwordValidation.lowercase}">
              • At least 2 lowercase letters
            </li>
            <li :class="{'text-red-400': !passwordValidation.match, 'text-green-400': passwordValidation.match}">
              • Passwords match
            </li>
          </ul>
        </div>

        <div>
          <button
            type="submit"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            :disabled="!isPasswordValid || loading"
          >
            <span v-if="loading">Resetting password...</span>
            <span v-else>Reset password</span>
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
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

interface FormData {
  password: string
  confirmPassword: string
}

interface PasswordValidation {
  length: boolean
  number: boolean
  special: boolean
  uppercase: boolean
  lowercase: boolean
  match: boolean
}

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const form = ref<FormData>({
  password: '',
  confirmPassword: ''
})

const passwordValidation = ref<PasswordValidation>({
  length: false,
  number: false,
  special: false,
  uppercase: false,
  lowercase: false,
  match: false
})

const loading = ref(false)
const error = ref('')
const success = ref('')

// Computed
const isPasswordValid = computed<boolean>(() => {
  return Object.values(passwordValidation.value).every(valid => valid)
})

// Methods
const validatePassword = (): void => {
  const password = form.value.password
  const confirmPassword = form.value.confirmPassword
  
  // Length validation (12-36 characters)
  passwordValidation.value.length = password.length >= 12 && password.length <= 36
  
  // Number validation (at least 1 number)
  passwordValidation.value.number = /\d/.test(password)
  
  // Special character validation
  passwordValidation.value.special = /[!@#$%^&*()_+=?<>:;{}[\]-]/.test(password)
  
  // Uppercase validation (at least 2 uppercase letters)
  const uppercaseCount = (password.match(/[A-Z]/g) || []).length
  passwordValidation.value.uppercase = uppercaseCount >= 2
  
  // Lowercase validation (at least 2 lowercase letters)
  const lowercaseCount = (password.match(/[a-z]/g) || []).length
  passwordValidation.value.lowercase = lowercaseCount >= 2

  // Password match validation
  passwordValidation.value.match = password === confirmPassword
}

const handleSubmit = async (): Promise<void> => {
  try {
    loading.value = true
    error.value = ''
    success.value = ''

    if (!isPasswordValid.value) {
      throw new Error('Password does not meet requirements')
    }

    const token = route.params.token as string
    await authStore.resetPassword(token, form.value.password)
    success.value = 'Password has been reset successfully'
    
    // Redirect to login after 2 seconds
    setTimeout(() => {
      router.push('/login')
    }, 2000)
  } catch (err: any) {
    error.value = err?.message || 'Failed to reset password'
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  try {
    const token = route.params.token as string
    await authStore.verifyPasswordResetToken(token)
  } catch (err: any) {
    error.value = 'Invalid or expired reset token'
    setTimeout(() => {
      router.push('/forgot-password')
    }, 2000)
  }
})
</script> 