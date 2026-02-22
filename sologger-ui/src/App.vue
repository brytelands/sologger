<!-- src/App.vue -->
<template>
  <div class="app">
    <nav class="px-4 py-3 mb-0 relative">
      <div class="container mx-auto">
        <!-- Mobile Menu Button -->
        <div class="md:hidden flex items-center justify-between">
          <button
              @click="isMobileMenuOpen = !isMobileMenuOpen"
              class="text-surface-50 p-2 rounded-lg hover:bg-white/10 transition-colors"
              aria-label="Toggle menu"
          >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
            >
              <path
                  v-if="!isMobileMenuOpen"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 6h16M4 12h16M4 18h16"
              />
              <path
                  v-else
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>

          <!-- Center Logo for Mobile -->
          <div class="flex items-center">
            <span class="text-xl font-bold text-primary-400">
              <img src="/sologger_header.png" alt="Sologger" class="h-8"/>
            </span>
          </div>

          <!-- Placeholder div for spacing -->
          <div class="w-8"></div>
        </div>

        <!-- Desktop Navigation -->
        <div class="hidden md:flex items-center justify-between">
          <div class="flex items-center gap-1">
            <router-link
                to="/"
                class="px-3 py-2 rounded-lg text-sm font-medium text-surface-50/80 hover:text-surface-50 hover:bg-white/10 transition-all"
            >
              Home
            </router-link>
            <router-link
                to="/convert"
                class="px-3 py-2 rounded-lg text-sm font-medium text-surface-50/80 hover:text-surface-50 hover:bg-white/10 transition-all"
            >
              Convert Logs
            </router-link>
            <router-link
                to="/about"
                class="px-3 py-2 rounded-lg text-sm font-medium text-surface-50/80 hover:text-surface-50 hover:bg-white/10 transition-all"
            >
              About
            </router-link>
          </div>

          <div class="flex items-center">
            <span class="text-2xl font-bold text-primary-400">
              <img src="/sologger_header.png" alt="Sologger"/>
            </span>
          </div>

          <div class="flex items-center gap-4">
            <!-- Theme Toggle -->
            <div class="relative theme-toggle">
              <button
                  type="button"
                  @click.stop="toggleThemeMenu"
                  class="p-2 rounded-lg bg-white/10 hover:bg-white/20 transition-colors"
                  :aria-label="'Current theme: ' + theme"
              >
                <!-- Sun icon for light mode -->
                <svg
                    v-if="theme === 'light'"
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5 text-gray-100"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                  <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
                  />
                </svg>
                <!-- Moon icon for dark mode -->
                <svg
                    v-else-if="theme === 'dark'"
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5 text-gray-100"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                  <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
                  />
                </svg>
                <!-- System icon -->
                <svg
                    v-else
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5 text-gray-100"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                  <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                  />
                </svg>
              </button>

              <div
                  v-if="isThemeMenuOpen"
                  class="absolute right-0 mt-2 w-44 rounded-xl shadow-xl bg-surface-800 border border-white/10 z-50 overflow-hidden"
              >
                <div class="py-1" role="menu">
                  <button
                      v-for="option in themeOptions"
                      :key="option.value"
                      type="button"
                      @click="selectTheme(option.value)"
                      class="w-full text-left px-4 py-2 text-sm text-gray-100 hover:bg-white/10 transition-colors"
                      :class="{ 'text-primary-400 font-medium': theme === option.value }"
                      role="menuitem"
                  >
                    {{ option.label }}
                  </button>
                </div>
              </div>
            </div>

            <a
                href="https://github.com/brytelands/sologger"
                target="_blank"
                rel="noopener noreferrer"
                class="p-2 rounded-lg text-surface-50/80 hover:text-surface-50 hover:bg-white/10 transition-all"
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

        <!-- Mobile Menu -->
        <div
            v-if="isMobileMenuOpen"
            class="md:hidden absolute top-full left-0 right-0 bg-surface-800 border-t border-white/10 shadow-xl z-50"
        >
          <div class="px-2 pt-2 pb-3 space-y-1">
            <router-link
                v-for="link in navigationLinks"
                :key="link.to"
                :to="link.to"
                class="block px-4 py-2.5 rounded-lg text-sm font-medium text-surface-50/80 hover:text-surface-50 hover:bg-white/10 transition-all"
                @click="isMobileMenuOpen = false"
            >
              {{ link.text }}
            </router-link>
          </div>
          <div class="px-4 py-3 border-t border-white/10">
            <div class="flex items-center justify-between">
              <div class="theme-toggle">
                <button
                    v-for="option in themeOptions"
                    :key="option.value"
                    type="button"
                    @click="selectTheme(option.value)"
                    class="mr-3 text-sm text-gray-100"
                    :class="{ 'text-primary-400': theme === option.value }"
                >
                  {{ option.label }}
                </button>
              </div>
              <a
                  href="https://github.com/brytelands/sologger"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="p-2 rounded-lg text-surface-50/80 hover:text-surface-50 hover:bg-white/10 transition-all"
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
        </div>
      </div>
    </nav>

    <router-view></router-view>
  </div>
</template>

<script>
import { ref, computed, onUnmounted } from 'vue';
import { useTheme } from './composables/useTheme';

export default {
  name: 'App',
  setup() {
    const { theme, updateTheme } = useTheme();
    const isThemeMenuOpen = ref(false);
    const isMobileMenuOpen = ref(false);

    const isDarkSystem = computed(() => {
      return window.matchMedia('(prefers-color-scheme: dark)').matches;
    });

    const themeOptions = [
      { value: 'light', label: 'Light' },
      { value: 'dark', label: 'Dark' },
      { value: 'system', label: 'System' }
    ];

    const navigationLinks = [
      { to: '/', text: 'Home' },
      { to: '/convert', text: 'Convert Logs' },
      { to: '/about', text: 'About' }
    ];

    const toggleThemeMenu = () => {
      isThemeMenuOpen.value = !isThemeMenuOpen.value;
    };

    const selectTheme = (newTheme) => {
      updateTheme(newTheme);
      isThemeMenuOpen.value = false;
    };

    // Close menu when clicking outside
    const handleClickOutside = (e) => {
      if (isThemeMenuOpen.value && !e.target.closest('.theme-toggle')) {
        isThemeMenuOpen.value = false;
      }
    };

    // Add event listener
    window.addEventListener('click', handleClickOutside);

    // Cleanup
    onUnmounted(() => {
      window.removeEventListener('click', handleClickOutside);
    });

    return {
      theme,
      isDarkSystem,
      isThemeMenuOpen,
      isMobileMenuOpen,
      themeOptions,
      navigationLinks,
      toggleThemeMenu,
      selectTheme
    };
  }
};
</script>

<style>
.router-link-active {
  color: var(--p-primary-400) !important;
  font-weight: 600;
  background-color: rgba(255, 69, 19, 0.12) !important;
}

@media (max-width: 768px) {
  .container {
    padding-left: 1rem;
    padding-right: 1rem;
  }
}
</style>