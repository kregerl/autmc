<script lang="ts">
    import { isValidVersionForForge, manifestStore, VersionEntry, VersionManifest} from "../../../store/manifeststore";
    import { updateSelectionClasses } from "../../../selectable";
    import VanillaVersionTable from "./VanillaVersionTable.svelte";
    import VersionTable, { Row } from "./VersionTable.svelte";

    interface Filter {
        id: string;
        name: string;
        checked: boolean;
    }

    let filters: Filter[] = [
        { id: "release", name: "Releases", checked: true },
        { id: "snapshot", name: "Snapshots", checked: true },
        { id: "old_beta", name: "Betas", checked: false },
        { id: "old_alpha", name: "Alphas", checked: false },
    ];

    const buttons = ["None", "Fabric", "Forge"];

    export let selectedModloader: string = "None";

    export let selectedVanillaVersion;
    export let selectedModloaderVersion;

    $: if (
        selectedModloader === "Forge" &&
        selectedVanillaVersion !== undefined &&
        !isValidVersionForForge($manifestStore, selectedVanillaVersion)
    ) {
        console.log("Here: ", selectedVanillaVersion);
        selectedModloader = updateSelectionClasses("None", buttons);
    }
    $: modloaderFilters = getModloaderFilters(selectedModloader);

    // TODO: Forge filters
    function getModloaderFilters(modloader: string) {
        if (modloader === "Fabric") {
            return [{ id: "stable", name: "Stable", checked: true }];
        } else {
            return [];
        }
    }

    function updateModloaderSelection() {
        if (this.classList.contains("disabled")) return;
        selectedModloader = updateSelectionClasses(this.id, buttons);
    }

    function getBodyForModloaderTable(manifest: VersionManifest): Row[] {
        let rows: Row[] = [];
        if (selectedModloader === "Fabric") {
            for (let version of manifest.fabric_versions) {
                rows.push({ id: version, entries: [version] });
            }
        } else if (selectedModloader === "Forge") {
            if (manifest.forge_versions.has(selectedVanillaVersion)) {
                for (let version of manifest.forge_versions.get(
                    selectedVanillaVersion
                )) {
                    rows.push({ id: version, entries: [version] });
                }
            }
            rows.reverse();
        }
        if (rows.length > 0) selectedModloaderVersion = rows.at(0).id;
        return rows;
    }

    function getHeadersForModloaderTable() {
        return { id: "Version", entries: ["Version"] };
    }

    function applyVanillaVersionFilters(
        versions: VersionEntry[],
        filters: Filter[]
    ): VersionEntry[] {
        let filteredVersions: VersionEntry[] = [];
        for (let version of versions) {
            for (let filter of filters) {
                if (version.versionType === filter.id && filter.checked) {
                    filteredVersions.push(version);
                }
            }
        }
        if (
            filteredVersions.filter(
                (version) => version.version === selectedVanillaVersion
            ).length < 1
        ) {
            selectedVanillaVersion = filteredVersions.at(0).version;
        }
        return filteredVersions;
    }
</script>

<div class="outer">
    <div class="vanilla-version">
        <div class="vanilla-header">
            <h3>Vanilla</h3>
        </div>
        <VanillaVersionTable
            versionEntries={applyVanillaVersionFilters(
                $manifestStore.vanilla_versions,
                filters
            )}
            bind:selected={selectedVanillaVersion}
        />
    </div>

    <div class="modloader-version">
        <div class="tabbar flex-row">
            {#each buttons as button, i}
                <div
                    id={button}
                    class="tab menu-button {i === 0 ? 'selected' : ''}
                    {button === 'Forge' &&
                    !isValidVersionForForge($manifestStore, selectedVanillaVersion)
                        ? 'disabled'
                        : ''}"
                    on:click={updateModloaderSelection}
                    on:keypress
                >
                    <h3>{button}</h3>
                </div>
            {/each}
        </div>
        {#if selectedModloader !== "None"}
            {#if getBodyForModloaderTable($manifestStore).length < 1}
                <h3>Nothing</h3>
            {:else}
                <VersionTable
                    headers={getHeadersForModloaderTable()}
                    body={getBodyForModloaderTable($manifestStore)}
                    bind:selected={selectedModloaderVersion}
                />
            {/if}
        {:else}
            <h3>No Modloader Selected</h3>
        {/if}
    </div>

    <div class="filters">
        <h2>Filters</h2>
        <h4>Version Type Filters</h4>
        {#each filters as filter}
            <label class="dropshadow checkbox-label" for={filter.name}>
                <input
                    type="checkbox"
                    class="filter-checkbox"
                    id={filter.name}
                    bind:checked={filter.checked}
                />
                {filter.name}
            </label>
        {/each}
        {#if selectedModloader !== "None"}
            <h4>{selectedModloader} Filters</h4>
        {/if}
        {#each modloaderFilters as filter}
            <label class="dropshadow checkbox-label" for={filter.name}>
                <input
                    type="checkbox"
                    class="filter-checkbox"
                    id={filter.name}
                    bind:checked={filter.checked}
                />
                {filter.name}
            </label>
        {/each}
    </div>
</div>

<style>
    .outer {
        display: grid;
        grid-template-columns: 1.2fr 1.2fr 0.6fr;
        grid-template-rows: 1fr;
        gap: 0px 0px;
        grid-template-areas: "vanilla-version modloader-version filters";
        width: 54vw;
    }

    .vanilla-version {
        grid-area: vanilla-version;
    }

    .vanilla-header {
        height: 4vh;
        text-align: center;
        font-size: 1.25vw;
        background-color: #4e4e4e;
        vertical-align: middle;
    }

    .vanilla-header > h3 {
        margin: 0px;
        margin-bottom: 0px;
        padding-top: 6px;
        font-size: 2vh;
    }

    .modloader-version {
        grid-area: modloader-version;
        margin-left: 2px;
        margin-right: 2px;
    }

    .modloader-version > h3 {
        text-align: center;
    }

    .tabbar {
        height: 4vh;
        justify-content: center;
    }

    .tabbar > .tab {
        position: sticky;
        width: 50%;
        margin: 0;
        text-align: center;
        background-color: #4e4e4e;
    }

    .tab {
        text-align: center;
        vertical-align: center;
        cursor: pointer;
    }

    .tab > h3 {
        margin-bottom: 0px;
        margin-top: 6px;
        font-size: 2vh;
    }

    .filters {
        grid-area: filters;
        margin-left: 2px;
        background-color: #4e4e4e;
        height: fit-content;
    }

    .filters > h2 {
        text-align: center;
        margin-bottom: 6px;
    }

    .filters > h4 {
        text-align: center;
        margin-bottom: 6px;
    }

    .filters > label {
        font-size: 1vw;
        cursor: pointer;
        margin-left: 8px;
    }
</style>
