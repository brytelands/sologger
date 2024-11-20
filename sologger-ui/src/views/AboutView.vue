<template>
  <div class="container mx-auto p-4">
    <h1 class="text-2xl font-bold mb-6 text-surface-800 dark:text-surface-100">
      About Sologger
    </h1>
    <div class="max-w-3xl text-surface-800 dark:text-surface-100">
      <p class="mb-6 text-lg">
        Sologger is a real-time Solana program log explorer and analysis tool.
        It helps developers monitor and debug their Solana programs across different networks.
      </p>

      <h2 class="text-xl font-bold mt-8 mb-4">Features</h2>
      <ul class="list-disc pl-6 mb-6 space-y-2">
        <li>Real-time log monitoring</li>
        <li>Multi-program support</li>
        <li>Cross-network compatibility</li>
        <li>Log parsing and analysis</li>
        <li>Data export capabilities</li>
      </ul>

      <h2 class="text-xl font-bold mt-8 mb-4">Getting Started</h2>
      <p class="mb-4">
        To begin monitoring your Solana program logs:
      </p>
      <ol class="list-decimal pl-6 mb-6 space-y-2">
        <li>Select a network (Devnet, Testnet, or enter your RPC URL (Helius, QuickNode, etc...). In order to listen to logs on Mainnet you must use your own URL</li>
        <li>Enter your program ID</li>
        <li>Click "Start Logs" to begin monitoring</li>
      </ol>

      <h2 class="text-xl font-bold mt-8 mb-4">Contributing</h2>
      <p class="mb-6">
        Contributions are welcome! Visit our
        <a
            href="https://github.com/brytelands/sologger"
            target="_blank"
            rel="noopener noreferrer"
            class="text-primary-400 hover:text-primary-500 underline"
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