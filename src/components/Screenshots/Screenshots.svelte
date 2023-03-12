<script context="module" lang="ts">
    export const ID = "Screenshots";
</script>

<script lang="ts">
    import { screenshotStore } from "../../screenshotstore";

    import ScreenshotRow from "./ScreenshotRow.svelte";

    function test() {
        console.log('here', this);
    }

</script>

<div class="images">
    {#each [...$screenshotStore] as [key, value]}
        <ScreenshotRow key={key} value={value} on:click={test}/>
    {/each}
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
