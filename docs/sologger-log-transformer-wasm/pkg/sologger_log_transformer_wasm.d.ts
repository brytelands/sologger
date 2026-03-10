/* tslint:disable */
/* eslint-disable */

export class WasmLogContextTransformer {
  free(): void;
  [Symbol.dispose](): void;
  from_rpc_response(response: any): any;
  from_rpc_logs_response(rpc_logs_response: any, slot: bigint): any;
  constructor(program_ids: string[]);
}

export class WasmLogParser {
  free(): void;
  [Symbol.dispose](): void;
  parse_logs(logs: string[], transaction_error: string, slot: bigint, signature: string): any;
  constructor(program_ids: string[]);
}

export function parse_logs_basic(logs: string[], program_ids: string[]): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmlogcontexttransformer_free: (a: number, b: number) => void;
  readonly wasmlogcontexttransformer_from_rpc_logs_response: (a: number, b: any, c: bigint) => [number, number, number];
  readonly wasmlogcontexttransformer_from_rpc_response: (a: number, b: any) => [number, number, number];
  readonly wasmlogcontexttransformer_new: (a: number, b: number) => number;
  readonly __wbg_wasmlogparser_free: (a: number, b: number) => void;
  readonly parse_logs_basic: (a: number, b: number, c: number, d: number) => any;
  readonly wasmlogparser_new: (a: number, b: number) => number;
  readonly wasmlogparser_parse_logs: (a: number, b: number, c: number, d: number, e: number, f: bigint, g: number, h: number) => any;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
