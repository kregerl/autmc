import { writable } from "svelte/store";

export const logStore = writable<Map<string, string[]>>();