<template>
  <div class="flex items-center justify-center min-h-screen bg-nightgray">
    <form @submit.prevent="handleUpdate" class="bg-gray-800 p-8 rounded shadow-md w-96">
      <h2 class="text-2xl font-bold mb-6 text-white">Profile</h2>
      <div class="flex flex-col items-center mb-4">
        <img :src="avatarUrl" class="w-20 h-20 rounded-full mb-2 object-cover" alt="Avatar" />
        <input type="file" accept="image/*" @change="handleAvatarChange" class="mb-2 text-white" />
        <button v-if="avatarFile" type="button" @click="uploadAvatar" class="px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700 mb-2">Upload Avatar</button>
      </div>
      <div class="mb-4">
        <label class="block text-gray-300 mb-1">Username</label>
        <input v-model="username" type="text" class="w-full p-2 rounded bg-gray-700 text-white" />
      </div>
      <div class="mb-4">
        <label class="block text-gray-300 mb-1">Email</label>
        <input v-model="email" type="email" class="w-full p-2 rounded bg-gray-700 text-white" />
      </div>
      <div class="mb-4">
        <label class="block text-gray-300 mb-1">Bio</label>
        <textarea v-model="bio" class="w-full p-2 rounded bg-gray-700 text-white" rows="2" />
      </div>
      <div class="mb-4">
        <label class="block text-gray-300 mb-1">Status</label>
        <input v-model="status" type="text" class="w-full p-2 rounded bg-gray-700 text-white" />
      </div>
      <div class="mb-6">
        <label class="block text-gray-300 mb-1">New Password</label>
        <input v-model="password" type="password" class="w-full p-2 rounded bg-gray-700 text-white" />
      </div>
      <div v-if="success" class="mb-4 text-green-500">{{ success }}</div>
      <div v-if="error" class="mb-4 text-red-500">{{ error }}</div>
      <button type="submit" class="w-full py-2 bg-green-600 text-white rounded hover:bg-green-700 mb-2">Update</button>
      <button type="button" @click="handleLogout" class="w-full py-2 bg-red-600 text-white rounded hover:bg-red-700 mb-2">Logout</button>
      <button type="button" @click="handleDelete" class="w-full py-2 bg-gray-700 text-white rounded hover:bg-red-800">Delete Account</button>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useUserStore } from '~/store/user-store'

const userStore = useUserStore()
const username = ref('')
const email = ref('')
const password = ref('')
const bio = ref('')
const status = ref('')
const error = ref('')
const success = ref('')
const avatarUrl = ref('https://ui-avatars.com/api/?name=User')
const avatarFile = ref<File | null>(null)

onMounted(() => {
  if (userStore.user) {
    username.value = userStore.user.username
    email.value = userStore.user.email
    bio.value = userStore.user.bio || ''
    status.value = userStore.user.status || ''
    avatarUrl.value = userStore.user.avatarUrl || avatarUrl.value
  }
})

async function handleUpdate() {
  error.value = ''
  success.value = ''
  try {
    const res = await $fetch('/api/users/me', {
      method: 'PUT',
      body: {
        username: username.value,
        email: email.value,
        password: password.value || undefined,
        bio: bio.value,
        status: status.value,
      },
      headers: { Authorization: `Bearer ${userStore.token}` },
    })
    if (res && res.user) {
      userStore.setUser(res.user, userStore.token)
      success.value = 'Profile updated successfully.'
      password.value = ''
    } else {
      error.value = res.error || 'Update failed.'
    }
  } catch (err: any) {
    error.value = err.data?.error || 'Update failed.'
  }
}

function handleLogout() {
  userStore.logout()
}

function handleAvatarChange(e: Event) {
  const files = (e.target as HTMLInputElement).files
  if (files && files[0]) {
    avatarFile.value = files[0]
    avatarUrl.value = URL.createObjectURL(files[0])
  }
}

async function uploadAvatar() {
  if (!avatarFile.value) return
  error.value = ''
  success.value = ''
  const formData = new FormData()
  formData.append('avatar', avatarFile.value)
  try {
    const res = await $fetch('/api/users/me/avatar', {
      method: 'POST',
      body: formData,
      headers: { Authorization: `Bearer ${userStore.token}` },
    })
    if (res && res.avatarUrl) {
      avatarUrl.value = res.avatarUrl
      userStore.setUser({ ...userStore.user, avatarUrl: res.avatarUrl }, userStore.token)
      success.value = 'Avatar updated successfully.'
      avatarFile.value = null
    } else {
      error.value = res.error || 'Avatar upload failed.'
    }
  } catch (err: any) {
    error.value = err.data?.error || 'Avatar upload failed.'
  }
}

async function handleDelete() {
  if (!confirm('Are you sure you want to delete your account? This action cannot be undone.')) return
  try {
    await $fetch('/api/users/me', {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${userStore.token}` },
    })
    userStore.logout()
  } catch (err: any) {
    error.value = err.data?.error || 'Delete failed.'
  }
}
</script> 