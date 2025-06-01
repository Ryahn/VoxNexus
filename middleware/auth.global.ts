import { defineNuxtRouteMiddleware, useRouter } from 'nuxt/app'
import { useUserStore } from '~/store/user-store'

export default defineNuxtRouteMiddleware((to, from) => {
  const userStore = useUserStore()
  const publicPages = ['/auth/login', '/auth/register']
  // On server, check event.context.authUser
  if (process.server) {
    // @ts-ignore
    const authUser = useNuxtApp().ssrContext?.event?.context?.authUser
    if (!authUser && !publicPages.includes(to.path)) {
      return navigateTo('/auth/login')
    }
  } else {
    // On client, check Pinia store
    if (!userStore.user && !publicPages.includes(to.path)) {
      return navigateTo('/auth/login')
    }
  }
}) 