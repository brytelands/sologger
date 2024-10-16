/* tslint:disable */
/* eslint-disable */
/**
 * @param {(string)[]} logs
 * @param {(string)[]} program_ids
 * @returns {any}
 */
export function parse_logs_basic(logs: (string)[], program_ids: (string)[]): any;
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
