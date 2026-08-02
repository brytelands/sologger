import PrimeVue from "primevue/config";
import ToastService from "primevue/toastservice";
import { createApp } from "vue";
import App from "./App.vue";
import "./assets/tailwind.css";
import "./style.css";
import router from "./router";

const initialTheme = localStorage.getItem('theme') || 'light';
const effectiveTheme = initialTheme === 'system'
    ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
    : initialTheme;
document.documentElement.setAttribute('data-theme', effectiveTheme);

const app = createApp(App);
app.use(router);
app.use(PrimeVue, { theme: "none" });
app.use(ToastService);
app.mount("#app");

