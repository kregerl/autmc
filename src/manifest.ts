import { invoke } from "@tauri-apps/api";

export interface VersionEntry {
    version: string;
    releasedDate: string;
    versionType: string;
}
export interface VersionManifest {
    vanilla_versions: VersionEntry[];
    fabric_versions: string[];
}

let manifest: VersionManifest = undefined;

export async function getManifest(): Promise<VersionManifest> {
    if (manifest === undefined) {
        manifest = await invoke("obtain_manifests");
    }
    return manifest;
}