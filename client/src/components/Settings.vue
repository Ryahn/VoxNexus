<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg shadow-xl w-full max-w-md">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-semibold text-white">Settings</h2>
          <button
            @click="$emit('close')"
            class="text-gray-400 hover:text-white"
          >
            <font-awesome-icon icon="times" />
          </button>
        </div>
      </div>

      <!-- Content -->
      <div class="px-6 py-4">
        <nav class="space-y-1">
          <router-link
            to="/profile"
            class="flex items-center px-4 py-2 text-gray-300 hover:bg-gray-700 rounded-md group"
            @click="$emit('close')"
          >
            <font-awesome-icon icon="user" class="mr-3 text-gray-400 group-hover:text-gray-300" />
            Profile Settings
          </router-link>

          <button
            class="w-full flex items-center px-4 py-2 text-gray-300 hover:bg-gray-700 rounded-md group"
            @click="toggleTheme"
          >
            <font-awesome-icon icon="moon" class="mr-3 text-gray-400 group-hover:text-gray-300" />
            {{ isDarkMode ? 'Light Mode' : 'Dark Mode' }}
          </button>

          <button
            class="w-full flex items-center px-4 py-2 text-gray-300 hover:bg-gray-700 rounded-md group"
            @click="toggleNotifications"
          >
            <font-awesome-icon icon="bell" class="mr-3 text-gray-400 group-hover:text-gray-300" />
            {{ notificationsEnabled ? 'Disable Notifications' : 'Enable Notifications' }}
          </button>

          <button
            class="w-full flex items-center px-4 py-2 text-red-400 hover:bg-red-900 hover:text-red-300 rounded-md group"
            @click="handleLogout"
          >
            <font-awesome-icon icon="sign-out-alt" class="mr-3 text-red-400 group-hover:text-red-300" />
            Logout
          </button>
        </nav>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { library } from '@fortawesome/fontawesome-svg-core';
import { 
  faUser, 
  faMoon, 
  faBell, 
  faSignOutAlt, 
  faTimes 
} from '@fortawesome/free-solid-svg-icons';

library.add(faUser, faMoon, faBell, faSignOutAlt, faTimes);

const router = useRouter();
const authStore = useAuthStore();

const isDarkMode = ref(true);
const notificationsEnabled = ref(true);

const toggleTheme = () => {
  isDarkMode.value = !isDarkMode.value;
  // TODO: Implement theme switching
};

const toggleNotifications = () => {
  notificationsEnabled.value = !notificationsEnabled.value;
  // TODO: Implement notification toggling
};

const handleLogout = async () => {
  try {
    await authStore.logout();
    router.push('/login');
  } catch (error) {
    console.error('Logout failed:', error);
  }
};
</script> 