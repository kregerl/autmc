<script lang="ts">
    import { onMount } from "svelte";

    export let selected;

    function updateSelection(element: HTMLElement, color: string): void {
        selected = element;
        selected.style.backgroundColor = color;
        selected.style.boxShadow = "0px 8px 4px -4px rgba(108, 73, 245, 0.5)";
    }

    function setSelected(): void {
        let tabs = document.getElementsByClassName("tab");
        let selectedTab = document.getElementById(this.id) as HTMLElement;
        updateSelection(selectedTab, "#4e4e4e");
        for (let i = 0; i < tabs.length; i++) {
            let tab = tabs[i] as HTMLElement;
            if (selected === tab)
                continue;
            tab.style.backgroundColor = "#333"
            tab.style.boxShadow = "";
        }
    }

    function tabKeyDown() {
        // TODO: Implement "Enter" updating selection of tabs.
    }

    onMount(() => {
        let tabs = document.getElementsByClassName("tab");
        if (tabs.length > 0) {
            updateSelection(tabs[0] as HTMLElement,  "#4e4e4e");
        }
    });
</script>


<div class="tab-bar flex-row">
    <div class="tab" id="instances" on:click={setSelected} on:keydown={tabKeyDown}>
        <h3 class="dropshadow">Instances</h3>
    </div>
    <div class="tab" id="log" on:click={setSelected} on:keydown={tabKeyDown}>
        <h3 class="dropshadow">Log</h3>
    </div>
</div>

<style>
    .tab-bar {
        width: 100%; 
        height: 100%;
        z-index: 100;
        background-color: #333;
        box-shadow: 0px 8px 4px -4px rgba(0, 0, 0, 0.65);
    }
    .tab {
        padding: 8px;
        min-width: 180px;
        background-color: #333;
        text-align: center;
    }

    h3 {
        color: white;
        font-size: 1.25em;
    }
</style>