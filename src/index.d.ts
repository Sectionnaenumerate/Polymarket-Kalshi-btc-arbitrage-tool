export interface AsyncState<T = any> {
    data: T | null;
    loading: boolean;
    error: Error | null;
    run: (...args: any[]) => Promise<T>;
}

export interface AsyncStateOptions {
    immediate?: boolean;
}

export declare function useAsyncState<T = any>(
    promiseFn: (...args: any[]) => Promise<T>,
    options?: AsyncStateOptions
): AsyncState<T>;

export declare function init(config?: Record<string, any>): { ready: boolean; version: string };
