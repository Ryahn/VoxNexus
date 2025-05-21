import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import './index.css'
import { library } from '@fortawesome/fontawesome-svg-core'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { faUser, faMicrophone, faCog, faMicrophoneSlash } from '@fortawesome/free-solid-svg-icons'

library.add(faUser, faMicrophone, faCog, faMicrophoneSlash)

const app = createApp(App)
app.use(router)
app.component('font-awesome-icon', FontAwesomeIcon)
app.mount('#app')
