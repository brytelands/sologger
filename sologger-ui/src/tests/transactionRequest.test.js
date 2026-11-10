import {describe, expect, it} from 'vitest';
import {getTransactionRequestBody} from '../composables/useTransactionRequest';

describe('getTransactionRequestBody', () => {
    it('requests the given signature with getTransaction', () => {
        const request = JSON.parse(getTransactionRequestBody('SIG123'));

        expect(request.method).toBe('getTransaction');
        expect(request.params[0]).toBe('SIG123');
    });

    it('accepts v1 transactions as the JSON integer 1', () => {
        const [, config] = JSON.parse(getTransactionRequestBody('SIG123')).params;

        expect(config.maxSupportedTransactionVersion).toBe(1);
    });
});
