import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import Aura from '@primevue/themes/aura';
import AnimateOnScroll from 'primevue/animateonscroll';

import App from './App.vue';
import router from './router';

import 'primeicons/primeicons.css';
import './assets/main.css';

const app = createApp(App);

const pinia = createPinia();
app.use(pinia);
app.use(router);

app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      darkModeSelector: '.dark',
      cssLayer: false,
    },
  },
});

app.directive('animateonscroll', AnimateOnScroll);

app.mount('#app');
