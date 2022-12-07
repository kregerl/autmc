<script context="module" lang="ts">
    import type Tab from "./Tab.svelte";

    interface TabMetadata {
        text: string;
        component: typeof Tab;
    }
</script>

<script lang="ts">
    import { onMount } from "svelte";

    export let tabs: TabMetadata[];
    export let selected: string;


    function updateSelection(tabId: string, color: string): void {
        let element = document.getElementById(tabId.toLowerCase());
        selected = element.id;
        element.style.backgroundColor = color;
    }

    function updateStyle(tab: string, color: string): void {
        let element = document.getElementById(tab.toLowerCase());
        element.style.backgroundColor = color;
        element.style.boxShadow = "";
    }

    function setSelected(): void {
        console.log("Clicked", this);
        updateSelection(this.id, "#4e4e4e");
        for (let i = 0; i < tabs.length; i++) {
            let tab = tabs[i];
            if (selected === tab.text.toLowerCase())
                continue;
            updateStyle(tab.text, "#333");
        }
    }

    function tabKeyDown() {
        // TODO: Implement "Enter" updating selection of tabs.
    }

    onMount(() => {
        if (tabs.length > 0) {
            updateSelection(tabs[0].text, "#4e4e4e");
        }
    });
</script>

<div class="tab-bar flex-row">
    {#each tabs as tab}
        <svelte:component this={tab.component} {...tab} on:click={setSelected} on:keydown={tabKeyDown}/>
    {/each}
</div>

<style>
    .tab-bar {
        width: 100%; 
        height: 100%;
        background-color: #333;
        justify-content: start;
    }
</style>