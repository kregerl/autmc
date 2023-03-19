<script lang="ts">
    import { listen } from "@tauri-apps/api/event";

    export let zipPath;
    let dragAndDropEnabled = true;

    listen("tauri://file-drop", (event) => {
        if (dragAndDropEnabled) {
            zipPath = event.payload[0];
            dragAndDropEnabled = false;
        }
    });
</script>

<div>
    {#if dragAndDropEnabled}
        <h3>Drag a zip file here</h3>
    {:else}
        <h3>Importing {zipPath}</h3>
    {/if}
</div>

<style>
    div {
        display: flex;
        width: 50vw;
        height: 60vh;
        border: 2px solid black;
        justify-content: center;
        align-content: center;
        flex-direction: column;
        text-align: center;
    }
</style>
