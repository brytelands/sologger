// src/App.vue
<template>
  <nav class="bg-surface-800 text-surface-100 p-4 mb-6">
    <div class="container mx-auto flex items-center justify-between">
      <div class="flex items-center gap-6">
        <router-link
            to="/"
            class="hover:text-primary-400 transition-colors"
        >
          Home
        </router-link>
        <router-link
            to="/convert"
            class="hover:text-primary-400 transition-colors"
        >
          Convert Logs
        </router-link>
        <router-link
            to="/about"
            class="hover:text-primary-400 transition-colors"
        >
          About
        </router-link>
      </div>

      <div class="absolute left-1/2 transform -translate-x-1/2 flex items-center">
        <span class="text-2xl font-bold text-primary-400">Sologger</span>
      </div>

      <div class="flex items-center">
        <a
            href="https://github.com/wkennedy"
            target="_blank"
            rel="noopener noreferrer"
            class="hover:text-primary-400 transition-colors"
        >
          <svg
              class="h-6 w-6"
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
          >
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
          </svg>
        </a>
      </div>
    </div>
  </nav>

  <router-view></router-view>
</template>

<script>
export default {
  name: 'App'
};
</script>

<style>
/* Your existing styles */
</style>
<style scoped>
.router-link-active {
  color: var(--p-primary-400);
  font-weight: 500;
}

nav {
  border-bottom: 1px solid var(--p-surface-700);
}

@media (prefers-color-scheme: dark) {
  nav {
    background-color: var(--p-surface-900);
    border-bottom-color: var(--p-surface-800);
  }
}
</style>