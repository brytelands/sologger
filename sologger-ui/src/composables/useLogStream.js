import {reactive} from 'vue';

/**
 * Remembers the most recent `limit` signatures so a transaction is handled once even
 * when several subscriptions report it (it mentions more than one monitored program).
 * Memory stays bounded: past the limit, the oldest signature is forgotten.
 */
export function createSignatureDeduper(limit = 10_000) {
    const seen = new Set();
    const ring = new Array(limit);
    let next = 0;
    return {
        isNew(signature) {
            if (seen.has(signature)) return false;
            if (ring[next] !== undefined) seen.delete(ring[next]);
            ring[next] = signature;
            next = (next + 1) % limit;
            seen.add(signature);
            return true;
        }
    };
}

/**
 * Live `logsSubscribe` streams, one WebSocket per monitored program.
 *
 * - Each transaction reaches `onTransaction({signature, slot, err, logs})` once, however
 *   many of the subscriptions report it.
 * - A connection that drops is reopened and resubscribed with exponential backoff
 *   (`backoff.initialMs` doubling up to `backoff.maxMs`), reset once a subscription is
 *   confirmed again. A subscription the RPC rejects, or a URL the browser refuses, is not
 *   retried.
 * - `disconnect` / `disconnectAll` close sockets and cancel pending reconnects; call
 *   `disconnectAll` when the owning component unmounts.
 *
 * Per-program status (`statusOf`, reactive): 'connecting' until the subscription is
 * confirmed, 'awaiting' until its first notification, then 'connected'; 'reconnecting'
 * while waiting to retry; 'disconnected' otherwise.
 *
 * `onEvent` receives {type: 'connected' | 'reconnected', programId},
 * {type: 'reconnecting', programId, attempt, delayMs} and
 * {type: 'rejected', programId, message}, for user-facing notices.
 */
export function createLogStream({
    WebSocketImpl,
    onTransaction = () => {},
    onEvent = () => {},
    commitment = 'finalized',
    backoff: {initialMs = 1000, maxMs = 30_000} = {},
    dedupeLimit = 10_000
} = {}) {
    const statuses = reactive(new Map());
    const connections = new Map();
    const dedupe = createSignatureDeduper(dedupeLimit);
    let nextRequestId = 1;

    const isCurrent = (programId, conn, socket) =>
        connections.get(programId) === conn && conn.socket === socket;

    function giveUp(programId, message) {
        connections.delete(programId);
        statuses.delete(programId);
        onEvent({type: 'rejected', programId, message});
    }

    function open(programId, conn) {
        const Impl = WebSocketImpl ?? globalThis.WebSocket;
        let socket;
        try {
            socket = new Impl(conn.url);
        } catch (error) {
            giveUp(programId, error.message);
            return;
        }
        conn.socket = socket;
        conn.requestId = nextRequestId++;
        statuses.set(programId, conn.attempt > 0 ? 'reconnecting' : 'connecting');

        socket.onopen = () => {
            if (!isCurrent(programId, conn, socket)) return;
            socket.send(JSON.stringify({
                jsonrpc: '2.0',
                id: conn.requestId,
                method: 'logsSubscribe',
                params: [{mentions: [programId]}, {commitment, encoding: 'json'}]
            }));
        };
        socket.onmessage = event => {
            if (isCurrent(programId, conn, socket)) handleMessage(programId, conn, socket, event);
        };
        socket.onclose = () => {
            if (isCurrent(programId, conn, socket)) scheduleReconnect(programId, conn);
        };
        // An error is always followed by a close, which decides what happens next
        socket.onerror = () => {};
    }

    function handleMessage(programId, conn, socket, event) {
        let message;
        try {
            message = JSON.parse(event.data);
        } catch {
            return;
        }

        if (message.id === conn.requestId) {
            if (message.error) {
                giveUp(programId, message.error.message ?? 'Subscription rejected');
                socket.close();
                return;
            }
            const recovered = conn.attempt > 0;
            conn.attempt = 0;
            statuses.set(programId, 'awaiting');
            onEvent({type: recovered ? 'reconnected' : 'connected', programId});
            return;
        }

        if (message.method !== 'logsNotification') return;
        const result = message.params?.result;
        const value = result?.value;
        if (!value?.signature) return;
        if (statuses.get(programId) !== 'connected') statuses.set(programId, 'connected');
        if (!dedupe.isNew(value.signature)) return;
        onTransaction({
            signature: value.signature,
            slot: result.context?.slot,
            err: value.err ?? null,
            logs: value.logs ?? []
        });
    }

    function scheduleReconnect(programId, conn) {
        conn.socket = null;
        conn.attempt += 1;
        const delayMs = Math.min(initialMs * 2 ** (conn.attempt - 1), maxMs);
        statuses.set(programId, 'reconnecting');
        onEvent({type: 'reconnecting', programId, attempt: conn.attempt, delayMs});
        conn.timer = setTimeout(() => {
            conn.timer = null;
            if (connections.get(programId) === conn) open(programId, conn);
        }, delayMs);
    }

    function connect(url, programId) {
        if (connections.has(programId)) return;
        const conn = {url, socket: null, attempt: 0, timer: null, requestId: null};
        connections.set(programId, conn);
        open(programId, conn);
    }

    function disconnect(programId) {
        const conn = connections.get(programId);
        connections.delete(programId);
        statuses.delete(programId);
        if (!conn) return;
        clearTimeout(conn.timer);
        conn.socket?.close();
    }

    function disconnectAll() {
        for (const programId of [...connections.keys()]) disconnect(programId);
    }

    return {
        connect,
        disconnect,
        disconnectAll,
        statusOf: programId => statuses.get(programId) ?? 'disconnected',
        /** Programs with an open socket or a pending reconnect. */
        activePrograms: () => [...connections.keys()]
    };
}
