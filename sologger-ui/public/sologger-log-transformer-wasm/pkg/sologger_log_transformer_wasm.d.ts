/* tslint:disable */
/* eslint-disable */
/**
 * @param {(string)[]} logs
 * @param {(string)[]} program_ids
 * @returns {any}
 */
export function parse_logs_basic(logs: (string)[], program_ids: (string)[]): any;
export class WasmLogContextTransformer {
  free(): void;
  /**
   * @param {(string)[]} program_ids
   */
  constructor(program_ids: (string)[]);
  /**
   * @param {any} response
   * @returns {any}
   */
  from_rpc_response(response: any): any;
  /**
   * @param {any} rpc_logs_response
   * @param {bigint} slot
   * @returns {any}
   */
  from_rpc_logs_response(rpc_logs_response: any, slot: bigint): any;
}
export class WasmLogParser {
  free(): void;
  /**
   * @param {(string)[]} program_ids
   */
  constructor(program_ids: (string)[]);
  /**
   * @param {(string)[]} logs
   * @param {string} transaction_error
   * @param {bigint} slot
   * @param {string} signature
   * @returns {any}
   */
  parse_logs(logs: (string)[], transaction_error: string, slot: bigint, signature: string): any;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmlogcontexttransformer_free: (a: number, b: number) => void;
  readonly wasmlogcontexttransformer_new: (a: number, b: number) => number;
  readonly wasmlogcontexttransformer_from_rpc_response: (a: number, b: number, c: number) => void;
  readonly wasmlogcontexttransformer_from_rpc_logs_response: (a: number, b: number, c: number, d: number) => void;
  readonly __wbg_wasmlogparser_free: (a: number, b: number) => void;
  readonly wasmlogparser_new: (a: number, b: number) => number;
  readonly wasmlogparser_parse_logs: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly parse_logs_basic: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
