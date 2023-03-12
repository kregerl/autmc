<script context="module" lang="ts">
    export const ID = "Logs";
</script>

<script lang="ts">
    import { afterUpdate, onMount } from "svelte";
    import { logStore } from "../../../logstore";
    import DropdownMenu from "./DropdownMenu.svelte";

    // Parent scrollable div
    let element: HTMLUListElement;
    let selectedInstance: string;
    let selectedLog: string;

    $: if (selectedInstance) {
        resetSelectedLog();
    }

    function resetSelectedLog() {
        selectedLog = $logStore.get(selectedInstance).keys().next().value;
    }

    $: if (selectedLog) {
        updateLogs();
    }

    $: if (element) {
        scrollToBottom();
    }

    $: logOptions = [...$logStore.get(selectedInstance).keys()];
    $: instanceOptions = [...$logStore.keys()];

    afterUpdate(() => {
        scrollToBottom();
    });

    function scrollToBottom() {
        if (selectedLog === "running")
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
        logs = $logStore.get(selectedInstance).get(selectedLog);
    }

    logStore.subscribe((value) => {
        console.log("Value: ", value);
        if (selectedInstance === undefined) {
            selectedInstance = value.keys().next().value;
            console.log("selectedInstance", selectedInstance);
        }
        if (selectedLog === undefined) {
            selectedLog = value.get(selectedInstance).keys().next().value;
            console.log("selectedLog", selectedInstance);
        }
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
    <ul bind:this={element}>
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
        word-wrap: break-word;
    }

    li.error {
        color: rgb(160, 0, 0);
    }

    li.warn {
        color: rgb(211, 197, 0);
    }
</style>
