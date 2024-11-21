<template>
  <div>
    <div class="container mx-auto p-4">
          <h1 class="text-2xl font-bold mb-6 text-surface-800 dark:text-surface-100">
            Solana Log Explorer
          </h1>
          <p class="mb-6 text-surface-600 dark:text-surface-300">
            Monitor and analyze Solana program logs in real-time across different networks.
          </p>
      <ProgramIdForm v-model="newProgramId" @addProgramId="addProgramId" />
      <div class="flex gap-4 mb-6">
        <select
            v-model="selectedEnvironment"
            @change="handleEnvironmentChange"
            class="px-4 py-2 border border-surface-200 rounded bg-surface-50 text-surface-800"
        >
          <option value="custom">Custom URL</option>
          <option v-for="env in environments" :key="env.key" :value="env.url">
            {{ env.key }}
          </option>
        </select>
        <input
            v-if="selectedEnvironment === 'custom'"
            v-model="customUrl"
            @change="handleCustomUrlChange"
            type="text"
            placeholder="Enter WebSocket URL (wss://...)"
            class="flex-1 px-4 py-2 border border-surface-200 rounded bg-surface-50 text-surface-800"
        />
        <button
            @click="disconnectWebSocket"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
        >
          Stop Logs
        </button>
        <button
            @click="startAllWebSockets"
            class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 flex items-center gap-2"
        >
          Start Logs
        </button>
        <button
            @click="clearAll"
            class="px-4 py-2 bg-yellow-500 text-white rounded hover:bg-yellow-600 flex items-center gap-2"
        >
          Clear All
        </button>
        <button
            @click="downloadLogs"
            class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 flex items-center gap-2"
            :disabled="!parsedLogs.length"
        >
          <span>Download JSON</span>
          <span v-if="!parsedLogs.length" class="text-sm opacity-75">(No data)</span>
        </button>
      </div>
      <div class="mb-6">
        <div class="flex flex-wrap gap-2">
          <div v-for="programId in programIds" :key="programId"
               class="flex items-center gap-2 bg-gray-100 px-3 py-1 rounded">
            <span>{{ programId }}</span>
            <div v-if="connectingWebsockets.has(programId)" class="flex items-center gap-2">
              <span class="text-primary-500">Connecting...</span>
              <svg class="animate-spin h-4 w-4 text-primary-500"
                   xmlns="http://www.w3.org/2000/svg"
                   fill="none"
                   viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>
            <span v-else-if="websockets.has(programId)" class="text-green-500">Connected</span>
            <span v-else class="text-gray-500">Disconnected</span>
          </div>
        </div>
      </div>
      <ProgramList :programIds="programIds" @removeProgramId="removeProgramId"/>
      <StatsGrid
          :parsedLogs="parsedLogs"
          :uniqueProgramsCount="uniqueProgramsCount"
          :lastUpdateTime="lastUpdateTime"
      />
      <div class="flex items-center gap-2 mb-4">
        <input
            type="checkbox"
            id="errorFilter"
            v-model="onlyShowErrors"
            class="w-4 h-4 text-primary-400 bg-surface-50 border-surface-300 rounded focus:ring-primary-500"
        />
        <label for="errorFilter" class="text-surface-800 dark:text-surface-100">
          Only show error logs
        </label>
      </div>
      <LogsTable
          :parsedLogs="parsedLogs"
          :hotSettings="hotSettings"
      />
    </div>
  </div>
</template>

<script>
// Import your existing App.vue script here and rename the component
import { onMounted } from 'vue';
import { registerAllModules } from 'handsontable/registry';
import init, { WasmLogContextTransformer } from '../deps/sologger-log-transformer-wasm/pkg/sologger_log_transformer_wasm.js';
import ProgramIdForm from '../components/ProgramIdForm.vue';
import ProgramList from '../components/ProgramList.vue';
import StatsGrid from '../components/StatsGrid.vue';
import LogsTable from '../components/LogsTable.vue';
import Button from "primevue/button";

registerAllModules();

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
  name: 'App',
  components: {
    Button,
    ProgramIdForm,
    ProgramList,
    StatsGrid,
    LogsTable
  },
  setup() {
    onMounted(async () => {
      await init();
      // const transformer = new WasmLogContextTransformer(["*"]);
      console.log("WASM Initialized");
      // transformer.from_rpc_response({});
    });

    return {};
  },
  data() {
    return {
      websockets: new Map(), // programId -> WebSocket
      connectingWebsockets: new Set(), // Track connecting state
      newProgramId: '',
      programIds: [],
      environments: [
        { key: 'Devnet', url: 'wss://api.devnet.solana.com' },
        { key: 'Testnet', url: 'wss://api.testnet.solana.com' }
      ],
      customUrl: '',
      selectedEnvironment: 'wss://api.devnet.solana.com',
      onlyShowErrors: false,
      parsedLogs: [],
      lastUpdateTime: '-',
      hotSettings: {
        columns: [
          { data: 'timestamp', title: 'Time', width: 100, type: 'text' },
          { data: 'level', title: 'Level', width: 80, type: 'text' },
          { data: 'signature', title: 'Signature', width: 200, type: 'text' },
          { data: 'slot', title: 'Slot', width: 100, type: 'numeric' },
          { data: 'programId', title: 'Program ID', width: 150, type: 'text' },
          { data: 'parentProgramId', title: 'Parent Program', width: 150, type: 'text' },
          { data: 'depth', title: 'Depth', width: 80, type: 'numeric' },
          { data: 'instructionIndex', title: 'Index', width: 80, type: 'numeric' },
          { data: 'invokeResult', title: 'Result', width: 100, type: 'text' },
          { data: 'logMessages', title: 'Log Messages', width: 300, type: 'text' },
          { data: 'rawLogs', title: 'Raw Logs', width: 300, type: 'text' },
          { data: 'dataLogs', title: 'Data Logs', width: 200, type: 'text' },
          { data: 'errors', title: 'Errors', width: 200, type: 'text' },
          { data: 'transactionError', title: 'TX Error', width: 200, type: 'text' }
        ],
        licenseKey: 'non-commercial-and-evaluation',
        columnSorting: true,
        filters: true,
        dropdownMenu: true,
        colHeaders: true,
        rowHeaders: true,
        width: '100%',
        manualRowResize: true,
        manualColumnResize: true,
        autoWrapRow: true,
        autoWrapCol: true,
        nestedRows: false,
        contextMenu: true,
        readOnly: true
      }
    };
  },
  computed: {
    uniqueProgramsCount() {
      const programs = new Set(this.parsedLogs.map(row => row.programId));
      return programs.size;
    }
  },
  methods: {
    parseLog(logData) {
      return {
        timestamp: new Date().toLocaleTimeString(),
        level: logData.solana.transaction_error !== null && logData.solana.transaction_error !== "" ? "Error" : "Info",
        signature: logData.signature,
        slot: logData.slot,
        programId: logData.solana.program_id,
        parentProgramId: logData.solana.parent_program_id,
        depth: logData.solana.depth,
        instructionIndex: logData.solana.instruction_index,
        invokeResult: logData.solana.invoke_result,
        logMessages: JSON.stringify(logData.solana.log_messages),
        dataLogs: JSON.stringify(logData.solana.data_logs),
        rawLogs: JSON.stringify(logData.solana.raw_logs),
        errors: JSON.stringify(logData.solana.errors),
        transactionError: logData.solana.transaction_error || ''
      };
    },
    extractProgramId(logEntry) {
      const match = logEntry.match(/Program (\w+) invoke/);
      return match ? match[1] : logEntry.match(/Program (\w+) success/)?.[1] ?? null;
    },
    determineStatus(logEntry) {
      if (logEntry.includes('success')) return 'Success';
      if (logEntry.includes('failed')) return 'Failed';
      if (logEntry.includes('invoke')) return 'Started';
      return 'Processing';
    },
    async addProgramId() {
      if (this.newProgramId && !this.programIds.includes(this.newProgramId)) {
        this.programIds.push(this.newProgramId);
        await this.connectWebSocketForProgram(this.newProgramId);
        this.newProgramId = '';
      }
    },

    async removeProgramId(index) {
      const programId = this.programIds[index];
      await this.disconnectWebSocketForProgram(programId);
      this.programIds.splice(index, 1);
    },
    handleCustomUrlChange() {
      if (this.customUrl && this.customUrl.startsWith('wss://')) {
        this.handleEnvironmentChange();
      } else {
        alert('Please enter a valid WebSocket URL starting with "wss://"');
        this.customUrl = '';
        this.selectedEnvironment = 'wss://api.devnet.solana.com';
      }
    },
    async handleEnvironmentChange() {
      const url = this.selectedEnvironment === 'custom' ? this.customUrl : this.selectedEnvironment;
      console.log(`Switching to environment: ${url}`);

      if (this.websockets.size > 0) {
        await this.reconnectToNewEnvironment();
      }
    },

    getEnvironmentName() {
      if (this.selectedEnvironment === 'custom') {
        return 'Custom';
      }
      return this.environments.find(env => env.url === this.selectedEnvironment)?.key || 'Unknown';
    },

    async reconnectToNewEnvironment() {
      // Store current program IDs
      const currentPrograms = [...this.programIds];

      // Clear everything
      await this.clearAll();

      // Restore program IDs
      this.programIds = currentPrograms;

      // Reconnect all websockets to new environment
      await this.startAllWebSockets();
    },
    updateTable(eventData) {

      if (eventData.method === 'logsNotification') {
        const transformer = new WasmLogContextTransformer(["*"])

        const logs = eventData.params.result.value.logs;
        const slot = eventData.params.result.context.slot;
        const signature = eventData.params.result.value.signature;
        const err = eventData.params.result.value.err === null ? null : eventData.params.result.value.err;

        try {
          const parsedLogs = transformer.from_rpc_logs_response({
            signature,
            err,
            logs
          }, BigInt(slot));

          let sanitizedLogs = [];
          parsedLogs.forEach((solana_log) => {
            const sanitizedLog = {
              signature: sanitizeLogMessage(signature),
              slot,
              solana: JSON.parse(sanitizeLogMessage(solana_log))
            };

            if(sanitizedLog.solana.transaction_error !== null && sanitizedLog.solana.transaction_error !== "") {
              // console.log('Dev Solana logs', JSON.stringify(sanitizedLog));
            } else {
              // console.log('Dev Solana logs', JSON.stringify(sanitizedLog));
            }

            if (!this.onlyShowErrors || (sanitizedLog.solana.transaction_error !== null && sanitizedLog.solana.transaction_error !== "")) {
              sanitizedLogs.push(sanitizedLog);
            }

          });

          try {
            const newParsedLogs = sanitizedLogs.map(log => this.parseLog(log));
            this.parsedLogs.unshift(...newParsedLogs);
            this.parsedLogs = this.parsedLogs.slice(0, 1000);
            this.lastUpdateTime = new Date().toLocaleTimeString();
          } catch (error) {
            console.error('Error parsing logs:', error);
          }

        } catch (error) {
          console.log('Error parsing logs', {
            error: sanitizeLogMessage(error.message),
            signature: sanitizeLogMessage(signature),
            slot
          });
        }
      }

      if (this.$refs.hotTable) {
        this.$refs.hotTable.hotInstance.render();
      }
    },
    async startAllWebSockets() {
      console.log('Starting WebSockets for all programs');
      try {
        // First disconnect any existing connections
        await this.disconnectWebSocket();

        // Start new connections for all program IDs
        const connectionPromises = this.programIds.map(programId =>
            this.connectWebSocketForProgram(programId)
        );

        await Promise.all(connectionPromises);
        console.log('All WebSocket connections established');
      } catch (error) {
        console.error('Error starting WebSocket connections:', error);
      }
    },

    async connectWebSocketForProgram(programId) {
      if (this.websockets.has(programId)) {
        console.log(`WebSocket already exists for program: ${programId}`);
        return;
      }

      const url = this.selectedEnvironment === 'custom' ? this.customUrl : this.selectedEnvironment;
      console.log(`Connecting WebSocket for program: ${programId} on ${url}`);
      this.connectingWebsockets.add(programId);

      return new Promise((resolve, reject) => {
        const ws = new WebSocket(url);
        const setupMessageHandler = () => {
          let hasReceivedMessage = false;

          return (event) => {
            const eventData = JSON.parse(event.data);

            // Check if this is the subscription confirmation message
            if (eventData.result !== undefined && !hasReceivedMessage) {
              hasReceivedMessage = true;
              this.connectingWebsockets.delete(programId);
              console.log(`Received first message for program: ${programId}`);
            }

            if (eventData.params?.result?.value) {
              this.lastUpdateTime = new Date().toLocaleTimeString();
              this.updateTable(eventData);
            }
          };
        };

        ws.onopen = () => {
          const subscribeMessage = {
            jsonrpc: '2.0',
            id: Date.now(),
            method: 'logsSubscribe',
            params: [
              { mentions: [programId] },
              { commitment: 'finalized', encoding: 'json' }
            ]
          };
          ws.send(JSON.stringify(subscribeMessage));
          console.log(`WebSocket connected and subscribed for program: ${programId}`);
          this.websockets.set(programId, ws);
          resolve();
        };

        ws.onmessage = setupMessageHandler();

        ws.onerror = (error) => {
          console.error(`WebSocket error for program ${programId}:`, error);
          this.websockets.delete(programId);
          this.connectingWebsockets.delete(programId);
          reject(error);
        };

        ws.onclose = () => {
          console.log(`WebSocket connection closed for program: ${programId}`);
          this.websockets.delete(programId);
          this.connectingWebsockets.delete(programId);
        };
      });
    },

    async disconnectWebSocket() {
      console.log('Disconnecting all WebSockets');
      const closePromises = Array.from(this.websockets.keys()).map(programId =>
          this.disconnectWebSocketForProgram(programId)
      );
      await Promise.all(closePromises);
      this.websockets.clear();
      console.log('All WebSocket connections closed');
    },

    async disconnectWebSocketForProgram(programId) {
      const ws = this.websockets.get(programId);
      if (ws) {
        await new Promise(resolve => {
          ws.onclose = () => {
            console.log(`WebSocket disconnected for program: ${programId}`);
            this.connectingWebsockets.delete(programId);
            resolve();
          };
          ws.close();
          this.websockets.delete(programId);
        });
      }
    },
    async clearAll() {
      await this.disconnectWebSocket();
      this.parsedLogs = [];
      this.programIds = [];
      this.lastUpdateTime = '-';
      this.websockets.clear();
      this.connectingWebsockets.clear();
      console.log('Cleared all data and connections');

      if (this.$refs.hotTable) {
        this.$refs.hotTable.hotInstance.render();
      }
    },

    downloadLogs() {
      try {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const environment = this.getEnvironmentName();
        const filename = `solana-logs-${environment}-${timestamp}.json`;

        const downloadData = {
          metadata: {
            exportedAt: new Date().toISOString(),
            environment: environment,
            url: this.selectedEnvironment === 'custom' ? this.customUrl : this.selectedEnvironment,
            programIds: this.programIds,
            totalLogs: this.parsedLogs.length
          },
          logs: this.parsedLogs
        };

        // Convert to JSON string
        const jsonString = JSON.stringify(downloadData, null, 2);

        // Create blob and download
        const blob = new Blob([jsonString], { type: 'application/json' });
        const url = URL.createObjectURL(blob);

        // Create download link
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;

        // Trigger download
        document.body.appendChild(link);
        link.click();

        // Cleanup
        document.body.removeChild(link);
        URL.revokeObjectURL(url);

        console.log(`Downloaded ${downloadData.logs.length} logs to ${filename}`);
      } catch (error) {
        console.error('Error downloading logs:', error);
        alert('Error downloading logs. Check console for details.');
      }
    },

    watch: {
      selectedEnvironment(newValue) {
        console.log(`Environment changed to: ${newValue}`);
      }
    }

    // Remove the connectWebSocket and reconnectWebSocket methods as they're no longer needed
  },
  beforeUnmount() {
    this.disconnectWebSocket();
  }
};
</script>
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