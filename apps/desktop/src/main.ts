import { createPinia } from 'pinia'
import ui from '@nuxt/ui/vue-plugin'
import { createApp } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'

import App from './App.vue'
import { useThemeStore } from './stores/theme'
import './styles/nuxt-ui.css'
import './styles/tokens.css'
import './styles/feedback.css'
import './styles/forms.css'
import './styles/base.css'

const app = createApp(App)
const pinia = createPinia()
const router = createRouter({
  history: createMemoryHistory(),
  routes: [{ path: '/', component: { render: () => null } }],
})

app.use(pinia)
app.use(router)
app.use(ui)
const theme = useThemeStore(pinia)
theme.initialize()
if (import.meta.hot) import.meta.hot.dispose(() => theme.dispose())
app.mount('#app')
