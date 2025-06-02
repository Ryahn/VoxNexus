import { createApp } from 'vue'
import './index.css'
import App from './App.vue'
import { createRouter, createWebHistory } from 'vue-router'
import Login from './views/auth/Login.vue'
import Register from './views/auth/Register.vue'
import ForgotPassword from './views/auth/ForgotPassword.vue'

const routes = [
  { path: '/login', component: Login },
  { path: '/register', component: Register },
  { path: '/forgot-password', component: ForgotPassword },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

createApp(App).use(router).mount('#app')
