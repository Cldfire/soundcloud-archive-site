import { writable } from 'svelte/store';

export const signedIn = writable(false);
export const userId = writable(-1);
