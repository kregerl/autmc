<script context="module" lang="ts">
    export const ID = "Logs";
</script>

<script lang="ts">
    import { afterUpdate, onMount } from "svelte";
    import { writableMap } from "../../../logstore";
    import DropdownMenu from "./DropdownMenu.svelte";

    // Parent scrollable div
    export let element: HTMLDivElement;
    let selectedInstance: string;
    let selectedLog: string;

    $: if (selectedInstance) {
        console.log("Updated: ", $writableMap.get(selectedInstance).keys())
        resetSelectedLog();
    }

    function resetSelectedLog() {
        selectedLog = $writableMap.get(selectedInstance).keys().next().value;
    }

    $: if (selectedLog) {
        updateLogs();
    }

    $: if (element) {
        scrollToBottom();
    }

    $: logOptions = [...$writableMap.get(selectedInstance).keys()];
    $: instanceOptions = [...$writableMap.keys()];

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

    // TODO: Push and pop error colors so the error description can also be colored.
    function getClassForLine(line) {
        if (isError(line)) {
            return "error";
        } else if (isWarning(line)) {
            return "warn";
        }
        return "";
    }

    let logs: string[];
    function updateLogs() {
        logs = $writableMap.get(selectedInstance).get(selectedLog);
    }

    writableMap.subscribe((value) => {
        console.log("value", value)
        selectedInstance = value.keys().next().value;
        console.log("selectedInstance", selectedInstance);
        selectedLog = value.get(selectedInstance).keys().next().value;
        updateLogs();
    });
</script>

<div class="wrapper">
    <div class="header">
        <DropdownMenu
            options={instanceOptions}
            bind:selected={selectedInstance}
        />
        <DropdownMenu options={logOptions} bind:selected={selectedLog} />
    </div>
    <ul>
        {#each logs as line}
            <li class={getClassForLine(line)}>{line}</li>
        {/each}
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
    }

    ul {
        color: white;
        height: 100%;
        padding-left: 0;
        margin: 0;
        list-style: none;
        overflow-y: scroll;
        overflow-x: hidden;
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
