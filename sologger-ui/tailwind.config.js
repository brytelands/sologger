const primeui = require('tailwindcss-primeui');

export default {
    content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
    // `dark:` follows the app's theme toggle (main.js / useTheme set <html data-theme>,
    // resolving "system" to light or dark), not the OS setting, which can disagree with it
    darkMode: ['selector', '[data-theme="dark"]'],
    plugins: [primeui]
};
