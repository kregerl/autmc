import { writable } from "svelte/store";

export const writableMap = writable(new Map<string, Map<string, string[]>>());