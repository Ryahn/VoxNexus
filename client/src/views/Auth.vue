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
          {{ isLogin ? 'Sign in to your account' : 'Create your account' }}
        </h2>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="handleSubmit">
        <div class="rounded-md shadow-sm -space-y-px">
          <div v-if="!isLogin">
            <label for="username" class="sr-only">Username</label>
            <input
              id="username"
              v-model="form.username"
              name="username"
              type="text"
              required
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-700 bg-gray-800 text-white placeholder-gray-400 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
              placeholder="Username"
              pattern="[a-zA-Z0-9\s\-_]+"
              title="Username can only contain letters, numbers, spaces, hyphens, and underscores"
            />
          </div>
          <div>
            <label for="email" class="sr-only">Email address</label>
            <input
              id="email"
              v-model="form.email"
              name="email"
              type="email"
              required
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-700 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
              :class="{ 'rounded-t-md': isLogin }"
              placeholder="Email address"
            />
          </div>
          <div>
            <label for="password" class="sr-only">Password</label>
            <input
              id="password"
              v-model="form.password"
              name="password"
              type="password"
              required
              minlength="12"
              maxlength="36"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-700 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
              :class="{ 'rounded-b-md': isLogin }"
              placeholder="Password"
              @input="validatePassword"
            />
          </div>
          <div v-if="!isLogin">
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

        <!-- Password Requirements Display (only shown during registration) -->
        <div v-if="!isLogin" class="mt-4 space-y-2">
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
            :disabled="(!isLogin && !isPasswordValid) || loading"
          >
            <span v-if="loading">{{ isLogin ? 'Signing in...' : 'Creating account...' }}</span>
            <span v-else>{{ isLogin ? 'Sign in' : 'Register' }}</span>
          </button>
        </div>

        <div v-if="error" class="text-red-500 text-sm text-center">
          {{ error }}
        </div>

        <div class="text-center">
          <button
            type="button"
            class="text-indigo-400 hover:text-indigo-300"
            @click="toggleMode"
          >
            {{ isLogin ? 'Need an account? Register' : 'Already have an account? Sign in' }}
          </button>
          <div v-if="isLogin" class="mt-2">
            <router-link
              to="/forgot-password"
              class="text-indigo-400 hover:text-indigo-300"
            >
              Forgot your password?
            </router-link>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

interface FormData {
  username: string
  email: string
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

// Initialize composables
const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

// State
const isLogin = ref<boolean>(true)
const loading = ref<boolean>(false)
const error = ref<string>('')

const form = ref<FormData>({
  username: '',
  email: '',
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

// Computed
const isPasswordValid = computed<boolean>(() => {
  return Object.values(passwordValidation.value).every(valid => valid)
})

// Methods
const toggleMode = (): void => {
  isLogin.value = !isLogin.value
  form.value = {
    username: '',
    email: '',
    password: '',
    confirmPassword: ''
  }
  passwordValidation.value = {
    length: false,
    number: false,
    special: false,
    uppercase: false,
    lowercase: false,
    match: false
  }
  error.value = ''
}

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
  passwordValidation.value.match = !isLogin.value ? password === confirmPassword : true
}

const handleSubmit = async (): Promise<void> => {
  try {
    loading.value = true
    error.value = ''

    console.log('[AUTH] Starting form submission', { isLogin: isLogin.value, form: form.value })

    if (!isLogin.value && !isPasswordValid.value) {
      console.log('[AUTH] Password validation failed')
      throw new Error('Password does not meet requirements')
    }

    if (isLogin.value) {
      console.log('[AUTH] Attempting login')
      await authStore.login(form.value.email, form.value.password)
    } else {
      console.log('[AUTH] Attempting registration')
      await authStore.register(
        form.value.username,
        form.value.email,
        form.value.password,
        form.value.confirmPassword
      )
      console.log('[AUTH] Registration successful')
    }

    const redirectPath = route.query.redirect as string || '/'
    console.log('[AUTH] Redirecting to:', redirectPath)
    router.push(redirectPath)
  } catch (err: any) {
    console.error('[AUTH] Error occurred:', err)
    error.value = err?.message || (isLogin.value ? 'Login failed' : 'Registration failed')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
/* Add any component-specific styles here */
</style> 