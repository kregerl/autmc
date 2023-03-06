<script context="module" lang="ts">
    export const ID = "Screenshots";
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api";

    import ScreenshotRow from "./ScreenshotRow.svelte";

    function test() {
        console.log('here', this);
    }
</script>

<div class="images">
    {#await invoke("get_screenshots") then screenshotMap}
        {#each Object.entries(screenshotMap) as [key, value]}
            <ScreenshotRow key={key} value={value} on:click={test}/>
        {/each}
    {/await}
</div>

<style>
    .images {
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100%;
    }
</style>
