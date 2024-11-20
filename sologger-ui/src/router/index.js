import { createRouter, createWebHistory } from 'vue-router';
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
    history: createWebHistory('/sologger/'),
    routes
})
export default router;