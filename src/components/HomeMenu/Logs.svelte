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
</script>

<ul>
    {#each lines as line}
        <li>{line}</li>
    {/each}
</ul>


<style>
    ul {
        color: white;
        height: 100%;
        padding-left: 0;
        margin: 0;
        list-style: none;
    }

    ul > li {
        padding: 8px;
        font-size: 1.25rem;
        background-color: #444;
    }
</style>
