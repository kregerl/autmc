import { writable } from "svelte/store";

export interface VersionEntry {
    version: string;
    releasedDate: string;
    versionType: string;
}

export interface VersionManifest {
    vanilla_versions: VersionEntry[];
    fabric_versions: string[];
    forge_versions: Map<string, string[]>;
}

export function isValidVersionForForge(manifest: VersionManifest, vanillaVersion: string): boolean {
    return new Map(Object.entries(manifest.forge_versions)).has(vanillaVersion);
} 

export const manifestStore = writable<VersionManifest>();