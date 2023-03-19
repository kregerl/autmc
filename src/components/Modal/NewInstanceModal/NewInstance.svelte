<script lang="ts">
    import { invoke, path } from "@tauri-apps/api";
    import { appConfigDir } from "@tauri-apps/api/path";
    import { updateSelectionClasses } from "../../../selectable";
    import { createEventDispatcher } from "svelte";
    import VanillaOptions from "./VanillaOptions.svelte";
    import { instanceStore } from "../../../store/instancestore";
    import ImportZip from "./ImportZip.svelte";

    let modal;
    const dispatch = createEventDispatcher();
    const buttons = ["Vanilla", "Curseforge", "Modrinth", "Import Zip"];
    let selectedInstanceType: string = buttons.at(0);

    let selectedVanillaVersion;
    let selectedModloaderVersion;
    let selectedModloader;

    let zipPath;

    $: instanceName =
        `Minecraft ` +
        (selectedModloader === "None" ? "" : selectedModloader + " ") +
        `${selectedVanillaVersion}`;
    $: instancePath = getInstancePath(instanceName);

    function close() {
        dispatch("close");
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key == "Escape") {
            close();
            return;
        } else if (event.key == "Tab") {
            // Trap "tab" key focus
            const nodes = modal.querySelectorAll("*") as HTMLElement[];
            const tabbable = Array.from(nodes).filter((n) => n.tabIndex >= 0);

            if (tabbable.length > 0) {
                let index = tabbable.indexOf(
                    document.activeElement as HTMLElement
                );
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

    function updateSelections() {
        selectedInstanceType = updateSelectionClasses(this.id, buttons);
    }

    async function getInstancePath(name: string): Promise<string> {
        const appDir = await appConfigDir();
        return path.join(appDir, "instances", name);
    }

    function next() {
        if (selectedInstanceType == "Vanilla") {
            console.log("Vanilla:", selectedVanillaVersion);
            console.log("Modloader Type:", selectedModloader);
            console.log("Modloader:", selectedModloaderVersion);
            invoke("obtain_version", {
                vanillaVersion: selectedVanillaVersion,
                modloaderType: selectedModloader,
                modloaderVersion: selectedModloaderVersion ?? "",
                instanceName: instanceName,
            });
        } else {
            // Import from zip.
            invoke("import_zip", {zipPath: zipPath});
        }

        // TODO: Make this go to another modal that has some jvm options.
        close();
    }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="background" on:keydown={tabKeyDown} />

<div class="modal" role="dialog" aria-modal="true" bind:this={modal}>
    <div class="header">
        <h1>Create New Instance</h1>
        <input
            type="image"
            src="svg/PlusSign.svg"
            alt="Close Instance Creation"
            on:click={close}
        />
    </div>
    <div class="version-options flex-column">
        <div class="buttons flex-row">
            {#each buttons as button, i}
                <button
                    id={button}
                    class="dropshadow {i === 0 ? 'selected' : ''}"
                    on:click={updateSelections}>{button}</button
                >
            {/each}
        </div>
        <div class="instance-name flex-column">
            <input
                type="text"
                placeholder="Instance Name"
                class={$instanceStore.includes(instanceName) ? "invalid" : ""}
                bind:value={instanceName}
            />
            {#await instancePath then path}
                <p>{path}</p>
            {/await}
        </div>
    </div>
    <div class="modal-content flex-column">
        {#if selectedInstanceType === "Vanilla"}
            <VanillaOptions
                bind:selectedModloader
                bind:selectedModloaderVersion
                bind:selectedVanillaVersion
            />
        {:else if selectedInstanceType === "Curseforge"}
            <h1>TODO</h1>
        {:else if selectedInstanceType === "Modrinth"}
            <h1>TODO</h1>
        {:else}
            <ImportZip bind:zipPath={zipPath}/>
        {/if}
    </div>
    <!--TODO: disable this button when the name already exists.  -->
    <button
        class="next-button"
        on:click={next}
        disabled={$instanceStore.includes(instanceName)}>Next</button
    >
</div>

<style>
    .modal-content {
        grid-area: modal-content;
        color: white;
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
        grid-template-columns: 1fr;
        grid-template-rows: 0.2fr 0.5fr 2.3fr;
        gap: 0px 0px;
        grid-template-areas:
            "header"
            "version-options"
            "modal-content";
        position: absolute;
        left: 50%;
        top: 50%;
        width: calc(100vw - 12%);
        height: calc(100vh - 12%);
        background-color: #333;
        transform: translate(-50%, -50%);
        border-radius: 1em;
        box-shadow: 5px 5px 16px 2px rgba(0, 0, 0, 0.75);
    }

    .header {
        grid-area: header;
        color: white;
        text-align: center;
    }

    input[type="image"] {
        position: absolute;
        top: 0;
        right: 0;
        height: 3vw;
        margin: 4px;
        transform: rotate(45deg);
        border: none;
    }

    .version-options {
        grid-area: version-options;
    }

    .flex-column {
        align-items: center;
    }

    .buttons {
        justify-content: center;
        width: 48vw;
    }

    button {
        flex-grow: 3;
    }

    input[type="text"] {
        width: 24vw;
        margin: 0px;
    }

    input[type="text"].invalid {
        border: 2px solid red;
    }

    .instance-name {
        color: white;
        align-items: start;
        width: 48vw;
        margin-top: 8px;
    }

    .instance-name > p {
        margin: 2px;
        font-size: 0.75vw;
    }

    .buttons {
        margin-bottom: 8px;
    }

    .buttons > button {
        border: none;
        color: white;
        border-radius: 0;
        background-color: #4e4e4e;
        cursor: pointer;
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

    .next-button:disabled {
        color: rgb(70, 70, 70);
        cursor: not-allowed;
    }

    .next-button:enabled:hover {
        background-color: #5e5e5e;
    }

    .next-button:enabled:active {
        background-color: #6e6e6e;
    }
</style>
