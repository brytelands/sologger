<template>
  <div class="container mx-auto px-4 py-6">
    <div class="mb-8">
      <h1 class="text-3xl font-bold mb-2 text-[var(--p-text-color)] tracking-tight">
        Transaction Lookup
      </h1>
      <p class="text-[var(--p-text-muted)] text-base">
        Paste a transaction signature to fetch it from an RPC and inspect its parsed logs,
        CPI tree and compute-unit breakdown.
      </p>
    </div>

    <!-- Controls -->
    <div class="card p-4 mb-6 space-y-3">
      <div class="flex flex-col md:flex-row gap-2">
        <select v-model="selectedRpc" class="input-base w-full md:w-56">
          <option v-for="env in rpcEnvironments" :key="env.url" :value="env.url">{{ env.key }}</option>
          <option value="custom">Custom RPC…</option>
        </select>
        <input
            v-if="selectedRpc === 'custom'"
            v-model="customRpcUrl"
            type="text"
            placeholder="https://your-rpc.example.com"
            class="input-base flex-1"
        />
        <input
            v-model="signature"
            type="text"
            placeholder="Transaction signature..."
            class="input-base flex-1 font-mono"
            @keyup.enter="lookup"
        />
        <button @click="lookup" :disabled="!signature.trim() || loading" class="btn btn-primary whitespace-nowrap">
          {{ loading ? 'Fetching…' : 'Lookup' }}
        </button>
      </div>
      <div class="flex flex-col md:flex-row md:items-center gap-2">
        <label class="btn btn-secondary cursor-pointer whitespace-nowrap w-fit">
          {{ idlFileName || 'Upload IDL (optional)' }}
          <input type="file" accept=".json" class="hidden" @change="handleIdlUpload"/>
        </label>
        <label class="flex items-center gap-2 text-xs text-[var(--p-text-muted)] cursor-pointer select-none">
          <input type="checkbox" v-model="autoFetchIdls" @change="persistRpcChoice" class="accent-[var(--p-primary-color)]"/>
          Auto-fetch on-chain IDLs
        </label>
        <span class="text-xs text-[var(--p-text-muted)]">
          With an IDL, Program data events are decoded and error codes resolved to names.
          Anchor programs that publish their IDL on chain are picked up automatically.
        </span>
      </div>
      <p v-if="idlNotice" class="text-green-500 text-xs">{{ idlNotice }}</p>
      <p v-if="error" class="text-red-500 text-sm">{{ error }}</p>
    </div>

    <!-- Result -->
    <div v-if="rows.length">
      <!-- Transaction summary -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
        <div class="card p-4 flex flex-col gap-1">
          <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Slot</h4>
          <p class="text-2xl font-bold text-[var(--p-text-color)]">{{ txSlot }}</p>
        </div>
        <div class="card p-4 flex flex-col gap-1">
          <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Status</h4>
          <p class="text-2xl font-bold" :class="txFailed ? 'text-red-500' : 'text-green-500'">
            {{ txFailed ? 'Failed' : 'Success' }}
          </p>
        </div>
        <div class="card p-4 flex flex-col gap-1">
          <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Invocations</h4>
          <p class="text-2xl font-bold text-[var(--p-text-color)]">{{ rows.length }}</p>
        </div>
        <div class="card p-4 flex flex-col gap-1">
          <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Top-level CU</h4>
          <p class="text-2xl font-bold text-[var(--p-text-color)]">{{ totalCu.toLocaleString() }}</p>
        </div>
      </div>

      <!-- CU flamegraph -->
      <div class="card p-4 mb-4">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)] mb-2">
          Compute Unit Breakdown (CPI tree)
        </h4>
        <CuFlamegraph :rows="rows"/>
      </div>

      <!-- Parsed invocations -->
      <div class="overflow-x-auto">
        <LogsTable :parsedLogs="rows" :uploadedIdl="tableIdl" @decode-with-idl="openDecode"/>
      </div>
    </div>
    <div v-else-if="!loading" class="h-40 flex items-center justify-center border border-dashed border-[var(--p-card-border)] rounded-xl bg-[var(--p-card-bg)]">
      <span class="text-[var(--p-text-color)]">Parsed transaction logs will appear here</span>
    </div>

    <!-- IDL decode modal (same shape as HomeView's) -->
    <div v-if="idlDecodedData" class="modal-overlay" @click.self="idlDecodedData = null">
      <div class="modal-panel modal-panel--wide">
        <div class="modal-header">
          <span class="font-semibold text-base">IDL Decode</span>
          <button @click="idlDecodedData = null" class="btn btn-secondary">✕ Close</button>
        </div>
        <div class="modal-body">
          <pre class="text-xs whitespace-pre-wrap">{{ idlDecodedData }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import {onMounted} from 'vue';
import init, {
  WasmLogContextTransformer
} from '../../public/sologger-log-transformer-wasm/pkg/sologger_log_transformer_wasm.js';
import {decodeWithIdl} from '../composables/useIdlDecoder';
import {idlCandidatePrograms, mapLogContext} from '../composables/useLogMapper';
import {sanitizeLogMessage} from '../composables/useLogSanitizer';
import LogsTable from '../components/LogsTable.vue';
import CuFlamegraph from '../components/CuFlamegraph.vue';

export default {
  name: 'LookupView',
  components: {LogsTable, CuFlamegraph},
  setup() {
    onMounted(async () => {
      await init();
    });
  },
  data() {
    return {
      rpcEnvironments: [
        {key: 'Devnet', url: 'https://api.devnet.solana.com'},
        {key: 'Testnet', url: 'https://api.testnet.solana.com'},
        {key: 'Mainnet', url: 'https://api.mainnet-beta.solana.com'}
      ],
      selectedRpc: 'https://api.devnet.solana.com',
      customRpcUrl: '',
      signature: '',
      loading: false,
      error: '',
      rows: [],
      uploadedIdl: null,
      idlFileName: '',
      idlDecodedData: null,
      // Signature -> program -> on-chain IDL discovery
      autoFetchIdls: true,
      autoIdls: {},
      autoIdlPrograms: [],
      idlNotice: '',
    };
  },
  computed: {
    rpcUrl() {
      return this.selectedRpc === 'custom' ? this.customRpcUrl.trim() : this.selectedRpc;
    },
    linkSuffix() {
      if (this.rpcUrl.includes('devnet')) return '?cluster=devnet';
      if (this.rpcUrl.includes('testnet')) return '?cluster=testnet';
      return '';
    },
    txSlot() {
      return this.rows[0]?.slot?.slot ?? '';
    },
    txFailed() {
      return this.rows.some(row => {
        try {
          return row.transactionError !== '' || JSON.parse(row.errors).length > 0;
        } catch {
          return row.transactionError !== '';
        }
      });
    },
    totalCu() {
      // Top-level invocations only: parents' CU already includes their CPI children
      return this.rows
          .filter(row => row.depth === 1)
          .reduce((sum, row) => sum + (row.computeUnits ?? 0), 0);
    },
    // Truthy when any IDL (manual or discovered) exists, so LogsTable offers the
    // decode button; openDecode picks the right IDL per program
    tableIdl() {
      if (this.uploadedIdl) return this.uploadedIdl;
      return this.autoIdlPrograms.length ? this.autoIdls[this.autoIdlPrograms[0]] : null;
    }
  },
  created() {
    const savedRpc = localStorage.getItem('sologger_lookupRpc');
    if (savedRpc) this.selectedRpc = savedRpc;
    const savedCustom = localStorage.getItem('sologger_lookupCustomRpc');
    if (savedCustom) this.customRpcUrl = savedCustom;
    const savedAutoIdl = localStorage.getItem('sologger_lookupAutoIdl');
    if (savedAutoIdl !== null) this.autoFetchIdls = savedAutoIdl === 'true';
    // Shareable deep link: /lookup?sig=<signature>
    const querySig = this.$route.query.sig;
    if (querySig) {
      this.signature = String(querySig);
      this.$nextTick(() => this.lookup());
    }
  },
  methods: {
    // One transformer for the component's lifetime; kept off `data` so Vue doesn't
    // proxy the WASM object.
    getTransformer() {
      if (!this._transformer) {
        this._transformer = new WasmLogContextTransformer(['*']);
      }
      return this._transformer;
    },
    persistRpcChoice() {
      localStorage.setItem('sologger_lookupRpc', this.selectedRpc);
      localStorage.setItem('sologger_lookupCustomRpc', this.customRpcUrl);
      localStorage.setItem('sologger_lookupAutoIdl', String(this.autoFetchIdls));
    },
    handleIdlUpload(event) {
      const file = event.target.files[0];
      if (!file) return;
      this.idlFileName = file.name;
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          this.uploadedIdl = JSON.parse(e.target.result);
          this.registerIdl();
          // Re-decode the current result with the fresh IDL
          if (this.rows.length && this.signature.trim()) this.lookup();
        } catch {
          this.error = 'Failed to parse IDL JSON file.';
          this.uploadedIdl = null;
          this.idlFileName = '';
        }
      };
      reader.readAsText(file);
    },
    registerIdl() {
      if (!this.uploadedIdl) return;
      const idlJson = JSON.stringify(this.uploadedIdl);
      // 0.30+ IDLs carry their program address; otherwise register once results arrive
      const targets = new Set();
      if (this.uploadedIdl.address) targets.add(this.uploadedIdl.address);
      for (const row of this.rows) {
        const pid = row.programId?.programId;
        if (pid) targets.add(pid);
      }
      for (const programId of targets) {
        try {
          this.getTransformer().add_idl(programId, idlJson);
        } catch (e) {
          console.warn(`Failed to register IDL for ${programId}:`, e);
        }
      }
    },
    async openDecode(row) {
      // Prefer the IDL discovered for this row's own program over the manual upload
      const programId = row.programId?.programId ?? row.programId ?? '';
      const idl = this.autoIdls[programId] ?? this.uploadedIdl;
      if (!idl) return;
      try {
        const decoded = await decodeWithIdl(idl, row);
        this.idlDecodedData = JSON.stringify(decoded, null, 2);
      } catch (e) {
        this.idlDecodedData = `Error decoding with IDL: ${e.message}`;
      }
    },
    parseRows(sig, err, logs, slot) {
      const parsed = this.getTransformer().from_rpc_logs_response({signature: sig, err, logs}, BigInt(slot));
      const explorer = localStorage.getItem('sologger_selectedExplorer') || 'solscan';
      return parsed.map(solanaLog => mapLogContext({
        signature: sig,
        slot,
        solana: JSON.parse(sanitizeLogMessage(solanaLog))
      }, {linkSuffix: this.linkSuffix, explorer}));
    },
    // Trace the transaction's programs to their published on-chain IDLs (Anchor stores
    // the IDL at a deterministic PDA; Program.fetchIdl resolves and inflates it).
    // Returns how many new IDLs were registered with the transformer.
    async discoverOnChainIdls() {
      const known = new Set(Object.keys(this.autoIdls));
      if (this.uploadedIdl?.address) known.add(this.uploadedIdl.address);
      const candidates = idlCandidatePrograms(this.rows, {known});
      if (!candidates.length) return 0;

      let found = 0;
      try {
        const {Program} = await import('@coral-xyz/anchor');
        const {Connection, PublicKey} = await import('@solana/web3.js');
        const connection = new Connection(this.rpcUrl);

        const results = await Promise.allSettled(candidates.map(async programId => {
          const idl = await Program.fetchIdl(new PublicKey(programId), {connection});
          return {programId, idl};
        }));
        for (const result of results) {
          if (result.status !== 'fulfilled' || !result.value.idl) continue;
          const {programId, idl} = result.value;
          try {
            this.getTransformer().add_idl(programId, JSON.stringify(idl));
            this.autoIdls[programId] = idl;
            this.autoIdlPrograms.push(programId);
            found++;
          } catch (e) {
            console.warn(`Discovered IDL for ${programId} but registration failed:`, e);
          }
        }
      } catch (e) {
        console.warn('On-chain IDL discovery failed:', e);
      }

      if (this.autoIdlPrograms.length) {
        this.idlNotice = `On-chain IDL loaded for ${
            this.autoIdlPrograms.map(p => p.substring(0, 8) + '…').join(', ')
        }`;
      }
      return found;
    },
    async lookup() {
      const sig = this.signature.trim();
      if (!sig || !this.rpcUrl) return;
      this.loading = true;
      this.error = '';
      this.idlNotice = '';
      this.persistRpcChoice();
      // Keep the URL shareable
      if (this.$route.query.sig !== sig) {
        this.$router.replace({query: {sig}});
      }
      try {
        const body = JSON.stringify({
          jsonrpc: '2.0', id: 1, method: 'getTransaction',
          params: [sig, {encoding: 'json', maxSupportedTransactionVersion: 0}]
        });
        const resp = await fetch(this.rpcUrl, {
          method: 'POST',
          headers: {'Content-Type': 'application/json'},
          body
        });
        const json = await resp.json();
        if (json.error) throw new Error(json.error.message);
        const tx = json.result;
        if (!tx) throw new Error('Transaction not found. It may not be finalized yet or may not exist on this network.');

        const logs = tx.meta?.logMessages ?? [];
        const slot = tx.slot;
        const err = tx.meta?.err ?? null;

        this.registerIdl();
        this.rows = this.parseRows(sig, err, logs, slot);

        // A manual IDL without an address: now that the programs are known, register
        // it against them and re-parse once
        if (this.uploadedIdl && !this.uploadedIdl.address && this.rows.length) {
          this.registerIdl();
          this.rows = this.parseRows(sig, err, logs, slot);
        }

        // Auto-discovery: rows are already on screen; enrich them when IDLs turn up
        if (this.autoFetchIdls && this.rows.length) {
          const found = await this.discoverOnChainIdls();
          if (found > 0) {
            this.rows = this.parseRows(sig, err, logs, slot);
          }
        }
      } catch (e) {
        this.rows = [];
        this.error = e.message ?? 'Unknown error fetching transaction.';
      } finally {
        this.loading = false;
      }
    }
  }
};
</script>
