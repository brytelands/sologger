import {createRouter, createWebHashHistory, createWebHistory} from 'vue-router';
import HomeView from '../views/HomeView.vue';
import ConvertView from '../views/ConvertView.vue';
import AboutView from '../views/AboutView.vue';

const routes = [
    {
        path: '/',
        name: 'home',
        component: HomeView
    },
    {
        path: '/convert',
        name: 'convert',
        component: ConvertView
    },
    {
        path: '/about',
        name: 'about',
        component: AboutView
    }
];

const router = createRouter({
    history: createWebHashHistory(import.meta.env.BASE_URL),
    routes
})
export default router;