<script context="module" lang="ts">
    export const ID = "Logs";
</script>

<script lang="ts">
    // TODO: Color warning and error logs different colors.
    import { afterUpdate } from "svelte";

    export let lines = [];
    // Parent scrollable div
    export let element: HTMLDivElement;

    $: if (lines && element) {
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
</script>

<ul>
    {#each lines as line}
        {#if isError(line)}
            <li class="error">{line}</li>
        {:else if isWarning(line)}
            <li class="warn">{line}</li>
        {:else}
            <li>{line}</li>
        {/if}
    {/each}
</ul>


<style>
    ul {
        color: white;
        height: 100vh;
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
