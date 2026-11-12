import {address, createAddressWithSeed, getProgramDerivedAddress} from '@solana/addresses';

// Anchor stores a program's IDL in an account at createWithSeed(PDA(no seeds), "anchor:idl"),
// owned by the program. Data: 8-byte discriminator, 32-byte authority, u32 LE length, then
// that many bytes of zlib-compressed JSON (the account may have unused capacity after it).
// This replaces Program.fetchIdl from @coral-xyz/anchor, which pulled in @solana/web3.js.
const IDL_SEED = 'anchor:idl';
const HEADER_BYTES = 8 + 32;

/** The address of a program's Anchor IDL account. Rejects an invalid program address. */
export async function anchorIdlAddress(programId) {
    const programAddress = address(programId);
    const [baseAddress] = await getProgramDerivedAddress({programAddress, seeds: []});
    return createAddressWithSeed({baseAddress, programAddress, seed: IDL_SEED});
}

/** Decodes an IDL account's base64 data into the IDL object. */
export async function decodeAnchorIdlAccount(base64Data) {
    const bytes = Uint8Array.from(atob(base64Data), char => char.charCodeAt(0));
    if (bytes.length < HEADER_BYTES + 4) {
        throw new Error('IDL account is too short to hold an IDL');
    }
    const length = new DataView(bytes.buffer, bytes.byteOffset + HEADER_BYTES, 4).getUint32(0, true);
    const compressed = bytes.subarray(HEADER_BYTES + 4, HEADER_BYTES + 4 + length);
    if (compressed.length !== length) {
        throw new Error('IDL account data is truncated');
    }
    // 'deflate' is the zlib format Anchor writes (pako.deflate)
    const inflated = new Blob([compressed]).stream().pipeThrough(new DecompressionStream('deflate'));
    return JSON.parse(await new Response(inflated).text());
}

/**
 * Fetches a program's Anchor IDL over JSON-RPC (one getAccountInfo call). Resolves to null
 * when the program has no IDL account; rejects on an invalid program address, an RPC or
 * HTTP error, or undecodable account data.
 */
export async function fetchAnchorIdl(rpcUrl, programId, {fetchImpl = fetch} = {}) {
    const idlAddress = await anchorIdlAddress(programId);
    const response = await fetchImpl(rpcUrl, {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: 'getAccountInfo',
            params: [idlAddress, {encoding: 'base64'}]
        })
    });
    if (!response.ok) {
        throw new Error(`RPC request failed with HTTP ${response.status}`);
    }
    const json = await response.json();
    if (json.error) {
        throw new Error(json.error.message ?? 'RPC error');
    }
    const account = json.result?.value;
    if (!account) return null;
    return decodeAnchorIdlAccount(account.data[0]);
}
