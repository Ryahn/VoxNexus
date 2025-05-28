import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { useAuthStore } from './auth'

export interface UserSettings {
  profile: {
    username: string
    email: string
    twoFactorEnabled: boolean
  }
  privacy: {
    friendRequests: 'everyone' | 'friendsOfFriends' | 'disabled'
    directMessages: 'everyone' | 'serverOnly' | 'friendsOnly' | 'disabled'
  }
  application: {
    theme: 'dark' | 'light'
    notifications: boolean
    language: string
    timezone: string
  }
}

export const useSettingsStore = defineStore('settings', () => {
  const authStore = useAuthStore()
  
  // Initialize settings with default values
  const settings = ref<UserSettings>({
    profile: {
      username: authStore.currentUser?.username || '',
      email: authStore.currentUser?.email || '',
      twoFactorEnabled: false
    },
    privacy: {
      friendRequests: 'everyone',
      directMessages: 'everyone'
    },
    application: {
      theme: 'dark',
      notifications: true,
      language: 'en',
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone
    }
  })

  // Load settings from localStorage on initialization
  const loadSettings = () => {
    const savedSettings = localStorage.getItem('userSettings')
    if (savedSettings) {
      settings.value = JSON.parse(savedSettings)
    }
  }

  // Auto-save settings when they change
  watch(settings, (newSettings) => {
    localStorage.setItem('userSettings', JSON.stringify(newSettings))
  }, { deep: true })

  // Update specific setting
  const updateSetting = <K extends keyof UserSettings>(
    category: K,
    key: keyof UserSettings[K],
    value: any
  ) => {
    if (settings.value[category]) {
      settings.value[category][key] = value
    }
  }

  // Initialize settings
  loadSettings()

  return {
    settings,
    updateSetting
  }
}) 