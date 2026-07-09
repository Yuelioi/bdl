import { createPinia } from 'pinia'
import ui from '@nuxt/ui/vue-plugin'
import { createApp } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'

import App from './App.vue'
import './styles/nuxt-ui.css'
import './styles/tokens.css'
import './styles/base.css'

const app = createApp(App)
const router = createRouter({
  history: createMemoryHistory(),
  routes: [],
})

app.use(createPinia())
app.use(router)
app.use(ui)
app.mount('#app')
