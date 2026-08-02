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

// Native/SPL programs that never publish an Anchor IDL — not worth an RPC round trip.
const NATIVE_PROGRAMS = new Set([
    '11111111111111111111111111111111',
    'ComputeBudget111111111111111111111111111111',
    'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
    'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb',
    'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
    'Vote111111111111111111111111111111111111111',
    'Stake11111111111111111111111111111111111111',
    'AddressLookupTab1e1111111111111111111111111',
    'BPFLoaderUpgradeab1e11111111111111111111111',
    'MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr',
    'Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo',
]);

/**
 * The programs of a parsed transaction worth asking the chain for an Anchor IDL:
 * unique, in invoke order, minus native programs and anything already `known`,
 * capped so one lookup never fans out into unbounded RPC calls.
 */
export function idlCandidatePrograms(rows, {known = new Set(), limit = 6} = {}) {
    const candidates = [];
    for (const row of rows) {
        const programId = row.programId?.programId ?? String(row.programId ?? '');
        if (!programId || NATIVE_PROGRAMS.has(programId) || known.has(programId)
            || candidates.includes(programId)) {
            continue;
        }
        candidates.push(programId);
        if (candidates.length >= limit) break;
    }
    return candidates;
}
