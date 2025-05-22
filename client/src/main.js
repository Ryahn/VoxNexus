import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './index.css'
import { library } from '@fortawesome/fontawesome-svg-core'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { faUser, faMicrophone, faCog, faMicrophoneSlash } from '@fortawesome/free-solid-svg-icons'
import axios from 'axios'

// Configure axios defaults
axios.defaults.baseURL = 'http://localhost:5000'
axios.defaults.withCredentials = true

// Add CSRF token to all requests
axios.interceptors.request.use(config => {
  // Get CSRF token from cookie
  const token = document.cookie.split('; ')
    .find(row => row.startsWith('XSRF-TOKEN='))
    ?.split('=')[1]
  
  if (token) {
    config.headers['X-CSRF-TOKEN'] = decodeURIComponent(token)
  }
  return config
})

library.add(faUser, faMicrophone, faCog, faMicrophoneSlash)

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.component('font-awesome-icon', FontAwesomeIcon)
app.mount('#app')
