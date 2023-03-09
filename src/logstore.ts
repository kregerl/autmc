import { writable } from "svelte/store";

export interface LogInformation {
    log_id: string;
    log_lines: string[];
}

export const writableMap = writable(new Map<string, string[]>());