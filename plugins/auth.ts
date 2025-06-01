import { defineNuxtPlugin, useNuxtApp } from '#app'
import { useUserStore } from '~/store/user-store'

export default defineNuxtPlugin(async (nuxtApp) => {
  const userStore = useUserStore()
  // Hydrate from SSR context if available
  // @ts-ignore
  const ssrUser = nuxtApp.ssrContext?.event?.context?.authUser
  if (ssrUser && !userStore.user) {
    userStore.setUser(ssrUser)
  }
  // On client, if still not set, fetch from API
  if (process.client && !userStore.user) {
    try {
      await userStore.fetchProfile()
    } catch (e) {
      userStore.clearUser()
    }
  }
}) 