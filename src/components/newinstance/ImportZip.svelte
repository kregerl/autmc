<script lang="ts">
    import { listen } from "@tauri-apps/api/event";

    export let zipPath: string | undefined;

    listen("tauri://file-drop", (event) => {
        zipPath = event.payload[0];
    });
</script>

<div class="wrapper">
    <div class="upload">
        {#if zipPath}
            <h3 class="high-emphasis">Importing {zipPath}</h3>
        {:else}
            <h3 class="high-emphasis">Drag and drop a modpack file here.</h3>
        {/if}
    </div>
</div>

<style>
    .wrapper {
        display: flex;
        justify-content: center;
        margin-top: 64px;
    }

    .upload { 
        display: flex;
        width: 50vw;
        height: 60vh;
        border: 2px dashed var(--light-blue);
        justify-content: center;
        align-content: center;
        flex-direction: column;
        text-align: center;
    }

    h3 {
        color: white;
        font-size: 2.4rem;
    }
</style>