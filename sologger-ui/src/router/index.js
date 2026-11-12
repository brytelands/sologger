import {createRouter, createWebHistory} from 'vue-router';

// Each view is its own chunk, loaded when first visited: the log table (SlickGrid) and
// charts (Chart.js) only download for the views that use them.
const routes = [
    {
        path: '/',
        name: 'home',
        component: () => import('../views/HomeView.vue')
    },
    {
        path: '/convert',
        name: 'convert',
        component: () => import('../views/ConvertView.vue')
    },
    {
        path: '/lookup',
        name: 'lookup',
        component: () => import('../views/LookupView.vue')
    },
    {
        path: '/about',
        name: 'about',
        component: () => import('../views/AboutView.vue')
    }
];

const router = createRouter({
    history: createWebHistory('/sologger/'),
    routes
})

// A tab opened before a redeploy asks for view chunks that no longer exist. Load the page
// it was navigating to from the server instead, which brings the new build. At most once
// per 10s, so a chunk that is really missing cannot cause a reload loop.
const CHUNK_ERROR = /dynamically imported module|Importing a module script failed/i;
router.onError((error, to) => {
    if (!CHUNK_ERROR.test(error?.message ?? '')) return;
    const last = Number(sessionStorage.getItem('sologger_chunkReloadAt') || 0);
    if (Date.now() - last < 10_000) return;
    sessionStorage.setItem('sologger_chunkReloadAt', String(Date.now()));
    window.location.assign(router.resolve(to.fullPath).href);
});

export default router;
