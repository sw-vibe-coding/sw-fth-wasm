/* tslint:disable */
/* eslint-disable */

export class Machine {
    free(): void;
    [Symbol.dispose](): void;
    eval_repl(line: string): void;
    get_dictionary_text(): string;
    get_history_text(): string;
    get_memory_text(): string;
    get_output_text(): string;
    get_stack_text(): string;
    get_trace_text(): string;
    load_source(src: string): void;
    constructor();
    reset(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_machine_free: (a: number, b: number) => void;
    readonly machine_eval_repl: (a: number, b: number, c: number) => void;
    readonly machine_get_dictionary_text: (a: number) => [number, number];
    readonly machine_get_history_text: (a: number) => [number, number];
    readonly machine_get_memory_text: (a: number) => [number, number];
    readonly machine_get_output_text: (a: number) => [number, number];
    readonly machine_get_stack_text: (a: number) => [number, number];
    readonly machine_get_trace_text: (a: number) => [number, number];
    readonly machine_load_source: (a: number, b: number, c: number) => void;
    readonly machine_new: () => number;
    readonly machine_reset: (a: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
