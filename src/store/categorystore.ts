import { writable } from "svelte/store";

export interface CurseforgeCategory {
    id: number;
    name: string;
    iconUrl: string,
}

export const categoryStore = writable<CurseforgeCategory[]>();