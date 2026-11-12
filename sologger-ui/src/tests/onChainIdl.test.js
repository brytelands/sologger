import {describe, expect, it} from 'vitest';
import {readFileSync} from 'fs';
import {dirname, resolve} from 'path';
import {fileURLToPath} from 'url';
import {deflateSync} from 'zlib';
import {anchorIdlAddress, decodeAnchorIdlAccount, fetchAnchorIdl} from '../composables/useOnChainIdl';

const __dirname = dirname(fileURLToPath(import.meta.url));
const IDL = JSON.parse(readFileSync(resolve(__dirname, 'CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C-idl.json'), 'utf8'));
const RPC = 'https://rpc.example';
const CPMM = 'CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C';

// Computed with idlAddress() from @coral-xyz/anchor 0.32.1, the library this replaces
const ANCHOR_IDL_ADDRESSES = {
    'CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C': '3HD1FNEKoNh5aYfvw3VrNWy6WwrEtS6JYx1RmFTE7DMC',
    'JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4': 'C88XWfp26heEmDkmfSzeXP7Fd7GQJ2j9dDTUsyiZbUTa',
    'dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH': '8BKPjRu7Hvd6Y2J67EdLx9MsrZFQwEMVwYg7NeVbAPF8',
    'CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK': 'FbDNAPsgm8Y5jWXQkXjpfN6Bp8xHeRkhMmsstAooPyY9',
    'whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc': '2KFqE4RWoPVbvodo8vbggCFeHPS8TDvgpwp79ALMrcyn',
    'LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo': '7UZRobkzaKVm1RbCH5WdFaYCGzCRjnu3prziHAsYiSyr',
    'KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD': '8qLKwp1fk8WyqmzarkuMeZEX3AzL4VDSmA2UZTKT2aCJ',
    'MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA': '9tb2tzSkQqDng2h8dC2xbuKHfQpmPoZP4vnd5Y1rkmmK',
    'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA': '4nmzuuebvZ9EVghi2khvz9SyxUNvuXXdAzWtG5N7avYf'
};

/**
 * An IDL account's data as Anchor writes it: 8-byte discriminator, 32-byte authority,
 * u32 LE length, zlib-compressed JSON, then any unused capacity.
 */
function idlAccountBase64(idl, {extraCapacity = 0, length} = {}) {
    const compressed = deflateSync(Buffer.from(JSON.stringify(idl)));
    const data = Buffer.alloc(8 + 32 + 4 + compressed.length + extraCapacity);
    Buffer.from([0x18, 0x46, 0x62, 0xbf, 0x3a, 0x90, 0x7b, 0x9e]).copy(data, 0);
    data.fill(7, 8, 40);
    data.writeUInt32LE(length ?? compressed.length, 40);
    compressed.copy(data, 44);
    return data.toString('base64');
}

/** A fetch stand-in that answers one JSON-RPC call and records the request. */
function rpcStub(reply, {status = 200} = {}) {
    const requests = [];
    const fetchImpl = async (url, init) => {
        requests.push({url, ...init, body: JSON.parse(init.body)});
        return new Response(JSON.stringify({jsonrpc: '2.0', id: 1, ...reply}), {status});
    };
    return {fetchImpl, requests};
}

describe('anchorIdlAddress', () => {
    it.each(Object.entries(ANCHOR_IDL_ADDRESSES))('matches Anchor for %s', async (programId, expected) => {
        expect(await anchorIdlAddress(programId)).toBe(expected);
    });

    it('rejects a string that is not a program address', async () => {
        await expect(anchorIdlAddress('not-a-pubkey')).rejects.toThrow();
    });
});

describe('decodeAnchorIdlAccount', () => {
    it('inflates the IDL JSON', async () => {
        expect(await decodeAnchorIdlAccount(idlAccountBase64(IDL))).toEqual(IDL);
    });

    it('ignores unused account capacity after the data', async () => {
        expect(await decodeAnchorIdlAccount(idlAccountBase64(IDL, {extraCapacity: 4096}))).toEqual(IDL);
    });

    it('rejects data shorter than its declared length', async () => {
        await expect(decodeAnchorIdlAccount(idlAccountBase64(IDL, {length: 10_000_000})))
            .rejects.toThrow(/truncated/);
    });

    it('rejects an account too short to hold the header', async () => {
        await expect(decodeAnchorIdlAccount(Buffer.alloc(20).toString('base64'))).rejects.toThrow(/too short/);
    });
});

describe('fetchAnchorIdl', () => {
    it('reads the IDL account with one getAccountInfo call', async () => {
        const {fetchImpl, requests} = rpcStub({
            result: {context: {slot: 1}, value: {data: [idlAccountBase64(IDL), 'base64'], owner: CPMM}}
        });

        expect(await fetchAnchorIdl(RPC, CPMM, {fetchImpl})).toEqual(IDL);
        expect(requests).toHaveLength(1);
        expect(requests[0].url).toBe(RPC);
        expect(requests[0].method).toBe('POST');
        expect(requests[0].body.method).toBe('getAccountInfo');
        expect(requests[0].body.params).toEqual([ANCHOR_IDL_ADDRESSES[CPMM], {encoding: 'base64'}]);
    });

    it('returns null when the program has no IDL account', async () => {
        const {fetchImpl} = rpcStub({result: {context: {slot: 1}, value: null}});
        expect(await fetchAnchorIdl(RPC, CPMM, {fetchImpl})).toBeNull();
    });

    it('throws an RPC error', async () => {
        const {fetchImpl} = rpcStub({error: {code: -32005, message: 'Too many requests'}});
        await expect(fetchAnchorIdl(RPC, CPMM, {fetchImpl})).rejects.toThrow('Too many requests');
    });

    it('throws on an HTTP error status', async () => {
        const {fetchImpl} = rpcStub({}, {status: 503});
        await expect(fetchAnchorIdl(RPC, CPMM, {fetchImpl})).rejects.toThrow(/503/);
    });
});
