<script lang="ts">
    import { invoke } from "@tauri-apps/api";
    
    import LogsHeader from "./LogsHeader.svelte";
    import Loader from "../components/Loader.svelte";
    import VirtualList from "../components/virtual-list/VirtualList.svelte";

    let selectedInstance: string;
    let selectedLog: string;
    let filter: string;

    interface TaggedLine {
        line: string;
        lineType: string;
    }

    // TODO: Cache lines once theyre already loaded once.
    async function retrieveLogLines(
        selectedInstance: string,
        selectedLog: string,
        filter: string
    ): Promise<TaggedLine[]> {
        console.log(selectedInstance, selectedLog);
        let lines: TaggedLine[] = await invoke("read_log_lines", {
            instanceName: selectedInstance,
            logName: selectedLog,
        });
        if (filter) {
            return lines.filter(line => line.line.includes(filter));
        } 
        return lines;
    }
</script>

<LogsHeader --grid-area="header" bind:selectedInstance bind:selectedLog bind:filter/>
<div>
    {#if !selectedInstance || !selectedLog}
        <h1 class="high-emphasis">No Logs</h1>
    {:else}
        {#await retrieveLogLines(selectedInstance, selectedLog, filter)}
            <Loader --color="var(--medium-black)" />
        {:then lines}
            {#if lines.length == 0}
                <h1 class="high-emphasis">No lines in {selectedLog}</h1>
            {:else}
                <VirtualList items={lines} let:item>
                    <p class="{item.lineType} line high-emphasis">
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
        line-height: 1.8rem;
        color: white;
        word-wrap: break-word;
        white-space: pre-wrap;
    }

    p.line {
        margin: 4px;
        padding: 12px;
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
