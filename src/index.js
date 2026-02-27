/**
 * Polymarket Kalshi Btc Arbitrage Tool SDK
 * Lightweight utility layer for async state management.
 * @module Polymarket-Kalshi-btc-arbitrage-tool
 */

'use strict';

/**
 * Creates an async state container.
 * @param {Function} promiseFn - A function that returns a Promise.
 * @param {Object} [options] - Configuration options.
 * @param {boolean} [options.immediate=false] - Whether to execute immediately.
 * @returns {Object} State container with data, loading, error, and run properties.
 */
function useAsyncState(promiseFn, options = {}) {
    const state = {
        data: null,
        loading: false,
        error: null,
        run: async (...args) => {
            state.loading = true;
            state.error = null;
            try {
                state.data = await promiseFn(...args);
            } catch (err) {
                state.error = err;
            } finally {
                state.loading = false;
            }
            return state.data;
        },
    };
    if (options.immediate) state.run();
    return state;
}

/**
 * Initializes the SDK with configuration.
 * @param {Object} config - SDK configuration.
 * @returns {{ ready: boolean, version: string }}
 */
function init(config = {}) {
    return { ready: true, version: '1.5.14', config };
}

module.exports = { useAsyncState, init };
