<script lang="ts">
    import { path } from "@tauri-apps/api";
    import { open } from "@tauri-apps/api/dialog";
    import { UnlistenFn, listen } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";

    export let zipPath: string | undefined;

    let unlistener: UnlistenFn;

    onMount(async () => {
        unlistener = await listen("tauri://file-drop", (event) => {
            zipPath = event.payload[0];
        });
    });

    onDestroy(() => unlistener());

    async function browse() {
        const selected = await open({
            multiple: false,
            directory: false,
            filters: [
                {
                    name: "Modpack Archive",
                    extensions: ["mrpack", "zip"],
                },
            ],
            title: "Select Modpack Archive",
        });
        zipPath = selected as string;
    }
</script>

<div class="wrapper">
    <div class="upload">
        <h3 class="high-emphasis">Drop modpack file here</h3>
        <h3 class="high-emphasis or">or</h3>

        <div class="button-wrapper flex-row">
            <div class="browse" on:click={browse} on:keydown>
                <span class="high-emphasis">Browse...</span>
            </div>
            {#if zipPath}
                {#await path.basename(zipPath) then name}
                    <p class="high-emphasis">Importing <span>{name}</span></p>
                {/await}
            {:else}
                <p class="low-emphasis">No file selected.</p>
            {/if}
        </div>
    </div>
</div>

<style>
    .wrapper {
        display: flex;
        justify-content: center;
        align-items: center;
        margin-top: 64px;
        width: 100%;
        height: 100%;
    }

    .upload {
        display: flex;
        width: 60%;
        height: 70%;
        border: 2px dashed var(--light-blue);
        flex-direction: column;
        align-items: center;
        justify-content: center;
    }

    .or {
        margin-top: 0;
    }

    p {
        margin-left: 12px;
        color: white;
        font-size: 1.4rem;
    }
    p > span {
        font-weight: bold;
    }

    h3 {
        color: white;
        font-size: 2.4rem;
    }

    .button-wrapper {
        width: 40%;
        border-radius: 8px;
        border: 2px solid var(--lightest-black);
    }

    .browse > span {
        color: white;
        font-size: 1.6rem;
    }

    .browse {
        padding: 8px;
        margin: 4px;
        border-radius: 8px;
        width: fit-content;
        background-color: var(--dark-purple);
        cursor: pointer;
    }
</style>
