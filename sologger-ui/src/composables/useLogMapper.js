// Maps a parsed LogContext (the snake_case JSON emitted by the WASM module) onto the
// row shape LogsTable.vue renders. Framework-free on purpose: HomeView, LookupView and
// the test suite all import this, so behavior changes here are covered by
// src/tests/mapLogContext.test.js.

/**
 * @param logData {{signature: any, slot: any, solana: object}} one parsed LogContext,
 *        wrapped with the signature/slot the caller received it under
 * @param options {{linkSuffix?: string, explorer?: string, timestamp?: string}}
 */
export function mapLogContext(logData, options = {}) {
    const {linkSuffix = '', explorer = 'solscan', timestamp} = options;
    const solana = logData.solana ?? {};

    // consumed_cu comes straight from the parser; fall back to scraping raw logs for
    // records produced by older WASM builds that predate the field
    let computeUnits = Number(solana.consumed_cu) > 0 ? Number(solana.consumed_cu) : null;
    if (computeUnits === null) {
        for (const entry of solana.raw_logs ?? []) {
            const cuMatch = String(entry).match(/consumed\s+(\d+)\s+of\s+\d+\s+compute units/i);
            if (cuMatch) {
                computeUnits = parseInt(cuMatch[1], 10);
                break;
            }
        }
    }

    return {
        timestamp: timestamp ?? new Date().toLocaleTimeString(),
        level: solana.transaction_error !== null && solana.transaction_error !== '' ? 'Error' : 'Info',
        signature: {signature: logData.signature, linkSuffix, explorer},
        slot: {slot: logData.slot, linkSuffix, explorer},
        programId: {programId: solana.program_id, linkSuffix, explorer},
        parentProgramId: {parentProgramId: solana.parent_program_id, linkSuffix, explorer},
        depth: solana.depth,
        instructionIndex: solana.instruction_index,
        instructionName: solana.instruction_name || '',
        invokeResult: solana.invoke_result,
        computeUnits: computeUnits,
        maxComputeUnits: Number(solana.max_cu) > 0 ? Number(solana.max_cu) : null,
        errorCode: solana.error_code ?? null,
        errorName: solana.error_name ?? '',
        decodedEvents: JSON.stringify(solana.decoded_events ?? []),
        logMessages: JSON.stringify(solana.log_messages ?? []),
        dataLogs: JSON.stringify(solana.data_logs ?? []),
        rawLogs: JSON.stringify(solana.raw_logs ?? []),
        errors: JSON.stringify(solana.errors ?? []),
        transactionError: solana.transaction_error || ''
    };
}
