<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl h-[80vh] flex">
      <!-- Left Sidebar -->
      <div class="w-64 border-r border-gray-700 p-4">
        <div class="flex items-center justify-between mb-6">
          <h2 class="text-xl font-semibold text-white">Settings</h2>
          <button
            @click="$emit('close')"
            class="text-gray-400 hover:text-white"
          >
            <font-awesome-icon icon="times" />
          </button>
        </div>
        
        <nav class="space-y-1">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            @click="activeTab = tab.id"
            class="w-full flex items-center px-4 py-2 text-gray-300 hover:bg-gray-700 rounded-md group"
            :class="{ 'bg-gray-700': activeTab === tab.id }"
          >
            <font-awesome-icon :icon="tab.icon" class="mr-3 text-gray-400 group-hover:text-gray-300" />
            {{ tab.name }}
          </button>
        </nav>
      </div>

      <!-- Right Content -->
      <div class="flex-1 p-6 overflow-y-auto">
        <!-- Profile Settings -->
        <div v-if="activeTab === 'profile'" class="space-y-6">
          <h3 class="text-lg font-semibold text-white mb-4">Profile Settings</h3>
          
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Username</label>
              <input
                v-model="settings.profile.username"
                type="text"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Email</label>
              <input
                v-model="settings.profile.email"
                type="email"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Password</label>
              <button
                class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-md"
                @click="changePassword"
              >
                Change Password
              </button>
            </div>

            <div class="flex items-center justify-between">
              <div>
                <label class="block text-sm font-medium text-gray-300">Two-Factor Authentication</label>
                <p class="text-sm text-gray-400">Add an extra layer of security to your account</p>
              </div>
              <button
                class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-md"
                @click="toggle2FA"
              >
                {{ settings.profile.twoFactorEnabled ? 'Disable' : 'Enable' }} 2FA
              </button>
            </div>
          </div>
        </div>

        <!-- Privacy Settings -->
        <div v-if="activeTab === 'privacy'" class="space-y-6">
          <h3 class="text-lg font-semibold text-white mb-4">Privacy Settings</h3>
          
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Friend Requests</label>
              <select
                v-model="settings.privacy.friendRequests"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="everyone">Everyone</option>
                <option value="friendsOfFriends">Friends of Friends</option>
                <option value="disabled">Disabled</option>
              </select>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Direct Messages</label>
              <select
                v-model="settings.privacy.directMessages"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="everyone">Everyone</option>
                <option value="serverOnly">Server Members Only</option>
                <option value="friendsOnly">Friends Only</option>
                <option value="disabled">Disabled</option>
              </select>
            </div>
          </div>
        </div>

        <!-- Application Settings -->
        <div v-if="activeTab === 'application'" class="space-y-6">
          <h3 class="text-lg font-semibold text-white mb-4">Application Settings</h3>
          
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Theme</label>
              <select
                v-model="settings.application.theme"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>

            <div class="flex items-center justify-between">
              <div>
                <label class="block text-sm font-medium text-gray-300">Notifications</label>
                <p class="text-sm text-gray-400">Receive notifications for mentions and messages</p>
              </div>
              <button
                class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-md"
                @click="settings.application.notifications = !settings.application.notifications"
              >
                {{ settings.application.notifications ? 'Disable' : 'Enable' }}
              </button>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Language</label>
              <select
                v-model="settings.application.language"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="en">English</option>
                <option value="es">Español</option>
                <option value="fr">Français</option>
                <option value="de">Deutsch</option>
              </select>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Timezone</label>
              <select
                v-model="settings.application.timezone"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option v-for="tz in timezones" :key="tz" :value="tz">{{ tz }}</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { library } from '@fortawesome/fontawesome-svg-core'
import { 
  faUser, 
  faShieldAlt, 
  faCog,
  faTimes
} from '@fortawesome/free-solid-svg-icons'

// Define component emits
const emit = defineEmits<{
  (e: 'close'): void
}>()

// Register Font Awesome icons
library.add(faUser, faShieldAlt, faCog, faTimes)

// Initialize settings store
const settingsStore = useSettingsStore()
const settings = computed(() => settingsStore.settings)

// Active tab state
const activeTab = ref('profile')

// Available tabs
const tabs = [
  { id: 'profile', name: 'Profile', icon: 'user' },
  { id: 'privacy', name: 'Privacy', icon: 'shield-alt' },
  { id: 'application', name: 'Application', icon: 'cog' }
]

// Timezone options
const timezones = Intl.supportedValuesOf('timeZone')

// Methods
const changePassword = () => {
  // TODO: Implement password change modal
  console.log('Change password clicked')
}

const toggle2FA = () => {
  settingsStore.updateSetting('profile', 'twoFactorEnabled', !settings.value.profile.twoFactorEnabled)
}
</script>

<style scoped>
/* Add any component-specific styles here */
</style> 