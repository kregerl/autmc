<script lang="ts">
    import { invoke } from "@tauri-apps/api";
    
    import VirtualList from "../components/virtuallist/VirtualList.svelte";
    import Loader from "../components/Loader.svelte";
    import ScreenshotRow from "./ScreenshotRow.svelte";
    import { screenshotStore } from "../store/screenshotstore";


    async function getScreenshots(): Promise<Map<string, string[]>> {
        if ($screenshotStore === undefined) $screenshotStore = new Map();

        for (let [key, value] of Object.entries(await invoke("get_screenshots"))) {
            // Sort and reverse are done in-place
            value.sort((a, b) => a.localeCompare(b, "en", { numeric: true }));
            value.reverse();
            $screenshotStore.set(key, value);
        }

        return $screenshotStore;
    }
</script>

<div>
    {#await getScreenshots()}
        <Loader />
    {:then screenshots}
    <VirtualList items={[...screenshots]} let:item>
        <ScreenshotRow instance={item[0]} screenshots={item[1]} />
    </VirtualList>
    {/await}
</div>

<style>
    div {
        margin-left: 8px;
        grid-area: var(--grid-area);
        overflow: hidden;
    }
</style>
