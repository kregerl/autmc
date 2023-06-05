import { writable } from "svelte/store";

function createStore() {
    // <Instance name, screenshot path src urls>
    const { subscribe, set, update } = writable(new Map<string, string[]>);
    return {
        subscribe,
        set,
        sort: () => update(map => sortMap(map))
    };
}

function sortMap(map: Map<string, string[]>): Map<string, string[]> {
    let sorted = new Map<string, string[]>();
    for (const [key, values] of Object.entries(map)) {
        sorted.set(key, (values as string[]).sort().reverse());
    }
    return sorted;
}

export const screenshotStore = createStore();