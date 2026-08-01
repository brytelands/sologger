// Decodes a parsed log row against an uploaded Anchor IDL.
// Framework-free on purpose: HomeView.vue and the test suite both import this,
// so behavior changes here are covered by src/tests/decodeWithIdl.test.js.
export async function decodeWithIdl(uploadedIdl, log) {
    if (!uploadedIdl) return null;

    // Dynamic import keeps @coral-xyz/anchor out of the initial bundle.
    const {BorshCoder} = await import('@coral-xyz/anchor');
    const coder = new BorshCoder(uploadedIdl);

    // Extract data logs from the row
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

    // Decode instruction data from dataLogs using BorshCoder
    const decodedInstructions = [];
    for (const b64 of dataLogs) {
        try {
            const buf = Buffer.from(b64, 'base64');
            const decoded = coder.instruction.decode(buf);
            if (decoded) decodedInstructions.push({name: decoded.name, data: decoded.data});
        } catch { /* skip undecoded entries */
        }
    }

    // Decode events from dataLogs using BorshCoder
    const decodedEvents = [];
    for (const b64 of dataLogs) {
        try {
            const decoded = coder.events.decode(b64);
            if (decoded) decodedEvents.push({name: decoded.name, data: decoded.data});
        } catch { /* skip undecoded entries */
        }
    }

    return {
        program: log.programId?.programId ?? log.programId ?? '',
        signature: log.signature?.signature ?? log.signature ?? '',
        matchedInstructions,
        decodedInstructions,
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
