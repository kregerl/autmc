<script lang="ts">
    import VirtualList from "../virtual-list/VirtualList.svelte";
    import VirtualListRow from "../virtual-list/VirtualListRow.svelte";
    import type {
        VersionEntry,
        VersionManifest,
    } from "../../store/manifeststore";
    import { ModloaderType } from "../../menu";
    import CheckboxInput from "../input/CheckboxInput.svelte";

    // Input
    export let versionManifest: VersionManifest;

    // Outputs
    export let selectedVanillaVersion: string =
        versionManifest.vanilla_versions.at(0).version;
    export let selectedModloaderVersion: string = "";
    export let modloaderType: ModloaderType = ModloaderType.None;

    let vanillaScrollToIndex;

    interface Filter {
        id: string;
        name: string;
        checked: boolean;
    }

    const FILTERS: Filter[] = [
        { id: "release", name: "Releases", checked: true },
        { id: "snapshot", name: "Snapshots", checked: false },
        { id: "old_beta", name: "Betas", checked: false },
        { id: "old_alpha", name: "Alphas", checked: false },
    ];

    // Filter vanilla versions based on the checked filters.
    // Must pass the manifest and filters here sicne the reactive var relys on it to update.
    $: filteredVanillaVersions = getFilteredVanillaVersions(
        versionManifest.vanilla_versions,
        FILTERS
    );

    $: tryScroll(filteredVanillaVersions);

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
        console.log(
            filteredVersions.find(
                (versionEntry) =>
                    versionEntry.version === selectedVanillaVersion
            )
        );
        let found = filteredVersions.find(
            (versionEntry) => versionEntry.version === selectedVanillaVersion
        );
        if (filteredVersions.length > 0 && found === undefined)
            selectedVanillaVersion = filteredVersions.at(0).version;
        console.log("Updating filtered versions");
        return filteredVersions;
    }

    // Scrolls to the selected element in the vanilla versions list
    function tryScroll(versions: VersionEntry[]) {
        setTimeout(() => {
            if (vanillaScrollToIndex) {
                let x = versions.find(
                    (element) => element.version === selectedVanillaVersion
                );
                let index = versions.indexOf(x);
                vanillaScrollToIndex(index);
            }
        }, 100);
    }

    // Filter forge versions based on the vanilla version
    $: filteredForgeVersions =
        versionManifest.forge_versions.get(selectedVanillaVersion) ?? [];

    // On change of vanilla version on forge tabs, make sure to update the selection
    $: if (selectedVanillaVersion == "" && modloaderType === ModloaderType.Forge) {
        selectedModloaderVersion = filteredForgeVersions.at(0);
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

    interface Pair<T> {
        entry: T;
        index: number;
    }

    function withIndex<T>(list: T[]): Pair<T>[] {
        return list.map((element, index) => {
            return { entry: element, index: index };
        });
    }
</script>

<div class="grid-container">
    <div class="vanilla-versions">
        {#if filteredVanillaVersions.length == 0}
            <p class="medium-emphasis">No Versions Matching Filters</p>
        {:else}
            <div class="header flex-row">
                <span class="high-emphasis">Version</span>
                <span class="high-emphasis">Release Type</span>
                <span class="high-emphasis">Released Date</span>
            </div>
            <VirtualList
                items={withIndex(filteredVanillaVersions)}
                bind:scrollToIndex={vanillaScrollToIndex}
                let:item
            >
                <VirtualListRow
                    id={item.entry.version}
                    index={item.index}
                    items={[
                        item.entry.version,
                        item.entry.versionType,
                        formatDate(item.entry.releasedDate),
                    ]}
                    on:click={setSelectedVanillaVersion}
                    selected={selectedVanillaVersion}
                />
            </VirtualList>
        {/if}
    </div>
    <div class="modloader-versions">
        <div class="modloader-button-wrapper flex-row">
            <div
                class="high-emphasis button
                {modloaderType === ModloaderType.None ? 'selected' : ''}"
                on:click={() => setModloaderType(ModloaderType.None)}
                on:keydown
            >
                None
            </div>
            <div
                class="high-emphasis button
                {modloaderType === ModloaderType.Fabric ? 'selected' : ''}"
                on:click={() => setModloaderType(ModloaderType.Fabric)}
                on:keydown
            >
                Fabric
            </div>
            <div
                class="high-emphasis button
                {modloaderType === ModloaderType.Forge ? 'selected' : ''}"
                on:click={() => setModloaderType(ModloaderType.Forge)}
                on:keydown
            >
                Forge
            </div>
        </div>
        {#if filteredVanillaVersions.length > 0 && modloaderType == ModloaderType.Fabric}
            <div class="header flex-row">
                <span class="high-emphasis">Version</span>
            </div>
            <VirtualList
                items={withIndex(versionManifest.fabric_versions)}
                let:item
            >
                <VirtualListRow
                    id={item.entry}
                    index={item.index}
                    items={[item.entry]}
                    on:click={setSelectedModloaderVersion}
                    selected={selectedModloaderVersion}
                />
            </VirtualList>
        {:else if filteredVanillaVersions.length > 0 && modloaderType == ModloaderType.Forge}
            <div class="header flex-row">
                <span class="high-emphasis">Version</span>
            </div>
            {#if filteredForgeVersions.length == 0}
                <p class="medium-emphasis">
                    No Forge Versions for {selectedVanillaVersion}
                </p>
            {:else}
                <VirtualList items={withIndex(filteredForgeVersions)} let:item>
                    <VirtualListRow
                        id={item.entry}
                        index={item.index}
                        items={[item.entry]}
                        on:click={setSelectedModloaderVersion}
                        selected={selectedModloaderVersion}
                    />
                </VirtualList>
            {/if}
        {:else}
            <p class="medium-emphasis">No Modloader Selected</p>
        {/if}
    </div>
    <div class="filters flex-col">
        <span class="high-emphasis">Version Filters</span>
        {#each FILTERS as filter}
            <CheckboxInput text={filter.name} bind:checked={filter.checked}/>
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
        overflow-y: hidden;
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

    .filters > span {
        margin: 0;
        padding: 4px 0 4px 0;
        height: 32px;
        line-height: 32px;
        font-size: 2.2vmin;
        font-weight: bold;
        text-align: center;
        background-color: var(--dark-black);
    }

    .selected {
        background-color: var(--medium-purple) !important;
    }

    .button {
        text-align: center;
        width: 100%;
        height: 32px;
        line-height: 32px;
        padding: 4px 0 4px 0;
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

    .header {
        height: 32px;
        line-height: 32px;
        padding: 4px 0 4px 0;
        background-color: var(--dark-black);
        color: white;
    }

    .header > span {
        font-size: 2.2vmin;
        font-weight: bold;
        margin: 0px;
        width: 33%;
    }
</style>
