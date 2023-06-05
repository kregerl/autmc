import { writable } from "svelte/store";

export const instanceStore = writable<InstanceConfiguration[]>();

export interface InstanceConfiguration {
    instance_name: string;
    jvm_path: string;
    arguments: string;
    modloader_type: string;
    modloader_version: string;
}