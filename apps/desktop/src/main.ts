import { addCollection } from '@iconify/vue';
import { createPinia } from 'pinia';
import ui from '@nuxt/ui/vue-plugin';
import { createApp } from 'vue';
import { createMemoryHistory, createRouter } from 'vue-router';
import localIcons from 'virtual:bdl-icon-bundle';

import App from './App.vue';
import { useThemeStore } from './shared/stores/theme';
import './shared/styles/nuxt-ui.css';
import './shared/styles/tokens.css';
import './shared/styles/feedback.css';
import './shared/styles/base.css';

addCollection(localIcons);

const app = createApp(App);
const pinia = createPinia();
const router = createRouter({
  history: createMemoryHistory(),
  routes: [{ path: '/', component: { render: () => null } }],
});

app.use(pinia);
app.use(router);
app.use(ui);
const theme = useThemeStore(pinia);
theme.initialize();
if (import.meta.hot) import.meta.hot.dispose(() => theme.dispose());
app.mount('#app');
