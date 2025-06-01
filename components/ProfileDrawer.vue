<template>
  <UDrawer v-model:open="drawerOpen" direction="right" :overlay="true" :inset="false" @close="$emit('close')">
    <template #body>
      <aside class="w-full max-w-2xl h-full bg-nightgray shadow-xl flex flex-col">
        <div class="flex h-full">
          <!-- Sidebar Tabs -->
          <nav class="w-64 bg-gray-900 text-white flex flex-col border-r border-gray-800">
            <div class="p-6 text-2xl font-bold border-b border-gray-800">Settings</div>
            <div class="flex-1 flex flex-col py-4">
              <button
                v-for="tab in tabs"
                :key="tab.key"
                @click="selectedTab = tab.key"
                :class="[
                  'text-left px-6 py-3 text-lg font-medium transition',
                  selectedTab === tab.key ? 'bg-gray-800 text-green-400' : 'hover:bg-gray-800 text-gray-300'
                ]"
              >
                {{ tab.label }}
              </button>
            </div>
          </nav>
          <!-- Main Content -->
          <main class="flex-1 flex flex-col p-8 overflow-auto">
            <button class="absolute top-4 right-4 text-gray-400 hover:text-white text-2xl z-10" @click="$emit('close')">&times;</button>
            <div v-if="selectedTab === 'profile'" class="w-full max-w-2xl">
              <form @submit.prevent="handleUpdate" class="bg-gray-800 p-8 rounded shadow-md w-full">
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
            <div v-else-if="selectedTab === 'privacy'" class="w-full max-w-2xl bg-gray-800 p-8 rounded shadow-md">
              <h2 class="text-2xl font-bold mb-6 text-white">Privacy Settings</h2>
              <div class="text-gray-300">Privacy settings content goes here.</div>
            </div>
            <div v-else-if="selectedTab === 'friends'" class="w-full max-w-2xl bg-gray-800 p-8 rounded shadow-md">
              <h2 class="text-2xl font-bold mb-6 text-white">Friend Settings</h2>
              <div class="text-gray-300">Friend settings content goes here.</div>
            </div>
          </main>
        </div>
      </aside>
    </template>
  </UDrawer>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useUserStore } from '~/store/user-store'
import { UDrawer } from '@nuxthq/ui'

const props = defineProps<{ open: boolean, user: any }>()
const emit = defineEmits(['close'])

const drawerOpen = computed({
  get: () => props.open,
  set: (val) => { if (!val) emit('close') }
})

const tabs = [
  { key: 'profile', label: 'Profile' },
  { key: 'privacy', label: 'Privacy' },
  { key: 'friends', label: 'Friend Settings' },
]
const selectedTab = ref('profile')

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

watch(() => props.user, (user) => {
  if (user && typeof user === 'object' && 'username' in user) {
    username.value = user.username || ''
    email.value = user.email || ''
    bio.value = user.bio || ''
    status.value = user.status || ''
    avatarUrl.value = (user as any).avatarUrl || 'https://ui-avatars.com/api/?name=User'
  }
}, { immediate: true })

async function handleUpdate() {
  error.value = ''
  success.value = ''
  try {
    const res = await $fetch('/api/users/me', {
      method: 'put',
      body: {
        username: username.value,
        email: email.value,
        password: password.value || undefined,
        bio: bio.value,
        status: status.value,
      },
    })
    if (res && typeof res === 'object' && 'user' in res && res.user) {
      userStore.setUser(res.user)
      success.value = 'Profile updated successfully.'
      password.value = ''
    } else if (res && typeof res === 'object' && 'error' in res) {
      error.value = res.error || 'Update failed.'
    } else {
      error.value = 'Update failed.'
    }
  } catch (err: any) {
    error.value = err.data?.error || 'Update failed.'
  }
}

async function handleLogout() {
  try {
    await $fetch('/api/auth/logout', { method: 'POST' })
  } catch {}
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
      method: 'post',
      body: formData,
    })
    if (res && typeof res === 'object' && 'avatarUrl' in res && res.avatarUrl) {
      avatarUrl.value = res.avatarUrl
      userStore.setUser({ ...userStore.user, avatarUrl: res.avatarUrl })
      success.value = 'Avatar updated successfully.'
      avatarFile.value = null
    } else if (res && typeof res === 'object' && 'error' in res) {
      error.value = res.error || 'Avatar upload failed.'
    } else {
      error.value = 'Avatar upload failed.'
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
    })
    await handleLogout()
  } catch (err: any) {
    error.value = err.data?.error || 'Delete failed.'
  }
}
</script>

<style scoped>
.drawer-slide-enter-active,
.drawer-slide-leave-active {
  transition: transform 0.25s cubic-bezier(.4,0,.2,1), opacity 0.2s;
}
.drawer-slide-enter-from,
.drawer-slide-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
.drawer-slide-enter-to,
.drawer-slide-leave-from {
  transform: translateX(0);
  opacity: 1;
}
</style> 