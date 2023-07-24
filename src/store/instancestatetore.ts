import { writable } from "svelte/store";

export const instanceStateStore = writable<Map<string, InstanceState>>(new Map());

export enum InstanceState {
    Initializing,
    Initialized,
}

export interface ExitCode {
    instanceName: string;
    code?: number;
}