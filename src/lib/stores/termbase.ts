// src/lib/stores/termbase.ts
import { writable } from 'svelte/store';
import type { TermEntry, TbInfo } from '$lib/ipc/termbase';

export const termMatches = writable<TermEntry[]>([]);
export const tbLoading = writable<boolean>(false);
export const availableTbs = writable<TbInfo[]>([]);
