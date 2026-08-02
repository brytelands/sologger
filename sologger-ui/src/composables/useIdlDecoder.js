// Decodes a parsed log row against an uploaded Anchor IDL.
//
// Event decoding is done by the Rust decoder (sologger-idl-decoder) through the WASM
// module's decode_program_data export — the former @coral-xyz/anchor BorshCoder path
// was retired once the Rust decoder shipped, so browser and binary share one decoder.
//
// Framework-free on purpose: HomeView.vue and the test suite both import this, so
// behavior changes here are covered by src/tests/decodeWithIdl.test.js. The WASM module
// must be initialized before calling (HomeView does this on mount; tests use initSync).
import {decode_program_data} from '../../public/sologger-log-transformer-wasm/pkg/sologger_log_transformer_wasm.js';

export async function decodeWithIdl(uploadedIdl, log) {
    if (!uploadedIdl) return null;

    let dataLogs = [];
    try {
        dataLogs = JSON.parse(log.dataLogs ?? '[]');
    } catch {
        dataLogs = [];
    }
    let rawLogs = [];
    try {
        rawLogs = JSON.parse(log.rawLogs ?? '[]');
    } catch {
        rawLogs = [];
    }

    // Try to match IDL instructions by name from log messages
    let logMessages = [];
    try {
        logMessages = JSON.parse(log.logMessages ?? '[]');
    } catch {
        logMessages = [];
    }

    const idlInstructions = uploadedIdl?.instructions ?? [];
    const matchedInstructions = [];
    for (const msg of logMessages) {
        const instrMatch = String(msg).match(/Instruction:\s*(\w+)/);
        if (instrMatch) {
            const name = instrMatch[1];
            const idlInstr = idlInstructions.find(i => i.name?.toLowerCase() === name.toLowerCase());
            if (idlInstr) matchedInstructions.push({name, idlInstruction: idlInstr});
        }
    }

    // Borsh-decode events from dataLogs with the Rust decoder (handles both the legacy
    // and the 0.30+ IDL spec)
    const idlJson = JSON.stringify(uploadedIdl);
    const decodedEvents = [];
    for (const b64 of dataLogs) {
        try {
            const decoded = decode_program_data(idlJson, String(b64));
            if (decoded) decodedEvents.push({name: decoded.name, data: decoded.data});
        } catch { /* skip undecodable entries */
        }
    }

    return {
        program: log.programId?.programId ?? log.programId ?? '',
        signature: log.signature?.signature ?? log.signature ?? '',
        matchedInstructions,
        decodedEvents,
        dataLogs,
        rawLogs,
        idlName: uploadedIdl?.name ?? uploadedIdl?.metadata?.name ?? 'Unknown',
        idlVersion: uploadedIdl?.version ?? uploadedIdl?.metadata?.version ?? 'Unknown',
        note: matchedInstructions.length === 0
            ? 'No matching IDL instructions found in log messages.'
            : `Found ${matchedInstructions.length} matching instruction(s) in IDL.`
    };
}
