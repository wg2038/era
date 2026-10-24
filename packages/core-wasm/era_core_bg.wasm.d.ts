/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const __wbg_wasmsimulation_free: (a: number, b: number) => void;
export const init_panic_hook: () => void;
export const kernel_version: () => [number, number];
export const run_monte_carlo: (a: number, b: number) => [number, number, number];
export const run_monte_carlo_all_decades: (a: number) => [number, number, number];
export const wasmsimulation_get_state: (a: number) => [number, number, number];
export const wasmsimulation_interact_friend: (a: number, b: number) => [number, number, number];
export const wasmsimulation_interact_relative: (a: number, b: number) => [number, number, number];
export const wasmsimulation_make_choice: (a: number, b: number, c: number) => [number, number, number];
export const wasmsimulation_new: (a: bigint, b: number) => number;
export const wasmsimulation_run_to_end: (a: number) => [number, number, number];
export const wasmsimulation_step_year: (a: number) => [number, number, number];
export const wasmsimulation_step_year_auto: (a: number) => [number, number, number];
export const wasmsimulation_switch_to_peer: (a: number, b: number) => number;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_start: () => void;
