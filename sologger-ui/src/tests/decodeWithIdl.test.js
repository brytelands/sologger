import { describe, it, expect } from 'vitest';
import { BorshCoder } from '@coral-xyz/anchor';

// Extracted pure implementation of decodeWithIdl from HomeView.vue
async function decodeWithIdl(uploadedIdl, log) {
  if (!uploadedIdl) return null;

  const coder = new BorshCoder(uploadedIdl);

  let dataLogs = [];
  try { dataLogs = JSON.parse(log.dataLogs ?? '[]'); } catch { dataLogs = []; }
  let rawLogs = [];
  try { rawLogs = JSON.parse(log.rawLogs ?? '[]'); } catch { rawLogs = []; }

  let logMessages = [];
  try { logMessages = JSON.parse(log.logMessages ?? '[]'); } catch { logMessages = []; }

  const idlInstructions = uploadedIdl?.instructions ?? [];
  const matchedInstructions = [];
  for (const msg of logMessages) {
    const instrMatch = String(msg).match(/Instruction:\s*(\w+)/);
    if (instrMatch) {
      const name = instrMatch[1];
      const idlInstr = idlInstructions.find(i => i.name?.toLowerCase() === name.toLowerCase());
      if (idlInstr) matchedInstructions.push({ name, idlInstruction: idlInstr });
    }
  }

  // Decode instruction data from dataLogs using BorshCoder
  const decodedInstructions = [];
  for (const b64 of dataLogs) {
    try {
      const buf = Buffer.from(b64, 'base64');
      const decoded = coder.instruction.decode(buf);
      if (decoded) decodedInstructions.push({ name: decoded.name, data: decoded.data });
    } catch { /* skip undecoded entries */ }
  }

  // Decode events from dataLogs using BorshCoder
  const decodedEvents = [];
  for (const b64 of dataLogs) {
    try {
      const decoded = coder.events.decode(b64);
      if (decoded) decodedEvents.push({ name: decoded.name, data: decoded.data });
    } catch { /* skip undecoded entries */ }
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

const sampleIdl = {
  name: 'my_program',
  version: '0.1.0',
  instructions: [
    { name: 'initialize', discriminator: [175,175,109,31,13,152,155,237], args: [{ name: 'amount', type: 'u64' }] },
    { name: 'transfer', discriminator: [163,52,200,231,140,3,69,186], args: [{ name: 'amount', type: 'u64' }] }
  ],
  events: [],
  errors: [],
  types: []
};

describe('decodeWithIdl', () => {
  it('returns null when no IDL is uploaded', async () => {
    const result = await decodeWithIdl(null, { logMessages: '[]', dataLogs: '[]', rawLogs: '[]' });
    expect(result).toBeNull();
  });

  it('matches a single IDL instruction from log messages', async () => {
    const log = {
      programId: 'ABC123',
      signature: 'SIG456',
      logMessages: JSON.stringify(['Program log: Instruction: initialize']),
      dataLogs: '[]',
      rawLogs: '[]'
    };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.matchedInstructions).toHaveLength(1);
    expect(result.matchedInstructions[0].name).toBe('initialize');
    expect(result.matchedInstructions[0].idlInstruction).toEqual(sampleIdl.instructions[0]);
    expect(result.note).toBe('Found 1 matching instruction(s) in IDL.');
  });

  it('matches multiple IDL instructions from log messages', async () => {
    const log = {
      programId: 'ABC123',
      signature: 'SIG456',
      logMessages: JSON.stringify([
        'Program log: Instruction: initialize',
        'Program log: Instruction: transfer'
      ]),
      dataLogs: '[]',
      rawLogs: '[]'
    };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.matchedInstructions).toHaveLength(2);
    expect(result.note).toBe('Found 2 matching instruction(s) in IDL.');
  });

  it('returns no-match note when instruction is not in IDL', async () => {
    const log = {
      programId: 'ABC123',
      signature: 'SIG456',
      logMessages: JSON.stringify(['Program log: Instruction: unknownInstruction']),
      dataLogs: '[]',
      rawLogs: '[]'
    };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.matchedInstructions).toHaveLength(0);
    expect(result.note).toBe('No matching IDL instructions found in log messages. Raw data logs shown below.');
  });

  it('is case-insensitive when matching instruction names', async () => {
    const log = {
      programId: 'ABC123',
      signature: 'SIG456',
      logMessages: JSON.stringify(['Program log: Instruction: INITIALIZE']),
      dataLogs: '[]',
      rawLogs: '[]'
    };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.matchedInstructions).toHaveLength(1);
    expect(result.matchedInstructions[0].name).toBe('INITIALIZE');
  });

  it('parses dataLogs and rawLogs from JSON strings', async () => {
    const log = {
      programId: 'ABC123',
      signature: 'SIG456',
      logMessages: '[]',
      dataLogs: JSON.stringify(['data1', 'data2']),
      rawLogs: JSON.stringify(['raw1'])
    };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.dataLogs).toEqual(['data1', 'data2']);
    expect(result.rawLogs).toEqual(['raw1']);
  });

  it('handles malformed JSON in dataLogs and rawLogs gracefully', async () => {
    const log = {
      programId: 'ABC123',
      signature: 'SIG456',
      logMessages: '[]',
      dataLogs: 'not-json',
      rawLogs: '{bad}'
    };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.dataLogs).toEqual([]);
    expect(result.rawLogs).toEqual([]);
  });

  it('extracts programId and signature from nested objects', async () => {
    const log = {
      programId: { programId: 'PROG999', linkSuffix: '?cluster=devnet' },
      signature: { signature: 'SIG888', linkSuffix: '?cluster=devnet' },
      logMessages: '[]',
      dataLogs: '[]',
      rawLogs: '[]'
    };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.program).toBe('PROG999');
    expect(result.signature).toBe('SIG888');
  });

  it('includes IDL name and version in the result', async () => {
    const log = { programId: '', signature: '', logMessages: '[]', dataLogs: '[]', rawLogs: '[]' };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.idlName).toBe('my_program');
    expect(result.idlVersion).toBe('0.1.0');
  });

  it('falls back to Unknown for IDL name and version when missing', async () => {
    const idlNoMeta = { instructions: [], events: [], errors: [], types: [] };
    const log = { programId: '', signature: '', logMessages: '[]', dataLogs: '[]', rawLogs: '[]' };
    const result = await decodeWithIdl(idlNoMeta, log);
    expect(result.idlName).toBe('Unknown');
    expect(result.idlVersion).toBe('Unknown');
  });

  it('handles missing logMessages field gracefully', async () => {
    const log = { programId: 'P', signature: 'S', dataLogs: '[]', rawLogs: '[]' };
    const result = await decodeWithIdl(sampleIdl, log);
    expect(result.matchedInstructions).toHaveLength(0);
  });

  it('decodes real Raydium CPMM swap_base_input log data using the CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C IDL', async () => {
    const { readFileSync } = await import('fs');
    const { resolve, dirname } = await import('path');
    const { fileURLToPath } = await import('url');
    const __dirname = dirname(fileURLToPath(import.meta.url));
    const raydiumIdl = JSON.parse(readFileSync(resolve(__dirname, 'CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C-idl.json'), 'utf-8'));

    const base64Data = 'QMbN6CYIceLSNPnH1+FupbOsfFcMMAYfk5pbewHNLOiV2Y+rcY++tVceiNvqAAAAXCE+PG0YAQAAQFlzBwAAAEdVjQ2aCAAAAAAAAAAAAAAAAAAAAAAAAAEGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAcIpi8RZuMywUoWwsWLby7IdSeCUTeKHmUBTEknb34PhALTEBAAAAAAAAAAAAAAAAAE=';

    const log = {
      programId: 'CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C',
      signature: '5xTestSignature',
      logMessages: JSON.stringify([
        'Program CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C invoke [1]',
        'Program log: Instruction: swap_base_input',
        'Program CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C success'
      ]),
      dataLogs: JSON.stringify([base64Data]),
      rawLogs: '[]'
    };

    const result = await decodeWithIdl(raydiumIdl, log);

    expect(result).not.toBeNull();
    expect(result.idlName).toBe('raydium_cp_swap');
    expect(result.idlVersion).toBe('0.2.0');
    expect(result.program).toBe('CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C');
    expect(result.matchedInstructions).toHaveLength(1);
    expect(result.matchedInstructions[0].name).toBe('swap_base_input');
    expect(result.dataLogs).toEqual([base64Data]);
    expect(result.note).toBe('Found 1 matching instruction(s) in IDL.');

    // BorshCoder event decoding: the base64 data is a SwapEvent
    expect(result.decodedEvents).toHaveLength(1);
    expect(result.decodedEvents[0].name).toBe('SwapEvent');
    expect(result.decodedEvents[0].data).toHaveProperty('pool_id');
    expect(result.decodedEvents[0].data).toHaveProperty('input_amount');
    expect(result.decodedEvents[0].data).toHaveProperty('output_amount');
    expect(result.decodedEvents[0].data.base_input).toBe(true);

    // No instruction-level decoding expected for this data
    expect(result.decodedInstructions).toHaveLength(0);
  });
});
