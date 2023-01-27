import { invoke } from "@tauri-apps/api";

export interface VersionEntry {
    version: string;
    releasedDate: string;
    versionType: string;
}

export interface VersionManifest {
    vanilla_versions: VersionEntry[];
    fabric_versions: string[];
    forge_versions: Map<string, string[]>
}

let manifest: VersionManifest = undefined;

export async function getManifest(): Promise<VersionManifest> {
    if (manifest === undefined) {
        manifest = await invoke("obtain_manifests");
        // Convert to actual map so "has()" and "get()" can be used
        manifest.forge_versions = new Map(Object.entries(manifest.forge_versions));
    }
    return manifest;
}

export function isValidVersionForForge(vanillaVersion: string): boolean {
    return manifest !== undefined && manifest.forge_versions.has(vanillaVersion);
}