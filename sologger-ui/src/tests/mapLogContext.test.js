import {describe, expect, it} from 'vitest';
import {idlCandidatePrograms, mapLogContext} from '../composables/useLogMapper';

function sampleContext(overrides = {}) {
    return {
        signature: 'SIG123',
        slot: 42,
        solana: {
            program_id: 'CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C',
            parent_program_id: '',
            depth: 1,
            instruction_index: 0,
            instruction_name: 'swap_base_input',
            invoke_result: '',
            consumed_cu: 30000,
            max_cu: 200000,
            error_code: 6000,
            error_name: 'NotApproved',
            decoded_events: ['{"name":"SwapEvent","data":{"base_input":true}}'],
            log_messages: ['Instruction: swap_base_input'],
            data_logs: ['QMbN...'],
            raw_logs: ['Program CPMM... invoke [1]'],
            errors: ['custom program error: 0x1770'],
            transaction_error: '',
            ...overrides
        }
    };
}

describe('mapLogContext', () => {
    it('maps parser fields onto the LogsTable row shape', () => {
        const row = mapLogContext(sampleContext(), {linkSuffix: '?cluster=devnet', explorer: 'solana'});

        expect(row.signature).toEqual({signature: 'SIG123', linkSuffix: '?cluster=devnet', explorer: 'solana'});
        expect(row.slot).toEqual({slot: 42, linkSuffix: '?cluster=devnet', explorer: 'solana'});
        expect(row.programId.programId).toBe('CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C');
        expect(row.depth).toBe(1);
        expect(row.instructionName).toBe('swap_base_input');
        expect(row.computeUnits).toBe(30000);
        expect(row.maxComputeUnits).toBe(200000);
        expect(row.errorCode).toBe(6000);
        expect(row.errorName).toBe('NotApproved');
        expect(JSON.parse(row.decodedEvents)).toHaveLength(1);
        expect(JSON.parse(row.errors)).toEqual(['custom program error: 0x1770']);
        expect(row.level).toBe('Info');
    });

    it('marks rows with a transaction error as Error level', () => {
        const row = mapLogContext(sampleContext({transaction_error: 'InstructionError'}));
        expect(row.level).toBe('Error');
        expect(row.transactionError).toBe('InstructionError');
    });

    it('falls back to scraping raw logs for compute units', () => {
        const row = mapLogContext(sampleContext({
            consumed_cu: 0,
            raw_logs: ['Program X consumed 12345 of 200000 compute units']
        }));
        expect(row.computeUnits).toBe(12345);
    });

    it('defaults enrichment fields when the parser did not fill them', () => {
        const row = mapLogContext(sampleContext({
            instruction_name: '',
            error_code: null,
            error_name: null,
            decoded_events: undefined,
            consumed_cu: 0,
            max_cu: 0,
            raw_logs: []
        }));
        expect(row.instructionName).toBe('');
        expect(row.errorCode).toBeNull();
        expect(row.errorName).toBe('');
        expect(row.decodedEvents).toBe('[]');
        expect(row.computeUnits).toBeNull();
        expect(row.maxComputeUnits).toBeNull();
    });

    it('uses a provided timestamp instead of the wall clock', () => {
        const row = mapLogContext(sampleContext(), {timestamp: '12:00:00'});
        expect(row.timestamp).toBe('12:00:00');
    });
});

describe('idlCandidatePrograms', () => {
    const row = programId => ({programId: {programId}});

    it('returns unique programs in invoke order, skipping natives', () => {
        const candidates = idlCandidatePrograms([
            row('ComputeBudget111111111111111111111111111111'),
            row('CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C'),
            row('11111111111111111111111111111111'),
            row('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
            row('CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C'),
            row('JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB'),
        ]);
        expect(candidates).toEqual([
            'CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C',
            'JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB',
        ]);
    });

    it('skips programs the caller already has an IDL for', () => {
        const known = new Set(['CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C']);
        const candidates = idlCandidatePrograms([
            row('CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C'),
            row('JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB'),
        ], {known});
        expect(candidates).toEqual(['JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB']);
    });

    it('caps the number of candidates', () => {
        const rows = [];
        for (let i = 0; i < 10; i++) {
            rows.push(row(`Program${i}1111111111111111111111111111111111`));
        }
        expect(idlCandidatePrograms(rows, {limit: 3})).toHaveLength(3);
        expect(idlCandidatePrograms(rows)).toHaveLength(6);
    });

    it('handles empty and malformed rows', () => {
        expect(idlCandidatePrograms([])).toEqual([]);
        expect(idlCandidatePrograms([{programId: null}, {programId: {programId: ''}}])).toEqual([]);
    });
});
