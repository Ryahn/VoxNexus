<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'

const username = ref('')
const email = ref('')
const password = ref('')
const confirmPassword = ref('')
const error = ref('')
const router = useRouter()

const passwordChecks = computed(() => {
  const pass = password.value
  const uname = username.value
  const mail = email.value
  const confirm = confirmPassword.value
  return [
    {
      label: 'At least 12 characters',
      met: pass.length >= 12,
    },
    {
      label: 'At least 2 special characters',
      met: (pass.match(/[^A-Za-z0-9]/g) || []).length >= 2,
    },
    {
      label: 'At least 2 numbers',
      met: (pass.match(/[0-9]/g) || []).length >= 2,
    },
    {
      label: 'At least 2 uppercase letters',
      met: (pass.match(/[A-Z]/g) || []).length >= 2,
    },
    {
      label: 'Cannot contain your username',
      met: uname ? pass.toLowerCase().indexOf(uname.toLowerCase()) === -1 : true,
    },
    {
      label: 'Cannot contain your email',
      met: mail ? pass.toLowerCase().indexOf(mail.toLowerCase()) === -1 : true,
    },
    {
      label: 'Passwords match',
      met: pass.length > 0 && pass === confirm,
    },
  ]
})

function handleRegister() {
  error.value = ''
  if (!username.value || !email.value || !password.value || !confirmPassword.value) {
    error.value = 'Please fill in all fields.'
    return
  }
  if (password.value !== confirmPassword.value) {
    error.value = 'Passwords do not match.'
    return
  }
  // Password requirements
  if (password.value.length < 12) {
    error.value = 'Password must be at least 12 characters long.'
    return
  }
  if ((password.value.match(/[^A-Za-z0-9]/g) || []).length < 2) {
    error.value = 'Password must contain at least 2 special characters.'
    return
  }
  if ((password.value.match(/[0-9]/g) || []).length < 2) {
    error.value = 'Password must contain at least 2 numbers.'
    return
  }
  if ((password.value.match(/[A-Z]/g) || []).length < 2) {
    error.value = 'Password must contain at least 2 uppercase letters.'
    return
  }
  if (password.value.toLowerCase().indexOf(username.value.toLowerCase()) !== -1) {
    error.value = 'Password cannot contain your username.'
    return
  }
  if (password.value.toLowerCase().indexOf(email.value.toLowerCase()) !== -1) {
    error.value = 'Password cannot contain your email.'
    return
  }
  // TODO: Connect to backend
  // router.push('/login')
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-900">
    <div class="w-full max-w-md bg-gray-800 rounded-lg shadow-lg p-8">
      <h2 class="text-2xl font-bold text-white mb-6 text-center">Create an Account</h2>
      <form @submit.prevent="handleRegister" class="space-y-4">
        <div>
          <label class="block text-gray-300 mb-1" for="username">Username</label>
          <input v-model="username" id="username" type="text" class="w-full px-4 py-2 rounded bg-gray-700 text-white focus:outline-none focus:ring-2 focus:ring-blue-500" autocomplete="username" />
        </div>
        <div>
          <label class="block text-gray-300 mb-1" for="email">Email</label>
          <input v-model="email" id="email" type="email" class="w-full px-4 py-2 rounded bg-gray-700 text-white focus:outline-none focus:ring-2 focus:ring-blue-500" autocomplete="email" />
        </div>
        <div>
          <label class="block text-gray-300 mb-1" for="password">Password</label>
          <input v-model="password" id="password" type="password" class="w-full px-4 py-2 rounded bg-gray-700 text-white focus:outline-none focus:ring-2 focus:ring-blue-500" autocomplete="new-password" />
        </div>
        <div>
          <label class="block text-gray-300 mb-1" for="confirmPassword">Confirm Password</label>
          <input v-model="confirmPassword" id="confirmPassword" type="password" class="w-full px-4 py-2 rounded bg-gray-700 text-white focus:outline-none focus:ring-2 focus:ring-blue-500" autocomplete="new-password" />
        </div>
        <!-- Password requirements checklist -->
        <ul class="mb-2 space-y-1">
          <li v-for="check in passwordChecks" :key="check.label" class="flex items-center text-sm" :class="check.met ? 'text-green-400' : 'text-red-400'">
            <span class="mr-2">
              <svg v-if="check.met" class="w-4 h-4 inline" fill="none" stroke="currentColor" stroke-width="3" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" /></svg>
              <svg v-else class="w-4 h-4 inline" fill="none" stroke="currentColor" stroke-width="3" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>
            </span>
            <span>{{ check.label }}</span>
          </li>
        </ul>
        <div v-if="error" class="text-red-400 text-sm">{{ error }}</div>
        <button type="submit" class="w-full py-2 bg-green-600 hover:bg-green-700 text-white font-semibold rounded transition">Register</button>
      </form>
      <div class="flex justify-between mt-4 text-sm">
        <router-link to="/login" class="text-blue-400 hover:underline">Already have an account?</router-link>
      </div>
    </div>
  </div>
</template> 