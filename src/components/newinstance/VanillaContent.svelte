<script lang="ts">
    import VirtualTable from "svelte-virtual-table";
    import type {
        VersionEntry,
        VersionManifest,
    } from "../../store/manifeststore";
    import { ModloaderType } from "../../menu";

    // Input
    export let versionManifest: VersionManifest;

    // Outputs
    export let selectedVanillaVersion: string =
        versionManifest.vanilla_versions.at(0).version;
    export let selectedModloaderVersion: string = "";
    export let modloaderType: ModloaderType = ModloaderType.None;

    interface Filter {
        id: string;
        name: string;
        checked: boolean;
    }

    const FILTERS: Filter[] = [
        { id: "release", name: "Releases", checked: true },
        { id: "snapshot", name: "Snapshots", checked: true },
        { id: "old_beta", name: "Betas", checked: false },
        { id: "old_alpha", name: "Alphas", checked: false },
    ];

    // Filter vanilla versions based on the checked filters.
    // Must pass the manifest and filters here sicne the reactive var relys on it to update.
    $: filteredVanillaVersions = getFilteredVanillaVersions(
        versionManifest.vanilla_versions,
        FILTERS
    );

    function getFilteredVanillaVersions(
        versions: VersionEntry[],
        filers: Filter[]
    ): VersionEntry[] {
        let filteredVersions = versions.filter((version) => {
            for (let filter of filers) {
                if (version.versionType === filter.id && filter.checked)
                    return true;
            }
        });
        if (filteredVersions.length > 0)
            selectedVanillaVersion = filteredVersions.at(0).version;
        return filteredVersions;
    }

    // Filter forge versions based on the vanilla version
    $: filteredForgeVersions =
        versionManifest.forge_versions.get(selectedVanillaVersion) ?? [];

    // On change of vanilla version on forge tabs, make sure to update the selection
    $: if (selectedVanillaVersion) {
        selectedModloaderVersion = filteredForgeVersions.at(0);
    }

    $: x = test(selectedVanillaVersion);

    function test(version: string) {
        console.log(version);
    }

    function setSelectedVanillaVersion(_event: MouseEvent) {
        selectedVanillaVersion = this.id;
    }

    function setModloaderType(type: ModloaderType) {
        modloaderType = type;
        if (type === ModloaderType.Fabric) {
            selectedModloaderVersion = versionManifest.fabric_versions.at(0);
        } else if (type === ModloaderType.Forge) {
            selectedModloaderVersion = filteredForgeVersions.at(0);
        }
    }

    function setSelectedModloaderVersion(_event: MouseEvent) {
        selectedModloaderVersion = this.id;
    }

    function formatDate(releasedDate: string): string {
        return new Date(Date.parse(releasedDate)).toLocaleDateString("en-US", {
            year: "numeric",
            month: "long",
            day: "numeric",
        });
    }
</script>

<div class="grid-container">
    <div class="vanilla-versions">
        {#if filteredVanillaVersions.length == 0}
            <p class="medium-emphasis">No Versions Matching Filters</p>
        {:else}
            <VirtualTable items={filteredVanillaVersions} let:item>
                <tr slot="thead">
                    <th class="high-emphasis">Version</th>
                    <th class="high-emphasis">Release Type</th>
                    <th class="high-emphasis">Release Date</th>
                </tr>
                <tr
                    id={item.version}
                    class="data-row {selectedVanillaVersion === item.version
                        ? 'selected'
                        : ''}"
                    on:click={setSelectedVanillaVersion}
                    slot="tbody"
                >
                    <td class="high-emphasis">{item.version}</td>
                    <td class="high-emphasis">{item.versionType}</td>
                    <td class="high-emphasis"
                        >{formatDate(item.releasedDate)}</td
                    >
                </tr>
            </VirtualTable>
        {/if}
    </div>
    <div class="modloader-versions">
        <div class="modloader-button-wrapper flex-row">
            <div
                class="high-emphasis button {modloaderType ===
                ModloaderType.None
                    ? 'selected'
                    : ''}"
                on:click={() => setModloaderType(ModloaderType.None)}
                on:keydown
            >
                None
            </div>
            <div
                class="high-emphasis button {modloaderType ===
                ModloaderType.Fabric
                    ? 'selected'
                    : ''}"
                on:click={() => setModloaderType(ModloaderType.Fabric)}
                on:keydown
            >
                Fabric
            </div>
            <div
                class="high-emphasis button {modloaderType ===
                ModloaderType.Forge
                    ? 'selected'
                    : ''}"
                on:click={() => setModloaderType(ModloaderType.Forge)}
                on:keydown
            >
                Forge
            </div>
        </div>
        {#if filteredVanillaVersions.length > 0 && modloaderType == ModloaderType.Fabric}
            <VirtualTable items={versionManifest.fabric_versions} let:item>
                <tr slot="thead">
                    <th class="high-emphasis">Version</th>
                </tr>
                <tr
                    class="data-row {selectedModloaderVersion === item
                        ? 'selected'
                        : ''}"
                    id={item}
                    on:click={setSelectedModloaderVersion}
                    slot="tbody"
                >
                    <td class="high-emphasis">{item}</td>
                </tr>
            </VirtualTable>
        {:else if filteredVanillaVersions.length > 0 && modloaderType == ModloaderType.Forge}
            {#if filteredForgeVersions.length == 0}
                <p class="medium-emphasis">
                    No Forge Versions for {selectedVanillaVersion}
                </p>
            {:else}
                <VirtualTable items={filteredForgeVersions} let:item>
                    <tr slot="thead">
                        <th class="high-emphasis">Version</th>
                    </tr>
                    <tr
                        class="data-row {selectedModloaderVersion === item
                            ? 'selected'
                            : ''}"
                        id={item}
                        on:click={setSelectedModloaderVersion}
                        slot="tbody"
                    >
                        <td class="high-emphasis">{item}</td>
                    </tr>
                </VirtualTable>
            {/if}
        {:else}
            <p class="medium-emphasis">No Modloader Selected</p>
        {/if}
    </div>
    <div class="filters flex-col">
        <h3 class="high-emphasis">Version Filters</h3>
        {#each FILTERS as filter}
            <label for={filter.name} class="high-emphasis">
                <input
                    id={filter.name}
                    type="checkbox"
                    bind:checked={filter.checked}
                />
                {filter.name}
            </label>
        {/each}
    </div>
</div>

<style>
    .grid-container {
        display: grid;
        grid-template-columns: 1.1fr 1.1fr 0.8fr;
        grid-template-rows: 1fr;
        gap: 0px 8px;
        grid-template-areas: "vanilla modloader filters";
        width: 100%;
        height: 100%;
        background-color: var(--medium-black);
    }

    .vanilla-versions {
        grid-area: vanilla;
        /* -12px since thats the padding */
        width: calc(100% - 12px);
        overflow-y: scroll;
        background-color: var(--light-black);
        padding: 6px;
        border-radius: 4px;
    }

    .modloader-versions {
        grid-area: modloader;
        /* -12px since thats the padding */
        width: calc(100% - 12px);
        overflow-y: hidden;
        background-color: var(--light-black);
        padding: 6px;
        border-radius: 4px;
    }

    .filters {
        grid-area: filters;
        /* -12px since thats the padding */
        width: calc(100% - 12px);
        height: 50%;
        color: white;
        font-size: 1.8rem;
        background-color: var(--light-black);
        padding: 6px;
        border-radius: 4px;
    }

    .filters > h3 {
        margin: 0;
        padding: 4px;
        text-align: center;
        background-color: var(--dark-black);
    }

    .filters > label {
        margin-top: 4px;
        cursor: pointer;
        width: fit-content;
        padding: 4px;
    }

    .filters > label > input {
        width: 1.5rem;
        height: 1.5rem;
    }

    .filters > label:hover {
        background-color: var(--lightest-black);
    }

    th,
    td {
        color: white;
        padding: 4px 0 4px 0;
    }

    th {
        font-size: 2vmin;
    }

    td {
        font-size: 1.5vmin;
    }

    .data-row:nth-child(odd) {
        background-color: var(--medium-light-black);
    }

    .data-row:nth-child(even) {
        background-color: var(--medium-black);
    }

    .data-row {
        cursor: pointer;
    }

    .data-row:hover:not(.selected) {
        background-color: var(--lightest-black);
    }

    .selected {
        background-color: #573993 !important;
    }

    .button {
        text-align: center;
        width: 100%;
        height: 32px;
        line-height: 32px;
        font-size: 2vmin;
        font-weight: bold;
        cursor: pointer;
        color: white;
        background-color: var(--dark-black);
    }

    .modloader-button-wrapper {
        justify-content: space-between;
    }

    p {
        color: white;
        font-size: 2.4rem;
        text-align: center;
    }
</style>
