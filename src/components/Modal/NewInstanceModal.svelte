<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import TabBar from "../Tabbar/TabBar.svelte";
    import Tab from "../Tabbar/Tab.svelte";
    import { invoke } from "@tauri-apps/api/tauri";
    
    let modal;
    let selected;
    interface VersionEntry {
        version: string,
        releasedDate: string,
        versionType: string,
    }
    interface VersionManifest {
        vanilla_versions: VersionEntry[],
        fabric_versions: string[]
    }

    let tabs = [
        {text: "Vanilla", component: Tab},
        {text: "Forge", component: Tab},
        {text: "Fabric", component: Tab},
        {text: "Curseforge", component: Tab}
    ];

    let vanillaVersions: {[key: string]: VersionEntry};
    let fabricVersions: string[];
    let selectedVersion: string;
    $: instanceName = "Minecraft " + selectedVersion;

    interface Filter {
        id: string,
        name: string,
        checked: boolean,
    }

    let filters: Filter[] = [
        {id: "release", name: "Releases", checked: true},
        {id: "snapshot", name: "Snapshots", checked: false},
        {id: "old_beta", name: "Betas", checked: false},
        {id: "old_alpha", name: "Alphas", checked: false}
    ]

    const dispatch = createEventDispatcher();

    function close() {
        dispatch("close");
    }

    function setActive() {
        for (let i = 0; i < Object.keys(vanillaVersions).length; i++) {
            let versionId = Object.values(vanillaVersions)[i].version;
            let element = document.getElementById(versionId) as HTMLElement;
            if (element.classList.contains("selected"))
                element.classList.remove("selected");
        }
        this.classList.add("selected");
        selectedVersion = this.id;
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key == "Escape") {
            close();
            return;
        } else if (event.key == "Tab") {
            // Trap "tab" key focus 
            const nodes = modal.querySelectorAll('*') as HTMLElement[];
			const tabbable = Array.from(nodes).filter(n => n.tabIndex >= 0);
            
            if (tabbable.length > 0) {
                let index = tabbable.indexOf(document.activeElement as HTMLElement);
                if (index === -1 && event.shiftKey) index = 0;

                index += tabbable.length + (event.shiftKey ? -1 : 1);
                index %= tabbable.length;

                tabbable[index].focus();
            }
            event.preventDefault();
        }
    }

    function tabKeyDown() {
        // TODO: Implement "Enter" updating selection of tabs.
    }

    function finish() {
        invoke("obtain_version", { selected: selectedVersion, instanceName: instanceName });
        close();
    }

    // TODO: Do filtering on web side since itll be faster to just filter out ones that arent selected then to await a promise from rust. 
    $: promise = getVersions(filters);
    
    async function getInstancePath() {
        let instancePath = await invoke("get_instance_path");
        return instancePath + '/' + instanceName;
    }

    //FIXME: Make this work with forge and fabric. Move to onMount and do filtering on frontend instead of rust
    async function getVersions(filters) {
        let manifest: VersionManifest = await invoke("obtain_manifests", { filters: filters });
        let entries: VersionEntry[] = manifest.vanilla_versions;
        vanillaVersions = Object.fromEntries(entries.map(x => [x.version, x]));
        fabricVersions = manifest.fabric_versions;
        selectedVersion = entries[0].version;
        return vanillaVersions; 
    }
</script>

<svelte:window on:keydown={handleKeyDown}/>

<div class="background" on:keydown={tabKeyDown}></div>

<div class="modal" role="dialog" aria-modal="true" bind:this={modal}>
    <input type="image" src="PlusSign.svg" alt="Close Instance Creation" on:click={close}>
    <h1 class="modal-header">New Instance</h1>
    <div class="version-options">
        {#if selected !== undefined && selected === "vanilla"}
            <table>
                <thead>
                    <tr>
                        <th>Version</th>
                        <th>Type</th>
                        <th>Released</th>
                    </tr>
                </thead>
                <tbody>
                    {#await promise} 
                        <h1>Loading...</h1>
                    {:then versions}
                        {#each Object.entries(versions) as version}
                            <tr id={version[0]} class={selectedVersion === version[0] ? "selected" : ""} on:click={setActive}>
                                <td>{version[0]}</td>
                                <td class="version-type">{version[1].versionType}</td>
                                <td class="release-date">{version[1].releasedDate}</td>
                            </tr>
                        {/each}
                    {:catch error}
                        <h1>Error: {error}</h1>
                    {/await}
                </tbody>
            </table>
        {:else if selected !== undefined && selected === "fabric"}
            <table>
                <thead>
                    <tr>
                        <th>Version</th>
                        <th>Type</th>
                    </tr>
                </thead>
                <tbody>
                    {#each fabricVersions as fabricVersion}
                    <!-- Fabric versions here is the same as ingame version. -->
                        <tr id={fabricVersion}}>
                            <td>{fabricVersion}</td>
                            <td>Empty</td>
                        </tr>
                    {/each}
                </tbody>
            </table>    
        {/if}
    </div>
   
    <div class="instance-name">
        <input type="text" id="name" placeholder="Instance Name" bind:value={instanceName}>
        {#await getInstancePath() then path}
            <p class="location">{path}</p>
        {/await}
    </div>

    <div class="modal-content">
        {#each filters as filter}
            <label class="dropshadow checkbox-label" for={filter.name}> 
                <input class="filter" type="checkbox" id={filter.name} bind:checked={filter.checked}>
                {filter.name}
            </label>
        {/each}
    </div>

    <div class="version-tabs">
        <TabBar --width=100% tabs={tabs} bind:selected/>
    </div>

    <div class="modal-footer">
        <!-- TODO: Onclick submit the instance to be created. -->
        <button class="dropshadow" on:click={finish}>Done</button>
    </div>

</div>

<style>
    tr.selected {
        background-color: red !important;
    }

    .background {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(51, 51, 51, 0.5);
    }

    .modal {
        display: grid; 
        grid-template-columns: 1.6fr 1.2fr; 
        grid-template-rows: 0.5fr 0.5fr 2.6fr 0.5fr; 
        gap: 0px 0px; 
        grid-template-areas: 
        "modal-header modal-header"
        "version-tabs instance-name"
        "version-options modal-content"
        "modal-footer modal-footer"; 
        position: absolute;
        left: 50%;
        top: 50%;
        width: calc(100vw - 24%);
        height: calc(100vh - 24%);
        background-color: #333;
        transform: translate(-50%, -50%);
        border-radius: 1em;
    }

    .modal-header {
        grid-area: modal-header;
        text-align: center;
        margin-top: 8px;
        margin-bottom: 0px;
        color: white;
    }

    .version-tabs {
        grid-area: version-tabs;
        margin-left: 8px;
    }

    .version-options {
        grid-area: version-options;
        overflow-y: scroll;
        height: 100%;
        color: white;
        margin-left: 8px;
    }

    .version-options thead th {
        position: sticky;
        top: 0;
        background-color: #4e4e4e;
        height: 40px;
    }

    .version-options table {
        border-collapse: collapse;
        table-layout: fixed;
        width: 100%;
    }

    .version-options th {
        padding: 8px;
    }

    .version-options tr {
        z-index: 1;
    }

    .version-options th, td {
        padding: 8px 15px;
        text-align: left;
    }

    .version-options tbody > tr:hover {
        background-color: #5E5E5E;
    }

    .version-options tbody > tr:active {
        background-color: #6E6E6E;
    }

    .instance-name > input[type=text] {
        border: none;
        margin: 8px;
    }

    .instance-name {
        grid-area: instance-name;
        color: white;
    }

    .instance-name > p {
        margin: 8px;
        margin-top: 0px;
        font-size: 0.75rem;
    }

    .modal-content {
        grid-area: modal-content;
        margin: 8px;
        margin-top: 0px;
        background-color: #4E4E4E;
    }

    .modal-content > label {
        color: white;
        font-size: 1rem;
        cursor: pointer;
    }

    .filter {
        margin-top: 8px;
        margin-left: 8px;
        width: 1.5rem;
        height: 1rem;
        cursor: pointer;
    }

    .modal-footer {
        grid-area: modal-footer;
    }

    .modal-footer > button {
        float:right;
        width: 80px;
        height: 50px;
        font-size: 1.2rem;
        margin: 12px;
        border-radius: 8px;
        border: none;
        background-color: #4E4E4E;
        color: white;
    }

    .modal-footer > button:hover {
        background-color: #5E5E5E;
    }

    .modal-footer > button:active {
        background-color: #6E6E6E;
    }

    input[type=image] {
        position: absolute;
        right: 0;
        height: 4em;
        margin: 4px;
        transform: rotate(45deg);
        border: none;
    }

</style>