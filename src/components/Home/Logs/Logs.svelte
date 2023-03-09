<script context="module" lang="ts">
    export const ID = "Logs";
</script>

<script lang="ts">
    // TODO: Color warning and error logs different colors.
    import { afterUpdate, onMount } from "svelte";
    import { LogInformation, writableMap } from "../../../logstore";
    import Login from "../../Login.svelte";
    import DropdownMenu from "./DropdownMenu.svelte";

    // Parent scrollable div
    export let element: HTMLDivElement;
    let selected: string;
    $:console.log("selected", selected)

    $: if (element) {
        scrollToBottom();
    }

    afterUpdate(() => {
        scrollToBottom();
    });

    function scrollToBottom() {
        element.scroll({ top: element.scrollHeight, behavior: "smooth" });
    }

    function isError(line) {
        return line.includes("/ERROR]:");
    }

    function isWarning(line) {
        return line.includes("/WARN]:");
    }

    function getClassForLine(line) {
        if (isError(line)) {
            return "error";
        } else if (isWarning(line)) {
            return "warn";
        }
        return "";
    }

    function getInstanceOptions(): string[] {
        return [...$writableMap.keys()];
    }
    
    let logs: string[];
    function updateLogs(value: Map<string, string[]>) {
        if (selected !== undefined) {
            logs = value.get(selected.trim())
        }
    }
    
    writableMap.subscribe(value => {
        updateLogs(value);
    });
    
    onMount(() => {
        updateLogs($writableMap);
    });

</script>

<div class="header">
    <DropdownMenu options={getInstanceOptions()} bind:selected />
</div>
<ul>
    {#if selected && logs}
        {#each logs as line}
            <li class={getClassForLine(line)}>{line}</li>
        {/each}
    {/if}
</ul>

<style>
    .header {
        display: flex;
        align-items: center;
        position: sticky;
        top: 0;
        background-image: linear-gradient(#444, #333);
        width: 100%;
        height: 60px;
        /* border: 2px solid red; */
    }

    ul {
        color: white;
        height: auto;
        padding-left: 0;
        margin: 0;
        list-style: none;
    }

    ul > li {
        padding: 8px;
        font-size: 1.25rem;
        background-color: #444;
    }

    li.error {
        color: rgb(160, 0, 0);
    }

    li.warn {
        color: rgb(211, 197, 0);
    }
</style>
