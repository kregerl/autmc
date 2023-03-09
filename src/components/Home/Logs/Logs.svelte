<script context="module" lang="ts">
    export const ID = "Logs";
</script>

<script lang="ts">
    // TODO: Color warning and error logs different colors.
    import { afterUpdate, onMount } from "svelte";
    import { writableMap } from "../../../logstore";
    import DropdownMenu from "./DropdownMenu.svelte";

    // Parent scrollable div
    export let element: HTMLDivElement;
    let selected: string;
    let selectedLog: string;

    $: if (selected || selectedLog) {
        updateLogs($writableMap);
    }

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

    // TODO: Push and pop error colors so the error description can also be red.
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
    function getOptions(): string[] {
        console.log("getOptions: selected", selected);
        if (selected === undefined) {
            selected = getInstanceOptions()[0];
        }
        let logOptions = [...$writableMap.get(selected).keys()];
        logs = [...$writableMap.get(selected).get(logOptions[0])];
        return logOptions;
    }

    function updateLogs(value: Map<string, Map<string, string[]>>) {
        if (
            selected !== undefined &&
            selectedLog !== undefined &&
            value.get(selected.trim()) !== undefined
        ) {
            logs = value.get(selected.trim()).get(selectedLog.trim());
        }
    }

    writableMap.subscribe((value) => {
        updateLogs(value);
        console.log("Updated");
    });

    onMount(() => {
        updateLogs($writableMap);
    });
</script>

<div class="wrapper">
    <div class="header">
        <DropdownMenu options={getInstanceOptions()} bind:selected />
        <DropdownMenu options={getOptions()} bind:selected={selectedLog} />
    </div>
    <ul>
        {#if selected && logs}
            {#each logs as line}
                <li class={getClassForLine(line)}>{line}</li>
            {/each}
        {/if}
    </ul>
</div>

<style>
    .wrapper {
        height: calc(100vh - 60px);
    }

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
        height: 100%;
        padding-left: 0;
        margin: 0;
        list-style: none;
        overflow-y: scroll;
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
