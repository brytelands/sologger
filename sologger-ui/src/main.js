import Button from "primevue/button";
import PrimeVue from "primevue/config";
import InputText from "primevue/inputtext";
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
app.component("InputText", InputText);
app.component("Button", Button);
app.use(PrimeVue, { theme: "none" });
app.mount("#app");

