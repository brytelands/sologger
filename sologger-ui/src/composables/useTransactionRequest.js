/** JSON-RPC request body for fetching one transaction's logs by signature. */
export function getTransactionRequestBody(signature) {
    return JSON.stringify({
        jsonrpc: '2.0', id: 1, method: 'getTransaction',
        params: [signature, {encoding: 'json', maxSupportedTransactionVersion: 1}]
    });
}
