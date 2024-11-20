// src/views/AboutView.vue
<template>
  <div class="container mx-auto p-4">
    <h1 class="text-2xl font-bold mb-6 text-surface-800 dark:text-surface-100">
      About Sologger
    </h1>
    <div class="prose dark:prose-invert">
      <p>
        Sologger is a real-time Solana program log explorer and analysis tool.
        It helps developers monitor and debug their Solana programs across different networks.
      </p>

      <h2>Features</h2>
      <ul>
        <li>Real-time log monitoring</li>
        <li>Multi-program support</li>
        <li>Cross-network compatibility</li>
        <li>Log parsing and analysis</li>
        <li>Data export capabilities</li>
      </ul>

      <h2>Getting Started</h2>
      <p>
        To begin monitoring your Solana program logs:
      </p>
      <ol>
        <li>Select a network (Mainnet, Devnet, or Testnet)</li>
        <li>Enter your program ID</li>
        <li>Click "Start Logs" to begin monitoring</li>
      </ol>

      <h2>Contributing</h2>
      <p>
        Contributions are welcome! Visit our
        <a
            href="https://github.com/wkennedy"
            target="_blank"
            rel="noopener noreferrer"
            class="text-primary-500 hover:text-primary-600"
        >
          GitHub repository
        </a>
        to get started.
      </p>
    </div>
  </div>
</template>

<script>
export default {
  name: 'AboutView'
};
</script>