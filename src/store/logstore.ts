import { writable } from "svelte/store";

// <Instance name, <Logfile name, Lines in logfile>>
export const logStore = writable(new Map<string, Map<string, string[]>>());