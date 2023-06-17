import { writable } from "svelte/store";

export const canvasCache = writable<Map<string, HTMLCanvasElement>>();