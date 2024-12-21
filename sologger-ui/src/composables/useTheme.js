// src/composables/useTheme.js
import { ref, onMounted, onUnmounted } from 'vue';

export const useTheme = () => {
    const theme = ref(localStorage.getItem('theme') || 'system');

    const getSystemTheme = () => {
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    };

    const updateTheme = (newTheme) => {
        theme.value = newTheme;
        localStorage.setItem('theme', newTheme);

        const effectiveTheme = newTheme === 'system' ? getSystemTheme() : newTheme;
        document.documentElement.setAttribute('data-theme', effectiveTheme);
    };

    // Handle system theme changes
    const systemThemeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleSystemThemeChange = (e) => {
        if (theme.value === 'system') {
            document.documentElement.setAttribute('data-theme', e.matches ? 'dark' : 'light');
        }
    };

    onMounted(() => {
        // Initialize theme
        updateTheme(theme.value);

        // Add system theme change listener
        systemThemeMediaQuery.addEventListener('change', handleSystemThemeChange);
    });

    onUnmounted(() => {
        // Clean up listener
        systemThemeMediaQuery.removeEventListener('change', handleSystemThemeChange);
    });

    return {
        theme,
        updateTheme
    };
};