const { useAsyncState, init } = require('../src/index');

describe('useAsyncState', () => {
    test('should return initial state', () => {
        const state = useAsyncState(() => Promise.resolve('ok'));
        expect(state).toHaveProperty('data', null);
        expect(state).toHaveProperty('loading', false);
        expect(state).toHaveProperty('error', null);
        expect(typeof state.run).toBe('function');
    });

    test('run should execute promise and update state', async () => {
        const fn = jest.fn().mockResolvedValue({ id: 1, name: 'test' });
        const state = useAsyncState(fn);
        const result = await state.run();
        expect(fn).toHaveBeenCalled();
        expect(result).toEqual({ id: 1, name: 'test' });
    });

    test('should handle errors gracefully', async () => {
        const err = new Error('Network failure');
        const fn = jest.fn().mockRejectedValue(err);
        const state = useAsyncState(fn);
        await state.run();
        expect(state.error).toBe(err);
        expect(state.data).toBeNull();
    });
});

describe('init', () => {
    test('should return ready state', () => {
        const result = init();
        expect(result.ready).toBe(true);
        expect(result.version).toBeDefined();
    });
});
