# AI Agent Guide for sologger-ui

This document provides essential information for AI agents working on the `sologger-ui` module.

## Project Overview
`sologger-ui` is a web application for visualizing and transforming Solana logs. It is built using **Vue 3**, **Vite**, and **JavaScript**.

## Technology Stack
- **Frontend Framework:** Vue 3 (Composition API and Options API used)
- **Build Tool:** Vite
- **Styling:** Tailwind CSS + PostCSS
- **UI Components:** PrimeVue (v4)
- **Routing:** Vue Router
- **Data Visualization:** Handsontable (via `@handsontable/vue3`), Chart.js (via `vue-chartjs`)
- **WebAssembly:** Integrates with `sologger-log-transformer-wasm` for log processing.

## Project Structure
- `src/components/`: Reusable Vue components (e.g., `LogsTable.vue`, `StatsGrid.vue`).
- `src/views/`: Main page components (e.g., `HomeView.vue`, `ConvertView.vue`).
- `src/composables/`: Vue composables (e.g., `useTheme.js`).
- `src/router/`: Navigation configuration.
- `src/assets/`: Static assets and global CSS.
- `public/`: Static files, including the WASM transformer package.

## Key Development Guidelines

### State Management
- Primarily uses Vue's `ref` and `reactive` within components or composables.
- Theme management is handled via the `useTheme` composable and stored in `localStorage`.

### Styling
- Use **Tailwind CSS** utility classes for layout and styling.
- PrimeVue components are used with `theme: "none"` in `main.js`, meaning they rely heavily on Tailwind and custom CSS.
- Custom CSS is located in `src/style.css` and component-specific `<style>` blocks.

### WASM Integration
- The application uses a WebAssembly module for log transformation.
- The WASM module is imported from `../../public/sologger-log-transformer-wasm/pkg/sologger_log_transformer_wasm.js` in views like `ConvertView.vue`.
- When interacting with the WASM transformer, ensure `BigInt` is used for slot numbers where required.

### Code Style
- Follow existing patterns in the codebase.
- Use `export default` for component definitions.
- Maintain consistent indentation (2 spaces).

## Common Tasks
- **Adding a new view:** Create a `.vue` file in `src/views/` and register it in `src/router/index.js`.
- **Modifying styles:** Prefer Tailwind classes in templates. If global changes are needed, check `src/assets/tailwind.css` or `src/style.css`.
- **Updating WASM:** If the Rust-based transformer changes, the files in `public/sologger-log-transformer-wasm/pkg/` may need to be refreshed.

## Development Commands
- `npm run dev`: Start the development server.
- `npm run build`: Build the application for production.
- `npm run preview`: Build and preview the production build.
