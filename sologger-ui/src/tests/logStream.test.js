import {afterEach, beforeEach, describe, expect, it, vi} from 'vitest';
import {createLogStream, createSignatureDeduper} from '../composables/useLogStream';

const URL = 'wss://rpc.example';
const TOKEN = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
const ATA = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL';

/** Just enough of the browser WebSocket for the stream; tests drive it by hand. */
class FakeSocket {
    static instances = [];

    constructor(url) {
        this.url = url;
        this.sent = [];
        this.closed = false;
        FakeSocket.instances.push(this);
    }

    send(data) {
        this.sent.push(JSON.parse(data));
    }

    close() {
        if (this.closed) return;
        this.closed = true;
        this.onclose?.({});
    }

    // Server side
    open() {
        this.onopen?.({});
    }

    confirmSubscription(subscriptionId = 7) {
        this.onmessage?.({data: JSON.stringify({jsonrpc: '2.0', id: this.sent[0].id, result: subscriptionId})});
    }

    rejectSubscription(message) {
        this.onmessage?.({
            data: JSON.stringify({jsonrpc: '2.0', id: this.sent[0].id, error: {code: -32602, message}})
        });
    }

    notify(signature, {slot = 42, err = null, logs = ['Program log: hi']} = {}) {
        this.onmessage?.({
            data: JSON.stringify({
                jsonrpc: '2.0',
                method: 'logsNotification',
                params: {subscription: 7, result: {context: {slot}, value: {signature, err, logs}}}
            })
        });
    }

    drop() {
        this.closed = true;
        this.onclose?.({});
    }
}

function setup(options = {}) {
    const transactions = [];
    const events = [];
    const stream = createLogStream({
        WebSocketImpl: FakeSocket,
        onTransaction: tx => transactions.push(tx),
        onEvent: event => events.push(event),
        ...options
    });
    return {stream, transactions, events};
}

/** Opens and confirms the newest socket, as a healthy RPC would. */
function subscribeLatest() {
    const socket = FakeSocket.instances.at(-1);
    socket.open();
    socket.confirmSubscription();
    return socket;
}

beforeEach(() => {
    FakeSocket.instances = [];
    vi.useFakeTimers();
});

afterEach(() => {
    vi.useRealTimers();
});

describe('createLogStream', () => {
    it('subscribes to the program\'s logs on open', () => {
        const {stream} = setup({commitment: 'confirmed'});
        stream.connect(URL, TOKEN);
        const socket = FakeSocket.instances[0];
        socket.open();

        expect(socket.url).toBe(URL);
        expect(socket.sent).toHaveLength(1);
        expect(socket.sent[0].method).toBe('logsSubscribe');
        expect(socket.sent[0].params).toEqual([{mentions: [TOKEN]}, {commitment: 'confirmed', encoding: 'json'}]);
    });

    it('delivers a transaction once even when two subscriptions report it', () => {
        const {stream, transactions} = setup();
        stream.connect(URL, TOKEN);
        const token = subscribeLatest();
        stream.connect(URL, ATA);
        const ata = subscribeLatest();

        token.notify('SIG1');
        ata.notify('SIG1');
        ata.notify('SIG2');

        expect(transactions.map(tx => tx.signature)).toEqual(['SIG1', 'SIG2']);
        expect(transactions[0]).toEqual({signature: 'SIG1', slot: 42, err: null, logs: ['Program log: hi']});
    });

    it('reports per-program status: connecting, awaiting, connected', () => {
        const {stream} = setup();
        stream.connect(URL, TOKEN);
        stream.connect(URL, ATA);
        expect(stream.statusOf(TOKEN)).toBe('connecting');

        const [token, ata] = FakeSocket.instances;
        token.open();
        expect(stream.statusOf(TOKEN)).toBe('connecting'); // subscribe sent, not yet confirmed
        token.confirmSubscription();
        ata.open();
        ata.confirmSubscription();
        expect(stream.statusOf(TOKEN)).toBe('awaiting');

        token.notify('SIG1');
        expect(stream.statusOf(TOKEN)).toBe('connected');
        expect(stream.statusOf(ATA)).toBe('awaiting'); // its own subscription has seen nothing yet
        expect(stream.statusOf('unknown')).toBe('disconnected');
    });

    it('reconnects a dropped connection with growing backoff and resubscribes', () => {
        const {stream, events} = setup({backoff: {initialMs: 1000, maxMs: 4000}});
        stream.connect(URL, TOKEN);
        subscribeLatest().drop();

        expect(stream.statusOf(TOKEN)).toBe('reconnecting');
        expect(events.at(-1)).toMatchObject({type: 'reconnecting', programId: TOKEN, attempt: 1, delayMs: 1000});
        vi.advanceTimersByTime(999);
        expect(FakeSocket.instances).toHaveLength(1);
        vi.advanceTimersByTime(1);
        expect(FakeSocket.instances).toHaveLength(2);

        // The retry fails before subscribing: wait longer, up to the cap
        FakeSocket.instances[1].drop();
        expect(events.at(-1)).toMatchObject({type: 'reconnecting', attempt: 2, delayMs: 2000});
        vi.advanceTimersByTime(2000);
        FakeSocket.instances[2].drop();
        expect(events.at(-1)).toMatchObject({attempt: 3, delayMs: 4000});
        vi.advanceTimersByTime(4000);
        FakeSocket.instances[3].drop();
        expect(events.at(-1)).toMatchObject({attempt: 4, delayMs: 4000});
        vi.advanceTimersByTime(4000);

        const socket = subscribeLatest();
        expect(socket.sent[0].params[0]).toEqual({mentions: [TOKEN]});
        expect(stream.statusOf(TOKEN)).toBe('awaiting');
        expect(events.at(-1)).toMatchObject({type: 'reconnected', programId: TOKEN});

        // A successful subscription resets the backoff
        socket.drop();
        expect(events.at(-1)).toMatchObject({type: 'reconnecting', attempt: 1, delayMs: 1000});
    });

    it('does not reconnect a program that was disconnected on purpose', () => {
        const {stream} = setup();
        stream.connect(URL, TOKEN);
        subscribeLatest();

        stream.disconnect(TOKEN);
        vi.advanceTimersByTime(60_000);

        expect(FakeSocket.instances).toHaveLength(1);
        expect(stream.statusOf(TOKEN)).toBe('disconnected');
    });

    it('cancels a pending reconnect on disconnect', () => {
        const {stream} = setup();
        stream.connect(URL, TOKEN);
        subscribeLatest().drop();
        expect(stream.statusOf(TOKEN)).toBe('reconnecting');

        stream.disconnect(TOKEN);
        vi.advanceTimersByTime(60_000);

        expect(FakeSocket.instances).toHaveLength(1);
        expect(stream.statusOf(TOKEN)).toBe('disconnected');
    });

    it('disconnectAll closes every socket and cancels every pending reconnect', () => {
        const {stream} = setup();
        stream.connect(URL, TOKEN);
        const token = subscribeLatest();
        stream.connect(URL, ATA);
        subscribeLatest().drop();

        stream.disconnectAll();
        vi.advanceTimersByTime(60_000);

        expect(token.closed).toBe(true);
        expect(FakeSocket.instances).toHaveLength(2);
        expect(stream.statusOf(TOKEN)).toBe('disconnected');
        expect(stream.statusOf(ATA)).toBe('disconnected');
    });

    it('gives up on a subscription the RPC rejects instead of retrying', () => {
        const {stream, events} = setup();
        stream.connect(URL, 'not-a-pubkey');
        const socket = FakeSocket.instances[0];
        socket.open();
        socket.rejectSubscription('Invalid Request: Invalid pubkey provided');
        vi.advanceTimersByTime(60_000);

        expect(FakeSocket.instances).toHaveLength(1);
        expect(socket.closed).toBe(true);
        expect(stream.statusOf('not-a-pubkey')).toBe('disconnected');
        expect(events.at(-1)).toEqual({
            type: 'rejected',
            programId: 'not-a-pubkey',
            message: 'Invalid Request: Invalid pubkey provided'
        });
    });

    it('reports a URL the browser refuses as rejected instead of throwing', () => {
        class RefusingSocket {
            constructor() {
                throw new SyntaxError('The URL \'nope\' is invalid.');
            }
        }
        const {stream, events} = setup({WebSocketImpl: RefusingSocket});

        expect(() => stream.connect('nope', TOKEN)).not.toThrow();
        expect(stream.statusOf(TOKEN)).toBe('disconnected');
        expect(events.at(-1)).toEqual({type: 'rejected', programId: TOKEN, message: 'The URL \'nope\' is invalid.'});
    });

    it('ignores a second connect for a program that is already connected', () => {
        const {stream} = setup();
        stream.connect(URL, TOKEN);
        stream.connect(URL, TOKEN);
        expect(FakeSocket.instances).toHaveLength(1);
    });

    it('ignores messages that are not JSON', () => {
        const {stream, transactions} = setup();
        stream.connect(URL, TOKEN);
        const socket = subscribeLatest();
        expect(() => socket.onmessage({data: 'not json'})).not.toThrow();
        expect(transactions).toHaveLength(0);
    });
});

describe('createSignatureDeduper', () => {
    it('reports a signature as new only the first time', () => {
        const dedupe = createSignatureDeduper(10);
        expect(dedupe.isNew('A')).toBe(true);
        expect(dedupe.isNew('A')).toBe(false);
        expect(dedupe.isNew('B')).toBe(true);
    });

    it('forgets the oldest signatures past its limit', () => {
        const dedupe = createSignatureDeduper(2);
        dedupe.isNew('A');
        dedupe.isNew('B');
        dedupe.isNew('C'); // evicts A

        expect(dedupe.isNew('B')).toBe(false);
        expect(dedupe.isNew('A')).toBe(true);
    });
});
