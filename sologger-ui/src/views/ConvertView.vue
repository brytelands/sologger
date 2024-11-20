<template>
  <div class="container mx-auto p-4">
    <h1 class="text-2xl font-bold mb-6 text-surface-800 dark:text-surface-100">
      Convert Solana Logs
    </h1>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div class="space-y-4">
        <textarea
            v-model="inputLogs"
            placeholder="Paste your raw Solana logs here, for example:

Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]
Program log: Instruction: Initialize
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program log: Initialized new event. Current value
Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 consumed 59783 of 200000 compute units
Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success
Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]
Program log: Create
Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units
Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)
Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete"
            class="w-full h-[600px] p-4 font-mono text-sm rounded-lg border border-surface-200 bg-surface-800 text-white dark:bg-surface-800 dark:border-surface-700"
            @input="handleInput"
        ></textarea>
        <button
            @click="convertLogs"
            class="px-4 py-2 bg-primary-500 text-white rounded hover:bg-primary-600 disabled:opacity-50"
            :disabled="!inputLogs"
        >
          Convert Logs
        </button>
      </div>

      <div class="space-y-4">
        <div
            v-if="convertedLogs"
            class="h-[600px] p-4 font-mono text-sm rounded-lg border border-surface-200 bg-surface-800 text-white dark:bg-surface-800 dark:border-surface-700 overflow-auto"
        >
          <pre>{{ formattedOutput }}</pre>
        </div>
        <div v-else class="h-[600px] flex items-center justify-center border border-dashed border-surface-200 dark:border-surface-700 rounded-lg bg-surface-800">
          <span class="text-surface-400">Converted logs will appear here</span>
        </div>
        <div class="flex gap-4">
          <button
              @click="downloadLogs"
              class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
              :disabled="!convertedLogs"
          >
            Download JSON
          </button>
          <button
              @click="copyToClipboard"
              class="px-4 py-2 bg-surface-700 text-white rounded hover:bg-surface-600 disabled:opacity-50"
              :disabled="!convertedLogs"
          >
            Copy to Clipboard
          </button>
        </div>
      </div>
    </div>

    <div
        v-if="error"
        class="mt-4 p-4 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-100 rounded-lg"
    >
      {{ error }}
    </div>
  </div>
</template>

<script>
import { WasmLogContextTransformer } from '../deps/sologger-log-transformer-wasm/pkg/sologger_log_transformer_wasm.js';

const sanitizeLogMessage = (message) => {
  if (typeof message === 'object') {
    return JSON.stringify(message, (key, value) => {
      if (typeof value === 'string') {
        return value.replace(/[\n\r\t]/g, ' ').replace(/\s+/g, ' ');
      }
      return value;
    });
  }
  if (typeof message === 'string') {
    return message.replace(/[\n\r\t]/g, ' ').replace(/\s+/g, ' ');
  }
  return message;
};

export default {
  name: 'ConvertView',
  data() {
    return {
      inputLogs: '',
      convertedLogs: null,
      error: null,
      transformer: null
    };
  },
  computed: {
    formattedOutput() {
      return this.convertedLogs ? JSON.stringify(this.convertedLogs, null, 2) : '';
    }
  },
  mounted() {
    this.transformer = new WasmLogContextTransformer(["*"]);
  },
  methods: {
    handleInput() {
      this.error = null;
      this.convertedLogs = null;
    },

    convertLogs() {
      try {
        // Parse input as JSON if possible
        let inputData;
        try {
          inputData = JSON.parse(this.inputLogs);
        } catch {
          // If not JSON, treat as raw logs
          inputData = {
            logs: this.inputLogs.split('\n').filter(line => line.trim()),
            signature: 'unknown',
            slot: Date.now(),
            err: null
          };
        }

        const parsedLogs = this.transformer.from_rpc_logs_response({
          signature: inputData.signature || 'unknown',
          err: inputData.err || null,
          logs: inputData.logs || []
        }, BigInt(inputData.slot || Date.now()));

        let sanitizedLogs = parsedLogs.map(log => ({
          signature: sanitizeLogMessage(inputData.signature),
          slot: inputData.slot,
          solana: JSON.parse(sanitizeLogMessage(log))
        }));

        this.convertedLogs = {
          metadata: {
            convertedAt: new Date().toISOString(),
            totalLogs: sanitizedLogs.length
          },
          logs: sanitizedLogs
        };
      } catch (error) {
        this.error = `Error converting logs: ${error.message}`;
        console.error('Error details:', error);
      }
    },

    async copyToClipboard() {
      try {
        await navigator.clipboard.writeText(this.formattedOutput);
      } catch (error) {
        this.error = 'Failed to copy to clipboard';
      }
    },

    downloadLogs() {
      if (!this.convertedLogs) return;

      const blob = new Blob([this.formattedOutput], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `converted-solana-logs-${new Date().toISOString()}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    }
  }
};
</script>

<style scoped>
pre {
  white-space: pre-wrap;
  word-wrap: break-word;
}

textarea {
  resize: none;
}
</style>