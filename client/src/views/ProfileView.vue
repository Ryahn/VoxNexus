<template>
  <div class="min-h-screen bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-3xl mx-auto">
      <!-- Profile Header -->
      <div class="bg-gray-800 rounded-lg shadow-lg p-6 mb-6">
        <div class="relative">
          <!-- Banner -->
          <div class="relative h-48 rounded-t-lg overflow-hidden mb-16">
            <img
              v-if="user?.banner"
              :src="user.banner"
              alt="Profile banner"
              class="w-full h-full object-cover"
            />
            <div
              v-else
              class="w-full h-full bg-indigo-600"
            ></div>
            <button
              @click="triggerBannerUpload"
              class="absolute bottom-4 right-4 px-4 py-2 bg-black bg-opacity-50 text-white rounded-md hover:bg-opacity-70 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              <font-awesome-icon icon="camera" class="mr-2" />
              Change Banner
            </button>
            <input
              ref="bannerInput"
              type="file"
              accept="image/*"
              class="hidden"
              @change="handleBannerUpload"
            />
          </div>

          <!-- Avatar -->
          <div class="absolute -bottom-12 left-6">
            <div class="relative">
              <div
                class="h-24 w-24 rounded-full bg-indigo-600 flex items-center justify-center overflow-hidden"
              >
                <img
                  v-if="user?.avatar"
                  :src="user.avatar"
                  alt="Profile avatar"
                  class="w-full h-full object-cover"
                />
                <span v-else class="text-3xl text-white">{{ userInitials }}</span>
              </div>
              <button
                @click="triggerAvatarUpload"
                class="absolute bottom-0 right-0 p-2 bg-black bg-opacity-50 text-white rounded-full hover:bg-opacity-70 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <font-awesome-icon icon="camera" />
              </button>
              <input
                ref="avatarInput"
                type="file"
                accept="image/*"
                class="hidden"
                @change="handleAvatarUpload"
              />
            </div>
          </div>
        </div>

        <div class="mt-16 flex items-center justify-between">
          <div>
            <h1 class="text-2xl font-bold text-white">{{ user?.username }}</h1>
            <p class="text-gray-400">{{ user?.email }}</p>
          </div>
          <div>
            <button
              @click="isEditing = !isEditing"
              class="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              {{ isEditing ? 'Cancel' : 'Edit Profile' }}
            </button>
          </div>
        </div>
      </div>

      <!-- Profile Form -->
      <div v-if="isEditing" class="bg-gray-800 rounded-lg shadow-lg p-6 mb-6">
        <form @submit.prevent="handleSubmit" class="space-y-6">
          <div>
            <label for="username" class="block text-sm font-medium text-gray-300">Username</label>
            <input
              type="text"
              id="username"
              v-model="form.username"
              class="mt-1 block w-full rounded-md bg-gray-700 border-gray-600 text-white shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              required
            />
          </div>

          <div>
            <label for="email" class="block text-sm font-medium text-gray-300">Email</label>
            <input
              type="email"
              id="email"
              v-model="form.email"
              class="mt-1 block w-full rounded-md bg-gray-700 border-gray-600 text-white shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              required
            />
          </div>

          <div>
            <label for="currentPassword" class="block text-sm font-medium text-gray-300">Current Password</label>
            <input
              type="password"
              id="currentPassword"
              v-model="form.currentPassword"
              class="mt-1 block w-full rounded-md bg-gray-700 border-gray-600 text-white shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            />
          </div>

          <div>
            <label for="newPassword" class="block text-sm font-medium text-gray-300">New Password</label>
            <input
              type="password"
              id="newPassword"
              v-model="form.newPassword"
              class="mt-1 block w-full rounded-md bg-gray-700 border-gray-600 text-white shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            />
          </div>

          <div>
            <label for="confirmPassword" class="block text-sm font-medium text-gray-300">Confirm New Password</label>
            <input
              type="password"
              id="confirmPassword"
              v-model="form.confirmPassword"
              class="mt-1 block w-full rounded-md bg-gray-700 border-gray-600 text-white shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            />
          </div>

          <div class="flex justify-end space-x-4">
            <button
              type="button"
              @click="isEditing = false"
              class="px-4 py-2 border border-gray-600 text-gray-300 rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              Cancel
            </button>
            <button
              type="submit"
              :disabled="loading"
              class="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
            >
              {{ loading ? 'Saving...' : 'Save Changes' }}
            </button>
          </div>
        </form>
      </div>

      <!-- Active Sessions -->
      <div class="bg-gray-800 rounded-lg shadow-lg p-6">
        <h2 class="text-xl font-bold text-white mb-4">Active Sessions</h2>
        <div class="space-y-4">
          <div v-for="session in sessions" :key="session.id" class="flex items-center justify-between p-4 bg-gray-700 rounded-lg">
            <div>
              <p class="text-white">{{ session.device }}</p>
              <p class="text-sm text-gray-400">Last active: {{ formatDate(session.lastActive) }}</p>
            </div>
            <button
              @click="logoutSession(session.id)"
              class="px-3 py-1 text-sm text-red-400 hover:text-red-300"
            >
              Logout
            </button>
          </div>
        </div>
        <div class="mt-4">
          <button
            @click="logoutAll"
            class="w-full px-4 py-2 text-red-400 border border-red-400 rounded-md hover:bg-red-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
          >
            Logout All Sessions
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useAuthStore } from '../stores/auth';
import { useRouter } from 'vue-router';
import { library } from '@fortawesome/fontawesome-svg-core';
import { faCamera } from '@fortawesome/free-solid-svg-icons';

library.add(faCamera);

const router = useRouter();
const authStore = useAuthStore();
const isEditing = ref(false);
const loading = ref(false);
const error = ref('');

const form = ref({
  username: '',
  email: '',
  currentPassword: '',
  newPassword: '',
  confirmPassword: ''
});

const sessions = ref([]);

const user = computed(() => authStore.user);

const userInitials = computed(() => {
  if (!user.value?.username) return '';
  return user.value.username
    .split(' ')
    .map(word => word[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);
});

const formatDate = (date) => {
  return new Date(date).toLocaleString();
};

const loadSessions = async () => {
  try {
    sessions.value = await authStore.getSessions();
  } catch (err) {
    console.error('Failed to load sessions:', err);
  }
};

const handleSubmit = async () => {
  try {
    loading.value = true;
    error.value = '';

    if (form.value.newPassword) {
      if (form.value.newPassword !== form.value.confirmPassword) {
        throw new Error('New passwords do not match');
      }
      if (!form.value.currentPassword) {
        throw new Error('Current password is required to set a new password');
      }
      await authStore.updatePassword(form.value.currentPassword, form.value.newPassword);
    }

    await authStore.updateProfile({
      username: form.value.username,
      email: form.value.email
    });

    isEditing.value = false;
    form.value = {
      username: '',
      email: '',
      currentPassword: '',
      newPassword: '',
      confirmPassword: ''
    };
  } catch (err) {
    error.value = err.message;
  } finally {
    loading.value = false;
  }
};

const logoutSession = async (sessionId) => {
  try {
    await authStore.logoutSession(sessionId);
    sessions.value = sessions.value.filter(session => session.id !== sessionId);
  } catch (err) {
    console.error('Failed to logout session:', err);
  }
};

const logoutAll = async () => {
  try {
    await authStore.logoutAll();
    router.push('/login');
  } catch (err) {
    console.error('Failed to logout all sessions:', err);
  }
};

const avatarInput = ref(null);
const bannerInput = ref(null);

const triggerAvatarUpload = () => {
  avatarInput.value.click();
};

const triggerBannerUpload = () => {
  bannerInput.value.click();
};

const handleAvatarUpload = async (event) => {
  const file = event.target.files[0];
  if (!file) return;

  try {
    loading.value = true;
    const formData = new FormData();
    formData.append('avatar', file);
    await authStore.uploadAvatar(formData);
  } catch (err) {
    error.value = err.message;
  } finally {
    loading.value = false;
  }
};

const handleBannerUpload = async (event) => {
  const file = event.target.files[0];
  if (!file) return;

  try {
    loading.value = true;
    const formData = new FormData();
    formData.append('banner', file);
    await authStore.uploadBanner(formData);
  } catch (err) {
    error.value = err.message;
  } finally {
    loading.value = false;
  }
};

onMounted(() => {
  if (user.value) {
    form.value.username = user.value.username;
    form.value.email = user.value.email;
  }
  loadSessions();
});
</script> 