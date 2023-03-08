<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { appConfigDir, join } from "@tauri-apps/api/path";
    import { fly, TransitionConfig } from "svelte/transition";
    import ViewImageModal from "../Modal/ViewImageModal/ViewImageModal.svelte";

    export let key;
    export let value;

    $: console.log("Created with key: ", key);

    async function getPath(
        instanceName: string,
        screenshotName: string
    ): Promise<string> {
        return convertFileSrc(
            await join(
                await appConfigDir(),
                `instances/${instanceName}/screenshots/${screenshotName}`
            )
        );
    }
    let hidden = false;
    function hideElements(_e: MouseEvent) {
        hidden = !hidden;
    }

    function conditionalFly(elemnt, args): TransitionConfig {
        if (hidden && elemnt !== undefined) {
            console.log("animate");
            return fly(elemnt, args);
        }
    }

    let lastTarget;
    let showImageModal = false;
    function openImageModal() {
        showImageModal = true;
        lastTarget = this.src;
    }
</script>

<div class="flex-row header-wrapper">
    <label>
        <input
            type="checkbox"
            id="checkbox"
            class="checkbox {key}"
            on:click={hideElements}
        />
        <img id="caret" src="svg/Caret.svg" alt="caret" />
    </label>

    <h3>{key}</h3>
</div>
{#if !hidden}
    <div out:conditionalFly={{ y: -10, duration: 300 }} class="images-row">
        {#each value as screenshot}
            {#await getPath(key, screenshot) then path}
                <img
                    src={path}
                    alt={screenshot}
                    on:click={openImageModal}
                    on:keydown
                />
            {/await}
        {/each}
    </div>
{/if}

{#if showImageModal && lastTarget !== undefined}
    <ViewImageModal
        image={lastTarget}
        on:close={() => (showImageModal = false)}
    />
{/if}

<style>
    .images-row {
        display: grid;
        grid-template-columns: repeat(5, minmax(0, 1fr));
        gap: 8px;
        height: auto;
        width: auto;
        margin-left: 8px;
        margin-right: 8px;
    }

    .images-row > img {
        max-width: 100%;
        height: auto;
        object-fit: cover;
    }

    .images-row > img:hover {
        opacity: 0.7;
        cursor: pointer;
    }

    h3 {
        color: white;
        margin: 0px;
        margin-bottom: 2px;
        margin-left: 8px;
    }

    .header-wrapper {
        align-items: center;
        margin-left: 8px;
        margin-top: 10px;
    }

    #checkbox {
        display: none;
    }

    #caret {
        margin-top: auto;
        justify-self: center;
        transition: all 0.25s;
        transform: rotate(90deg);
        z-index: 1;
    }

    #checkbox:checked + #caret {
        transform: rotate(0);
    }
</style>
