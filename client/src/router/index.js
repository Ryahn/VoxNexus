import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const routes = [
  {
    path: '/auth',
    name: 'auth',
    component: () => import('../views/Auth.vue'),
    meta: { requiresGuest: true }
  },
  {
    path: '/',
    name: 'home',
    component: () => import('../views/HomeView.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/profile',
    name: 'profile',
    component: () => import('../views/ProfileView.vue'),
    meta: { requiresAuth: true }
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

// Navigation guard
router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore()
  const requiresAuth = to.matched.some(record => record.meta.requiresAuth)
  const requiresGuest = to.matched.some(record => record.meta.requiresGuest)

  // If the route requires auth and user is not authenticated
  if (requiresAuth && !authStore.isAuthenticated) {
    // Try to fetch current user
    try {
      await authStore.fetchCurrentUser()
      next()
    } catch (error) {
      next({ name: 'auth', query: { redirect: to.fullPath } })
    }
  }
  // If the route requires guest and user is authenticated
  else if (requiresGuest && authStore.isAuthenticated) {
    next({ name: 'home' })
  }
  // Otherwise proceed
  else {
    next()
  }
})

export default router