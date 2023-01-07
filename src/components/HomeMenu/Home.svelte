<script lang="ts">
    import {listen } from '@tauri-apps/api/event'
    import Menu from "./Menu.svelte";
    import Tab from "../Tabbar/Tab.svelte";
    import Instances from "./Instances.svelte";
    import TabBar from "../Tabbar/TabBar.svelte";
    import { onMount } from 'svelte';
    import Logs from './Logs.svelte';

    let selected;
    // Logs
    let lines;
    let element;

    const navTabs = [
        {text: "Instances", fs: "1.25em", component: Tab},
        {text: "Logs", fs: "1.25em",component: Tab},
    ];

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
        <Menu/>
    </div>
    <div class="header">
        <TabBar --min-width=180px --font-size=1.25em tabs={navTabs} bind:selected></TabBar>
    </div>
    <div class="content" bind:this={element}>
        {#if selected !== undefined && selected === "instances"}
            <Instances/>
        {:else if selected !== undefined && selected === "logs"}
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
        grid-template-columns: 350px auto; 
        grid-template-rows: 80px calc(100vh - 80px); 
        gap: 0px 0px;
        grid-template-areas: 
        "menu header"
        "menu content"; 
    }

    .menu {
        grid-area: menu;
    }

    .header {
        grid-area: header;
    }

    .content {
        grid-area: content;
        overflow-y: scroll;
        background-color: #444;
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
