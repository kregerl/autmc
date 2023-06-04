<script lang="ts">
    import { invoke } from "@tauri-apps/api";
    import LogsHeader from "./LogsHeader.svelte";
    import Loader from "../components/Loader.svelte";
    import VirtualList from "../components/virtuallist/VirtualList.svelte";

    let selectedInstance: string;
    let selectedLog: string;

    interface TaggedLine {
        line: string;
        lineType: string;
    }

    // TODO: Cache lines once theyre already loaded once.
    async function retrieveLogLines(
        selectedInstance: string,
        selectedLog: string
    ): Promise<TaggedLine[]> {
        return await invoke("read_log_lines", {
            instanceName: selectedInstance,
            logName: selectedLog,
        });
    }
</script>

<LogsHeader --grid-area="header" bind:selectedInstance bind:selectedLog />
<div>
    {#if !selectedInstance || !selectedLog}
        <h1 class="high-emphasis">No Logs</h1>
    {:else}
        {#await retrieveLogLines(selectedInstance, selectedLog)}
            <Loader />
        {:then lines}
            {#if lines.length == 0}
                <h1 class="high-emphasis">No lines in {selectedLog}</h1>
            {:else}
                <VirtualList items={lines} let:item>
                    <p class="{item.lineType} high-emphasis">
                        {item.line}
                    </p>
                </VirtualList>
            {/if}
        {/await}
    {/if}
</div>

<style>
    div {
        margin-left: 8px;
        grid-area: var(--grid-area);
        overflow: hidden;
    }

    p {
        font-size: 1.8rem;
        color: white;
        word-wrap: break-word;
        white-space: pre-wrap;
    }

    h1 {
        color: white;
    }

    .error {
        color: red;
    }

    .warning {
        color: yellow;
    }
</style>
