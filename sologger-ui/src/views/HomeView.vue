<template>
  <div>
    <div class="container mx-auto p-4">
      <h1 class="text-2xl font-bold mb-6 text-[var(--p-text-color)]">
        Solana Log Explorer
      </h1>
      <p class="mb-6 text-[var(--p-text-color)]">
        Monitor and analyze Solana program logs in real-time across different networks.
      </p>

      <!-- Program ID Form -->
      <ProgramIdForm v-model="newProgramId" @addProgramId="addProgramId" class="mb-4"/>

      <!-- Controls Section -->
      <div class="space-y-4 mb-6">
        <!-- Environment Selection -->
        <div class="flex flex-col md:flex-row gap-4">
          <select
              v-model="selectedEnvironment"
              @change="handleEnvironmentChange"
              class="px-4 py-2 border border-surface-200 rounded bg-surface-50 text-surface-800 w-full md:w-auto"
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
        </div>

        <!-- Action Buttons -->
        <div class="grid grid-cols-2 md:flex gap-2 md:gap-4">
          <button
              @click="disconnectWebSocket"
              class="px-3 py-2 md:px-4 bg-red-500 text-white rounded hover:bg-red-600 text-sm md:text-base"
          >
            Stop Logs
          </button>
          <button
              @click="startAllWebSockets"
              class="px-3 py-2 md:px-4 bg-green-500 text-white rounded hover:bg-green-600 text-sm md:text-base flex items-center justify-center gap-2"
          >
            Start Logs
          </button>
          <button
              @click="clearAll"
              class="px-3 py-2 md:px-4 bg-yellow-500 text-white rounded hover:bg-yellow-600 text-sm md:text-base flex items-center justify-center gap-2"
          >
            Clear All
          </button>
          <button
              @click="downloadLogs"
              class="px-3 py-2 md:px-4 bg-blue-500 text-white rounded hover:bg-blue-600 text-sm md:text-base flex items-center justify-center gap-2"
              :disabled="!parsedLogs.length"
          >
            <span>Download</span>
            <span v-if="!parsedLogs.length" class="text-xs md:text-sm opacity-75">(No data)</span>
          </button>
        </div>
      </div>

      <!-- Program Status Chips -->
      <div class="mb-6 overflow-x-auto">
        <div class="flex flex-wrap gap-2 min-w-min">
          <div v-for="programId in programIds" :key="programId"
               class="flex items-center gap-2 bg-gray-100 px-3 py-1 rounded text-sm">
            <span class="truncate max-w-[150px] md:max-w-none">{{ programId }}</span>
            <div v-if="connectingWebsockets.has(programId)" class="flex items-center gap-2">
              <span class="text-primary-500 text-xs md:text-sm">Connecting...</span>
              <svg class="animate-spin h-4 w-4 text-primary-500"
                   xmlns="http://www.w3.org/2000/svg"
                   fill="none"
                   viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>
            <span v-else-if="websockets.has(programId) && receivingMessages" class="text-green-500 text-xs md:text-sm">Connected</span>
            <span v-else-if="websockets.has(programId) && !receivingMessages" class="text-green-500 text-xs md:text-sm">Awaiting logs...May take several seconds</span>
            <span v-else class="text-gray-500 text-xs md:text-sm">Disconnected</span>
          </div>
        </div>
      </div>

      <!-- Program List and Stats -->
      <div class="space-y-4 mb-6">
        <ProgramList :programIds="programIds" @removeProgramId="removeProgramId"/>
        <StatsGrid
            :parsedLogs="parsedLogs"
            :uniqueProgramsCount="uniqueProgramsCount"
            :lastUpdateTime="lastUpdateTime"
        />
      </div>

      <!-- Error Filter -->
      <div class="flex items-center gap-2 mb-4">
        <input
            type="checkbox"
            id="errorFilter"
            v-model="onlyShowErrors"
            class="w-4 h-4 text-primary-400 bg-surface-50 border-surface-300 rounded focus:ring-primary-500"
        />
        <label for="errorFilter" class="text-[var(--p-text-color)] text-sm md:text-base">
          Only show error logs
        </label>
      </div>

      <!-- Logs Table with Mobile Optimization -->
      <div class="overflow-x-auto">
        <LogsTable
            :parsedLogs="parsedLogs"
            :hotSettings="getMobileOptimizedHotSettings"
        />
      </div>
    </div>
  </div>
</template>
<script>
// Import your existing script here
import {onMounted} from 'vue';
import {registerAllModules} from 'handsontable/registry';
import init, {
  WasmLogContextTransformer
} from '../../public/sologger-log-transformer-wasm/pkg/sologger_log_transformer_wasm.js';
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
      console.log("WASM Initialized");
    });

    return {};
  },
  data() {
    return {
      // ... your existing data properties ...
      websockets: new Map(),
      connectingWebsockets: new Set(),
      receivingMessages: false,
      newProgramId: '',
      programIds: [],
      environments: [
        {key: 'Devnet', url: 'wss://api.devnet.solana.com'},
        {key: 'Testnet', url: 'wss://api.testnet.solana.com'}
      ],
      customUrl: '',
      selectedEnvironment: 'wss://api.devnet.solana.com',
      onlyShowErrors: false,
      parsedLogs: [],
      lastUpdateTime: '-',
      hotSettings: {
        columns: [
          {data: 'timestamp', title: 'Time', width: 100, type: 'text'},
          {data: 'level', title: 'Level', width: 80, type: 'text'},
          {
            data: 'signature', title: 'Signature', width: 200,
            renderer: function (instance, td, row, col, prop, value, cellProperties) {
              const link = document.createElement('a');
              link.href = `https://solscan.io/tx/${value.signature}${value.linkSuffix}`;
              link.target = '_blank';
              link.textContent = value.signature;
              td.innerHTML = '';
              td.appendChild(link);
              return td;
            }
          },
          {
            data: 'slot',
            title: 'Slot',
            width: 100,
            renderer: function (instance, td, row, col, prop, value, cellProperties) {
              const link = document.createElement('a');
              link.href = `https://solscan.io/block/${value.slot}${value.linkSuffix}`;
              link.target = '_blank';
              link.textContent = value.slot;
              td.innerHTML = '';
              td.appendChild(link);
              return td;
            }
          },
          {
            data: 'programId',
            title: 'Program ID',
            width: 150,
            renderer: function (instance, td, row, col, prop, value, cellProperties) {
              const link = document.createElement('a');
              link.href = `https://solscan.io/account/${value.programId}${value.linkSuffix}`;
              link.target = '_blank';
              link.textContent = value.programId;
              td.innerHTML = '';
              td.appendChild(link);
              return td;
            }
          },
          {
            data: 'parentProgramId', title: 'Parent Program', width: 150,
            renderer: function (instance, td, row, col, prop, value, cellProperties) {
              const link = document.createElement('a');
              link.href = `https://solscan.io/account/${value.parentProgramId}${value.linkSuffix}`;
              link.target = '_blank';
              link.textContent = value.parentProgramId;
              td.innerHTML = '';
              td.appendChild(link);
              return td;
            }
          },
          {data: 'depth', title: 'Depth', width: 80, type: 'numeric'},
          {data: 'instructionIndex', title: 'Index', width: 80, type: 'numeric'},
          {data: 'invokeResult', title: 'Result', width: 100, type: 'text'},
          {data: 'logMessages', title: 'Log Messages', width: 300, type: 'text'},
          {data: 'rawLogs', title: 'Raw Logs', width: 300, type: 'text'},
          {data: 'dataLogs', title: 'Data Logs', width: 200, type: 'text'},
          {data: 'errors', title: 'Errors', width: 200, type: 'text'},
          {data: 'transactionError', title: 'TX Error', width: 200, type: 'text'}
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
    },
    getMobileOptimizedHotSettings() {
      const baseSettings = {...this.hotSettings};

      // Adjust column widths for mobile
      baseSettings.columns = baseSettings.columns.map(column => {
        const newColumn = {...column};
        if (window.innerWidth < 768) {
          // Reduce column widths on mobile
          newColumn.width = Math.min(column.width, 120);

          // Truncate long text fields
          if (['logMessages', 'rawLogs', 'dataLogs', 'errors'].includes(column.data)) {
            newColumn.width = 150;
          }
        }
        return newColumn;
      });

      // Additional mobile optimizations
      if (window.innerWidth < 768) {
        baseSettings.colHeaders = true;
        baseSettings.rowHeaders = false; // Hide row headers on mobile
        baseSettings.contextMenu = false; // Disable context menu on mobile
        baseSettings.dropdownMenu = false; // Disable dropdown menu on mobile
      }

      return baseSettings;
    }
  },
  methods: {
    parseLog(logData) {
      let linkSuffix = '';
      if (this.selectedEnvironment.includes('dev')) {
        linkSuffix = '?cluster=devnet';
      } else if (this.selectedEnvironment.includes('test')) {
        linkSuffix = '?cluster=testnet';
      }
      let signatureData = {signature: logData.signature, linkSuffix: linkSuffix};
      let slotData = {slot: logData.slot, linkSuffix: linkSuffix};
      let programData = {programId: logData.solana.program_id, linkSuffix: linkSuffix};
      let parentProgramData = {parentProgramId: logData.solana.parent_program_id, linkSuffix: linkSuffix};
      return {
        timestamp: new Date().toLocaleTimeString(),
        level: logData.solana.transaction_error !== null && logData.solana.transaction_error !== "" ? "Error" : "Info",
        signature: signatureData,
        slot: slotData,
        programId: programData,
        parentProgramId: parentProgramData,
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

            if (sanitizedLog.solana.transaction_error !== null && sanitizedLog.solana.transaction_error !== "") {
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
              this.receivingMessages = true;
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
              {mentions: [programId]},
              {commitment: 'finalized', encoding: 'json'}
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
        const blob = new Blob([jsonString], {type: 'application/json'});
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
:root {
  --p-text-color: var(--p-surface-800);
}

:root[data-theme="dark"] {
  --p-text-color: var(--p-surface-50);
}

/* Keep your existing styles */
.router-link-active {
  color: var(--p-primary-400);
  font-weight: 500;
}

nav {
  border-bottom: 1px solid var(--p-surface-700);
}

/* Mobile optimizations */
@media (max-width: 768px) {
  .container {
    padding-left: 0.5rem;
    padding-right: 0.5rem;
  }

  /* Optimize table for mobile */
  :deep(.handsontable) {
    font-size: 12px;
  }

  :deep(.handsontable td) {
    padding: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}
</style>