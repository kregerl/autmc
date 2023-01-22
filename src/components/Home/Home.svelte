<script lang="ts">
    import {listen } from '@tauri-apps/api/event'
    import Menu from "./Menu/Menu.svelte";
    import Instances, {ID as INSTANCE_ID} from "./Instances.svelte";
    import Logs, {ID as LOGS_ID} from './Logs.svelte';
    import { onMount } from 'svelte';
    import type { VersionManifest } from '../../manifest';

    let selected;
    // Logs
    let lines;
    let element;

    // FIXME: Lines are always appended, if an instance is closed the logs should be cleared here.
    onMount(async () => {
        const unlisten = await listen("instance-logging", (event) => {
            lines = [...lines, event.payload];
            console.log("Event", event);
        });
    });
</script>

<div class="container">
    <div class="menu">
        <Menu bind:selectedTab={selected}/>
    </div>
    <div class="content" bind:this={element}>
        {#if selected === INSTANCE_ID}
            <Instances/>
        {:else if selected === LOGS_ID}
            {#if lines !== undefined && lines.length === 0}
            <div class="flex-row">
                <h1>No Logs</h1>
            </div>
            {:else}
                <Logs bind:lines bind:element/>
            {/if}
        {/if}
    </div>
</div>

<style>
    .container {
        display: grid; 
        grid-template-columns: 0.4fr 1.6fr; 
        gap: 0px 0px;
        grid-template-areas: 
        "menu content"
        "menu content"; 
    }

    .menu {
        grid-area: menu;
    }

    .content {
        grid-area: content;
        overflow-y: scroll;
        background-color: #333;
    }

    .content:has(div.flex-row) {
        display: flex;
        flex-direction: column;
        justify-content: center;
    }

    .content > .flex-row {
        justify-content: center;
    }

    .content > .flex-row > h1 {
        color: white;
        text-align: center;
    }

</style>
