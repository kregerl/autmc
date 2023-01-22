<script lang="ts">
    import { getManifest } from "../../manifest";
    import { updateSelectionClasses } from "../../selectable";
    import VanillaVersionTable from "./VanillaVersionTable.svelte";
    import VersionTable from "./VersionTable.svelte";

    interface Filter {
        id: string;
        name: string;
        checked: boolean;
    }

    let filters: Filter[] = [
        { id: "release", name: "Releases", checked: true },
        { id: "snapshot", name: "Snapshots", checked: false },
        { id: "old_beta", name: "Betas", checked: false },
        { id: "old_alpha", name: "Alphas", checked: false },
    ];

    const buttons = ["None", "Fabric", "Forge"];

    let selectedModloader: string = "None";

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
        selectedModloader = updateSelectionClasses(this.id, buttons);
    }
</script>

<div class="outer">
    <div class="vanilla-version">
        {#await getManifest() then manifest}
            <VanillaVersionTable versionEntries={manifest.vanilla_versions} />
        {/await}
    </div>

    <div class="modloader-version">
        <div class="tabbar flex-row">
            {#each buttons as button, i}
                <div
                    id={button}
                    class="tab menu-button {i === 0 ? 'selected' : ''}"
                    on:click={updateModloaderSelection}
                    on:keypress
                >
                    <h3>{button}</h3>
                </div>
            {/each}
        </div>
        {#if selectedModloader !== "None"}
        <!-- FIXME: Wrap in a div so the overflow can work correctly.  -->
            {#await getManifest() then manifest}
                <VersionTable
                    --header-height="4vh"
                    --font-size="2vh"
                    headers={["Version"]}
                    body={manifest.fabric_versions}
                />
            {/await}
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
        <h4>{selectedModloader} Filters</h4>
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
<button class="next-button dropshadow" on:click={close}>Next</button>

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
        margin-right: 2px;
        max-height: 58vh;
        overflow: scroll;
    }

    .modloader-version {
        grid-area: modloader-version;
        margin-left: 2px;
        margin-right: 2px;
        max-height: 58vh;
        overflow: scroll;
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

    .next-button {
        position: absolute;
        bottom: 0;
        right: 0;
        margin: 12px;
        width: 8vw;
        height: 6vh;
        font-size: 1vw;
        border-radius: 8px;
        text-align: center;
        vertical-align: middle;
        color: white;
        background-color: #4e4e4e;
        border: none;
    }
</style>
