<script context="module" lang="ts">
    export const ID = "Screenshots";
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api";
    import { flip } from "svelte/animate";

    import ScreenshotRow from "./ScreenshotRow.svelte";

    function test() {
        console.log('here', this);
    }

    function sortScreenshotMap(screenshotMap) {
        let original = Object.entries(screenshotMap);
        let sorted = new Map(Array.from(original).sort(([a], [b]) => a.localeCompare(b)));
        console.log(sorted);
        return [...sorted];
    }
</script>

<div class="images">
    {#await invoke("get_screenshots") then screenshotMap}
        {#each sortScreenshotMap(screenshotMap) as [key, value]}
            <ScreenshotRow key={key} value={value} on:click={test}/>
        {/each}
    {/await}
</div>
<svelte:body on:contextmenu|preventDefault/>


<style>
    .images {
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100%;
    }
</style>
