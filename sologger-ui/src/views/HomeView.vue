<template>
  <div>
    <div class="container mx-auto px-4 py-6">
      <div class="mb-8">
        <h1 class="text-3xl font-bold mb-2 text-[var(--p-text-color)] tracking-tight">
          Solana Log Explorer
        </h1>
        <p class="text-[var(--p-text-muted)] text-base">
          Monitor and analyze Solana program logs in real-time across different networks.
        </p>
      </div>

      <!-- Mainnet Warning Modal -->
      <div v-if="showMainnetWarning" class="modal-overlay" @click.self="cancelMainnet">
        <div class="modal-panel">
          <div class="modal-header">
            <span class="font-semibold text-base">⚠️ Mainnet Warning</span>
            <button @click="cancelMainnet" class="btn btn-secondary" aria-label="Close">✕</button>
          </div>
          <div class="modal-body">
            <p class="text-[var(--p-text-color)] mb-3">
              Monitoring Mainnet logs is resource-intensive. Public RPC endpoints have strict rate limits and may drop
              connections frequently.
              We strongly recommend using a <strong>private RPC provider</strong>.
            </p>
            <p class="text-[var(--p-text-muted)] text-sm mb-4">Popular providers with free tiers:</p>
            <div class="flex flex-col gap-2 mb-4">
              <a href="https://helius.dev" target="_blank" class="rpc-provider-link">
                <span class="font-semibold">Helius</span>
                <span class="text-xs text-[var(--p-text-muted)]">helius.dev — Solana-native, generous free tier</span>
              </a>
              <a href="https://quicknode.com" target="_blank" class="rpc-provider-link">
                <span class="font-semibold">QuickNode</span>
                <span class="text-xs text-[var(--p-text-muted)]">quicknode.com — Multi-chain, fast global nodes</span>
              </a>
              <a href="https://ironforge.cloud" target="_blank" class="rpc-provider-link">
                <span class="font-semibold">Ironforge</span>
                <span class="text-xs text-[var(--p-text-muted)]">ironforge.cloud — Solana-focused infrastructure</span>
              </a>
            </div>
            <p class="text-[var(--p-text-muted)] text-xs mb-4">
              To use a private RPC, select <strong>Custom URL</strong> and enter your WebSocket endpoint (wss://...).
            </p>
            <div class="flex gap-2 justify-end">
              <button @click="cancelMainnet" class="btn btn-secondary">Cancel</button>
              <button @click="confirmMainnet" class="btn btn-warning">Continue with Public Mainnet</button>
            </div>
          </div>
        </div>
      </div>

      <!-- IDL Decoded Data Modal -->
      <div v-if="idlDecodedData" class="modal-overlay" @click.self="idlDecodedData = null">
        <div class="modal-panel modal-panel--wide">
          <div class="modal-header">
            <span class="font-semibold text-base">🔍 IDL Decoded Instruction Data</span>
            <button @click="idlDecodedData = null" class="btn btn-secondary">✕ Close</button>
          </div>
          <div class="modal-body modal-body--rows">
            <template v-if="parsedIdlDecodedData">
              <div v-for="(value, key) in parsedIdlDecodedData" :key="key" class="detail-row">
                <span class="detail-label">{{ key }}</span>
                <span class="detail-value">{{ formatIdlDetailValue(value) }}</span>
              </div>
            </template>
            <pre v-else class="idl-decoded-pre">{{ idlDecodedData }}</pre>
          </div>
        </div>
      </div>

      <!-- Program ID Form -->
      <ProgramIdForm v-model="newProgramId" @addProgramId="addProgramId" class="mb-4"/>

      <!-- Controls Section -->
      <div class="space-y-4 mb-6">
        <!-- Explorer & Environment Selection -->
        <div class="flex flex-col md:flex-row gap-4">
          <select v-model="selectedExplorer" class="input-base w-full md:w-auto">
            <option value="solscan">Solscan</option>
            <option value="solana">Solana Explorer</option>
            <option value="orb">Orb</option>
          </select>
          <select
              v-model="selectedEnvironment"
              @change="handleEnvironmentChange"
              class="input-base w-full md:w-auto"
          >
            <option value="custom">Custom URL</option>
            <option v-for="env in environments" :key="env.key" :value="env.url">
              {{ env.key }}
            </option>
          </select>

          <div v-if="selectedEnvironment === 'custom'" class="flex flex-1 gap-1">
            <input
                v-if="!maskApiKey || !customUrlHasApiKey"
                v-model="customUrl"
                @change="handleCustomUrlChange"
                type="text"
                placeholder="Enter WebSocket URL (wss://...)"
                class="input-base flex-1"
            />
            <input
                v-else
                :value="maskedCustomUrl"
                @focus="maskApiKey = false"
                type="text"
                placeholder="Enter WebSocket URL (wss://...)"
                class="input-base flex-1"
                readonly
            />
            <button
                v-if="customUrlHasApiKey"
                @click="maskApiKey = !maskApiKey"
                class="btn btn-secondary"
                :title="maskApiKey ? 'Show API key' : 'Hide API key'"
                type="button"
            >{{ maskApiKey ? '👁' : '🙈' }}
            </button>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex flex-wrap items-center gap-x-6 gap-y-3">
          <div class="flex flex-wrap gap-2">
            <button
                @click="startAllWebSockets"
                class="btn btn-primary"
            >
              Connect
            </button>
            <button
                @click="disconnectWebSocket"
                class="btn btn-secondary"
            >
              Disconnect
            </button>
            <button
                @click="togglePause"
                :class="isPaused ? 'btn btn-warning' : 'btn btn-secondary'"
            >
              {{ isPaused ? 'Resume' : 'Pause' }}
            </button>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <button
                @click="clearLogs"
                class="btn btn-secondary"
            >
              Clear Logs
            </button>
            <div class="flex items-center gap-1">
              <label for="max-logs" class="text-xs text-[var(--p-text-muted)] whitespace-nowrap">Max Logs:</label>
              <input
                  id="max-logs"
                  v-model.number="maxLogs"
                  type="number"
                  min="100"
                  step="100"
                  class="input-base w-24 text-sm"
              />
            </div>
          </div>
          <div class="flex flex-wrap gap-2">
            <button
                @click="exportJSON"
                class="btn btn-secondary"
                :disabled="!parsedLogs.length"
            >
              Export JSON
            </button>
            <button
                @click="exportCSV"
                class="btn btn-secondary"
                :disabled="!parsedLogs.length"
            >
              Export CSV
            </button>
          </div>
          <button
              @click="clearAll"
              class="btn btn-danger md:ml-auto"
          >
            Clear All
          </button>
        </div>
      </div>

      <!-- Monitored Programs (status + remove) -->
      <ProgramList :programs="programStatuses" @removeProgramId="removeProgramId"/>

      <!-- Stats & Charts -->
      <CollapsibleSection title="Stats & Charts" storage-key="stats" bare class="mb-6">
        <StatsGrid
            :parsedLogs="parsedLogs"
            :uniqueSignaturesCount="uniqueSignaturesCount"
            :lastUpdateTime="lastUpdateTime"
        />
      </CollapsibleSection>

      <!-- IDL Upload for WASM Decoding -->
      <CollapsibleSection
          title="IDL Decoding"
          storage-key="idl"
          :default-open="false"
          :badge="uploadedIdl ? 'Loaded: ' + idlFileName : ''"
          class="mb-4"
      >
        <div class="flex flex-col md:flex-row gap-2 items-start md:items-center">
          <label class="btn btn-secondary cursor-pointer">
            📂 {{ uploadedIdl ? '✅ IDL Loaded: ' + idlFileName : 'Upload IDL (JSON)' }}
            <input type="file" accept=".json" class="hidden" @change="handleIdlUpload"/>
          </label>
          <button
              v-if="uploadedIdl"
              @click="uploadedIdl = null; idlFileName = ''"
              class="btn btn-danger"
          >Remove IDL
          </button>
          <span v-if="uploadedIdl" class="text-xs text-[var(--p-text-muted)]">
            Select a log row in the table to decode its data logs using the uploaded IDL.
          </span>
        </div>
      </CollapsibleSection>

      <!-- Log Replay Tool -->
      <CollapsibleSection title="Log Replay" storage-key="replay" :default-open="false" class="mb-4">
        <div class="flex flex-col md:flex-row gap-2">
          <input
              v-model="replaySignature"
              type="text"
              placeholder="Paste a transaction signature to fetch & replay logs..."
              class="input-base flex-1"
          />
          <button
              @click="replayTransaction"
              class="btn btn-secondary"
              :disabled="replayLoading || !replaySignature.trim()"
          >
            {{ replayLoading ? 'Fetching...' : 'Replay' }}
          </button>
        </div>
        <p v-if="replayError" class="text-red-500 text-xs mt-1">{{ replayError }}</p>
      </CollapsibleSection>

      <!-- Global Search and Filter Bar -->
      <div class="flex flex-col md:flex-row gap-2 mb-4">
        <input
            v-model="searchQuery"
            type="text"
            placeholder="Search logs (supports regex)..."
            class="input-base flex-1"
        />
        <select v-model="filterLevel" class="input-base w-full md:w-40">
          <option value="">All Levels</option>
          <option value="Info">Info</option>
          <option value="Error">Error</option>
        </select>
        <input
            v-model="filterInstruction"
            type="text"
            placeholder="Filter by instruction name..."
            class="input-base w-full md:w-56"
        />
      </div>

      <!-- Logs Table with Mobile Optimization -->
      <div class="overflow-x-auto">
        <LogsTable
            :parsedLogs="filteredLogs"
            :uploadedIdl="uploadedIdl"
            @decode-with-idl="decodeWithIdl"
        />
      </div>
    </div>
  </div>
</template>
<script>
import {onMounted} from 'vue';
import {useToast} from 'primevue/usetoast';
import init, {
  WasmLogContextTransformer
} from '../../public/sologger-log-transformer-wasm/pkg/sologger_log_transformer_wasm.js';
import {decodeWithIdl as decodeLogWithIdl} from '../composables/useIdlDecoder';
import {sanitizeLogMessage} from '../composables/useLogSanitizer';
import ProgramIdForm from '../components/ProgramIdForm.vue';
import ProgramList from '../components/ProgramList.vue';
import StatsGrid from '../components/StatsGrid.vue';
import LogsTable from '../components/LogsTable.vue';
import CollapsibleSection from '../components/CollapsibleSection.vue';

export default {
  name: 'App',
  components: {
    ProgramIdForm,
    ProgramList,
    StatsGrid,
    LogsTable,
    CollapsibleSection
  },
  setup() {
    const toast = useToast();
    onMounted(async () => {
      await init();
      console.log("WASM Initialized");
    });

    return {toast};
  },
  data() {
    return {
      websockets: new Map(),
      connectingWebsockets: new Set(),
      receivingMessages: false,
      newProgramId: '',
      programIds: [],
      isPaused: false,
      selectedExplorer: 'solscan',
      replaySignature: '',
      replayLoading: false,
      replayError: '',
      searchQuery: '',
      filterLevel: '',
      filterInstruction: '',
      environments: [
        {key: 'Devnet', url: 'wss://api.devnet.solana.com'},
        {key: 'Testnet', url: 'wss://api.testnet.solana.com'},
        {key: 'Mainnet', url: 'wss://api.mainnet-beta.solana.com'}
      ],
      showMainnetWarning: false,
      pendingMainnetUrl: null,
      uploadedIdl: null,
      idlDecodedData: null,
      idlFileName: '',
      customUrl: '',
      maskApiKey: true,
      selectedEnvironment: 'wss://api.devnet.solana.com',
      maxLogs: 1000,
      parsedLogs: [],
      lastUpdateTime: '-'
    };
  },
  computed: {
    programStatuses() {
      return this.programIds.map(id => {
        let status;
        if (this.connectingWebsockets.has(id)) status = 'connecting';
        else if (this.websockets.has(id)) status = this.receivingMessages ? 'connected' : 'awaiting';
        else status = 'disconnected';
        return {id, status};
      });
    },
    filteredLogs() {
      let logs = this.parsedLogs;
      if (this.filterLevel) {
        logs = logs.filter(log => log.level === this.filterLevel);
      }
      if (this.filterInstruction) {
        const instr = this.filterInstruction.toLowerCase();
        logs = logs.filter(log =>
            (log.logMessages && log.logMessages.toLowerCase().includes(instr)) ||
            (log.rawLogs && log.rawLogs.toLowerCase().includes(instr))
        );
      }
      if (this.searchQuery) {
        try {
          const regex = new RegExp(this.searchQuery, 'i');
          logs = logs.filter(log =>
              Object.values(log).some(val => val && regex.test(String(val)))
          );
        } catch {
          const q = this.searchQuery.toLowerCase();
          logs = logs.filter(log =>
              Object.values(log).some(val => val && String(val).toLowerCase().includes(q))
          );
        }
      }
      return logs;
    },
    parsedIdlDecodedData() {
      if (!this.idlDecodedData) return null;
      try {
        return JSON.parse(this.idlDecodedData);
      } catch {
        return null;
      }
    },
    uniqueSignaturesCount() {
      const signatures = new Set(this.parsedLogs.map(row => row.signature?.signature ?? row.signature));
      return signatures.size;
    },
    customUrlHasApiKey() {
      return /[?&]api-key=/i.test(this.customUrl);
    },
    maskedCustomUrl() {
      return this.customUrl.replace(/([\?&]api-key=)([^&]+)/i, (_, prefix, key) => {
        return prefix + key.substring(0, 4) + '••••••••' + key.substring(key.length - 4);
      });
    }
  },
  methods: {
    // One transformer for the component's lifetime; kept off `data` so Vue doesn't proxy the WASM object.
    getTransformer() {
      if (!this._transformer) {
        this._transformer = new WasmLogContextTransformer(['*']);
      }
      return this._transformer;
    },
    formatIdlDetailValue(value) {
      if (value === null || value === undefined) return '';
      if (Array.isArray(value)) {
        return value.map(v => typeof v === 'object' ? JSON.stringify(v, null, 2) : String(v)).join('\n');
      }
      if (typeof value === 'object') return JSON.stringify(value, null, 2);
      return String(value);
    },
    parseLog(logData) {
      const isDevnet = this.selectedEnvironment.includes('dev');
      const isTestnet = this.selectedEnvironment.includes('test');
      let linkSuffix = '';
      if (isDevnet) linkSuffix = '?cluster=devnet';
      else if (isTestnet) linkSuffix = '?cluster=testnet';
      let signatureData = {signature: logData.signature, linkSuffix: linkSuffix, explorer: this.selectedExplorer};
      let slotData = {slot: logData.slot, linkSuffix: linkSuffix, explorer: this.selectedExplorer};
      let programData = {programId: logData.solana.program_id, linkSuffix: linkSuffix, explorer: this.selectedExplorer};
      let parentProgramData = {
        parentProgramId: logData.solana.parent_program_id,
        linkSuffix: linkSuffix,
        explorer: this.selectedExplorer
      };

      // Extract compute units consumed from raw logs
      let computeUnits = null;
      const rawLogsArr = logData.solana.raw_logs ?? [];
      for (const entry of rawLogsArr) {
        const cuMatch = String(entry).match(/consumed\s+(\d+)\s+of\s+\d+\s+compute units/i);
        if (cuMatch) {
          computeUnits = parseInt(cuMatch[1], 10);
          break;
        }
      }

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
        computeUnits: computeUnits,
        logMessages: JSON.stringify(logData.solana.log_messages),
        dataLogs: JSON.stringify(logData.solana.data_logs),
        rawLogs: JSON.stringify(logData.solana.raw_logs),
        errors: JSON.stringify(logData.solana.errors),
        transactionError: logData.solana.transaction_error || ''
      };
    },
    async addProgramId() {
      if (this.newProgramId && !this.programIds.includes(this.newProgramId)) {
        const programIdToAdd = this.newProgramId;
        this.programIds.push(programIdToAdd);
        await this.connectWebSocketForProgram(programIdToAdd);
        this.newProgramId = '';

        // Attempt to fetch IDL for the program
        try {
          const {Program} = await import('@coral-xyz/anchor');
          const {Connection, PublicKey} = await import('@solana/web3.js');
          const wsUrl = this.selectedEnvironment === 'custom' ? this.customUrl : this.selectedEnvironment;
          const httpUrl = wsUrl.replace(/^wss?:\/\//, 'https://');
          const connection = new Connection(httpUrl);
          const pubkey = new PublicKey(programIdToAdd);
          const idl = await Program.fetchIdl(pubkey, {connection});
          if (idl) {
            this.uploadedIdl = idl;
            this.idlFileName = `${programIdToAdd.substring(0, 8)}...-on-chain.json`;
            this.toast.add({
              severity: 'success',
              summary: 'IDL Found',
              detail: `On-chain IDL loaded for ${programIdToAdd.substring(0, 8)}...`,
              life: 4000
            });
          } else {
            this.toast.add({
              severity: 'warn',
              summary: 'No IDL Found',
              detail: `No on-chain IDL found for ${programIdToAdd.substring(0, 8)}... You can upload one manually.`,
              life: 6000
            });
          }
        } catch (e) {
          console.warn('IDL fetch failed:', e);
          this.toast.add({
            severity: 'warn',
            summary: 'No IDL Found',
            detail: `Could not fetch IDL for ${programIdToAdd.substring(0, 8)}... You can upload one manually.`,
            life: 6000
          });
        }
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
        this.toast.add({
          severity: 'warn',
          summary: 'Invalid URL',
          detail: 'Enter a WebSocket URL starting with "wss://".',
          life: 4000
        });
        this.customUrl = '';
        this.selectedEnvironment = 'wss://api.devnet.solana.com';
      }
    },
    cancelMainnet() {
      this.showMainnetWarning = false;
      this.selectedEnvironment = this.pendingMainnetUrl ? 'wss://api.devnet.solana.com' : this.selectedEnvironment;
      this.pendingMainnetUrl = null;
    },
    confirmMainnet() {
      this.showMainnetWarning = false;
      this.selectedEnvironment = this.pendingMainnetUrl;
      this.pendingMainnetUrl = null;
      this.handleEnvironmentChange();
    },
    async handleEnvironmentChange() {
      if (this.selectedEnvironment === 'wss://api.mainnet-beta.solana.com') {
        this.pendingMainnetUrl = this.selectedEnvironment;
        this.showMainnetWarning = true;
        return;
      }
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
    togglePause() {
      this.isPaused = !this.isPaused;
    },
    clearLogs() {
      this.parsedLogs = [];
      this.lastUpdateTime = '-';
    },
    saveToLocalStorage() {
      localStorage.setItem('sologger_programIds', JSON.stringify(this.programIds));
      localStorage.setItem('sologger_customUrl', this.customUrl);
      localStorage.setItem('sologger_selectedEnvironment', this.selectedEnvironment);
      localStorage.setItem('sologger_selectedExplorer', this.selectedExplorer);
      localStorage.setItem('sologger_maxLogs', String(this.maxLogs));
    },
    loadFromLocalStorage() {
      const savedProgramIds = localStorage.getItem('sologger_programIds');
      if (savedProgramIds) {
        try {
          this.programIds = JSON.parse(savedProgramIds);
        } catch {
          this.programIds = [];
        }
      }
      const savedCustomUrl = localStorage.getItem('sologger_customUrl');
      if (savedCustomUrl) this.customUrl = savedCustomUrl;
      const savedEnv = localStorage.getItem('sologger_selectedEnvironment');
      if (savedEnv) this.selectedEnvironment = savedEnv;
      const savedExplorer = localStorage.getItem('sologger_selectedExplorer');
      if (savedExplorer) this.selectedExplorer = savedExplorer;
      const savedMaxLogs = localStorage.getItem('sologger_maxLogs');
      if (savedMaxLogs) {
        const n = parseInt(savedMaxLogs, 10);
        if (n >= 100) this.maxLogs = n;
      }
    },
    updateUrl() {
      const params = new URLSearchParams();
      if (this.programIds.length) params.set('programs', this.programIds.join(','));
      const envKey = this.environments.find(e => e.url === this.selectedEnvironment)?.key?.toLowerCase() ?? 'custom';
      params.set('network', envKey);
      const newUrl = `${window.location.pathname}?${params.toString()}`;
      window.history.replaceState({}, '', newUrl);
    },
    loadFromUrl() {
      const params = new URLSearchParams(window.location.search);
      const programs = params.get('programs');
      if (programs) {
        const ids = programs.split(',').map(s => s.trim()).filter(Boolean);
        if (ids.length) this.programIds = ids;
      }
      const network = params.get('network');
      if (network) {
        const match = this.environments.find(e => e.key.toLowerCase() === network.toLowerCase());
        if (match) this.selectedEnvironment = match.url;
      }
    },
    updateTable(eventData) {

      if (this.isPaused) return;

      if (eventData.method === 'logsNotification') {
        const transformer = this.getTransformer();

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

            sanitizedLogs.push(sanitizedLog);
          });

          try {
            const newParsedLogs = sanitizedLogs.map(log => this.parseLog(log));
            this.parsedLogs.unshift(...newParsedLogs);
            if (this.parsedLogs.length > this.maxLogs) {
              const removeCount = Math.ceil(this.maxLogs * 0.2);
              this.parsedLogs = this.parsedLogs.slice(0, this.parsedLogs.length - removeCount);
            }
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
          this.toast.add({
            severity: 'success',
            summary: 'Connected',
            detail: `Subscribed to ${programId.substring(0, 8)}...`,
            life: 3000
          });
          resolve();
        };

        ws.onmessage = setupMessageHandler();

        ws.onerror = (error) => {
          console.error(`WebSocket error for program ${programId}:`, error);
          this.websockets.delete(programId);
          this.connectingWebsockets.delete(programId);
          this.toast.add({
            severity: 'error',
            summary: 'Connection Error',
            detail: `Failed to connect for ${programId.substring(0, 8)}...`,
            life: 4000
          });
          reject(error);
        };

        ws.onclose = () => {
          console.log(`WebSocket connection closed for program: ${programId}`);
          this.websockets.delete(programId);
          this.connectingWebsockets.delete(programId);
          this.toast.add({
            severity: 'warn',
            summary: 'Disconnected',
            detail: `WebSocket closed for ${programId.substring(0, 8)}...`,
            life: 3000
          });
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
      this.uploadedIdl = null;
      this.idlFileName = '';
      console.log('Cleared all data and connections');

    },

    triggerDownload(content, filename, mimeType) {
      const blob = new Blob([content], {type: mimeType});
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    },
    exportJSON() {
      try {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const environment = this.getEnvironmentName();
        const downloadData = {
          metadata: {
            exportedAt: new Date().toISOString(),
            environment,
            url: this.selectedEnvironment === 'custom' ? this.customUrl : this.selectedEnvironment,
            programIds: this.programIds,
            totalLogs: this.parsedLogs.length
          },
          logs: this.parsedLogs
        };
        this.triggerDownload(JSON.stringify(downloadData, null, 2), `solana-logs-${environment}-${timestamp}.json`, 'application/json');
        this.toast.add({
          severity: 'success',
          summary: 'Export Complete',
          detail: 'Logs exported as JSON successfully.',
          life: 3000
        });
      } catch (error) {
        console.error('Error exporting JSON:', error);
        this.toast.add({
          severity: 'error',
          summary: 'Export Failed',
          detail: 'Error exporting JSON. Check console for details.',
          life: 4000
        });
      }
    },
    exportCSV() {
      try {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const environment = this.getEnvironmentName();
        const csvCols = ['timestamp', 'level', 'signature', 'slot', 'programId', 'parentProgramId', 'depth', 'instructionIndex', 'invokeResult', 'computeUnits', 'logMessages', 'dataLogs', 'rawLogs', 'errors', 'transactionError'];
        const escape = v => {
          if (v === null || v === undefined) return '';
          const s = typeof v === 'object' ? JSON.stringify(v) : String(v);
          return `"${s.replace(/"/g, '""')}"`;
        };
        const flatField = (row, col) => {
          const v = row[col];
          if (v && typeof v === 'object') return v[col] ?? JSON.stringify(v);
          return v;
        };
        const header = csvCols.join(',');
        const rows = this.parsedLogs.map(row => csvCols.map(col => escape(flatField(row, col))).join(','));
        this.triggerDownload([header, ...rows].join('\n'), `solana-logs-${environment}-${timestamp}.csv`, 'text/csv');
        this.toast.add({
          severity: 'success',
          summary: 'Export Complete',
          detail: 'Logs exported as CSV successfully.',
          life: 3000
        });
      } catch (error) {
        console.error('Error exporting CSV:', error);
        this.toast.add({
          severity: 'error',
          summary: 'Export Failed',
          detail: 'Error exporting CSV. Check console for details.',
          life: 4000
        });
      }
    },
    handleIdlUpload(event) {
      const file = event.target.files[0];
      if (!file) return;
      this.idlFileName = file.name;
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          this.uploadedIdl = JSON.parse(e.target.result);
          this.toast.add({
            severity: 'success',
            summary: 'IDL Loaded',
            detail: `${file.name} loaded successfully.`,
            life: 3000
          });
        } catch {
          this.toast.add({
            severity: 'error',
            summary: 'IDL Error',
            detail: 'Failed to parse IDL JSON file.',
            life: 4000
          });
          this.uploadedIdl = null;
          this.idlFileName = '';
        }
      };
      reader.readAsText(file);
    },
    async decodeWithIdl(log) {
      if (!this.uploadedIdl) return;
      try {
        const decoded = await decodeLogWithIdl(this.uploadedIdl, log);
        this.idlDecodedData = JSON.stringify(decoded, null, 2);
      } catch (e) {
        this.idlDecodedData = `Error decoding with IDL: ${e.message}`;
      }
    },
    async replayTransaction() {
      const sig = this.replaySignature.trim();
      if (!sig) return;
      this.replayLoading = true;
      this.replayError = '';
      try {
        // Determine HTTP RPC URL from current WS environment
        let rpcUrl = this.selectedEnvironment === 'custom' ? this.customUrl : this.selectedEnvironment;
        rpcUrl = rpcUrl.replace(/^wss?:\/\//, 'https://');
        const body = JSON.stringify({
          jsonrpc: '2.0', id: 1, method: 'getTransaction',
          params: [sig, {encoding: 'json', maxSupportedTransactionVersion: 0}]
        });
        const resp = await fetch(rpcUrl, {method: 'POST', headers: {'Content-Type': 'application/json'}, body});
        const json = await resp.json();
        if (json.error) throw new Error(json.error.message);
        const tx = json.result;
        if (!tx) throw new Error('Transaction not found. It may not be finalized yet or may not exist on this network.');
        const logs = tx.meta?.logMessages ?? [];
        const slot = tx.slot;
        const err = tx.meta?.err ?? null;
        const parsedLogs = this.getTransformer().from_rpc_logs_response({signature: sig, err, logs}, BigInt(slot));
        const newLogs = parsedLogs.map(l => this.parseLog({
          signature: sig, slot,
          solana: JSON.parse(sanitizeLogMessage(l))
        }));
        this.parsedLogs.unshift(...newLogs);
        if (this.parsedLogs.length > this.maxLogs) {
          const removeCount = Math.ceil(this.maxLogs * 0.2);
          this.parsedLogs = this.parsedLogs.slice(0, this.parsedLogs.length - removeCount);
        }
        this.lastUpdateTime = new Date().toLocaleTimeString();
        this.replaySignature = '';
      } catch (e) {
        this.replayError = e.message ?? 'Unknown error fetching transaction.';
      } finally {
        this.replayLoading = false;
      }
    }
  },
  created() {
    this.loadFromLocalStorage();
    this.loadFromUrl();
  },
  watch: {
    programIds: {
      deep: true,
      handler() {
        this.saveToLocalStorage();
        this.updateUrl();
      }
    },
    customUrl() {
      this.saveToLocalStorage();
    },
    selectedEnvironment() {
      this.saveToLocalStorage();
      this.updateUrl();
    },
    selectedExplorer() {
      this.saveToLocalStorage();
    },
    maxLogs() {
      this.saveToLocalStorage();
    }
  },
  beforeUnmount() {
    this.disconnectWebSocket();
  }
};
</script>

<style scoped>
/* Mobile optimizations */
@media (max-width: 768px) {
  .container {
    padding-left: 0.5rem;
    padding-right: 0.5rem;
  }
}

.rpc-provider-link {
  display: flex;
  flex-direction: column;
  padding: 0.6rem 0.75rem;
  border: 1px solid var(--p-card-border);
  border-radius: 0.5rem;
  text-decoration: none;
  color: var(--p-text-color);
  transition: border-color 0.15s;
}

.rpc-provider-link:hover {
  border-color: var(--p-primary-color);
  color: var(--p-primary-color);
}

.idl-decoded-pre {
  font-size: 0.8rem;
  font-family: monospace;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--p-text-color);
  background: var(--p-code-bg);
  border: 1px solid var(--p-card-border);
  padding: 0.75rem;
  border-radius: 0.5rem;
  margin: 0;
}
</style>