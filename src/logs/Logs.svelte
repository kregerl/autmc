<script lang="ts">
    import { invoke } from "@tauri-apps/api";

    import LogsHeader from "./LogsHeader.svelte";
    import TextLoader from "../components/loader/TextLoader.svelte";
    import VirtualList from "../components/virtual-list/VirtualList.svelte";
    import { onDestroy, onMount } from "svelte";
    // import { UnlistenFn, listen } from "@tauri-apps/api/event";
    import { logStore } from "../store/logstore";
    import { Terminal } from "xterm";
    import { FitAddon } from "xterm-addon-fit";

    const terminal = new Terminal();
    const fitAddon = new FitAddon();

    let terminalDiv: HTMLDivElement;
    let selectedInstance: string;
    let selectedLog: string;
    let filter: string;
    let useRegex: boolean;

    interface TaggedLine {
        line: string;
        lineType: string;
    }

    function writeLines(lines: TaggedLine[]) {
        for (let line of lines) {
            let tag = "";
            switch (line.lineType) {
                case "error": {
                    tag = "\x1b[31m";
                    break;
                }
                case "warning": {
                    tag = "\x1b[33m";
                    break;
                }
            }
            terminal.writeln(`${tag}${line.line}\x1b[0m`);
        }
    }

    // TODO: Cache lines once theyre already loaded once.
    async function retrieveLogLines(
        useRegex: boolean,
        selectedInstance: string,
        selectedLog: string,
        filter: string,
    ): Promise<void> {
        console.log(selectedInstance, selectedLog);
        let lines: TaggedLine[] = await invoke("read_log_lines", {
            instanceName: selectedInstance,
            logName: selectedLog,
        });
        if (filter) {
            console.log("filter", filter);
            if (useRegex) {
                lines = lines.filter((line) => line.line.match(filter));
            } else {
                lines = lines.filter((line) => line.line.includes(filter));
            }
        }
        terminal.clear();
        writeLines(lines);
        fitAddon.fit();
    }

    interface Logging {
        instance_name: string;
        category: string;
        line: string;
    }

    // let loggingUnlistener: UnlistenFn;
    onMount(() => {
        terminal.open(terminalDiv);
        terminal.loadAddon(fitAddon);
        terminal.options.fontFamily = "monospace"
        terminal.options.scrollback = 10000
        fitAddon.fit();
        // loggingUnlistener = await listen<Logging>(
        //     "instance-logging",
        //     (event) => {
        //         const payload = event.payload;
        //         // selectedInstance = payload.instance_name;
        //         // $logStore.set(selectedInstance, [
        //         //     ...$logStore.get(selectedInstance),
        //         //     payload.category,
        //         // ]);
        //     },
        // );
    });
</script>

<LogsHeader
    --grid-area="header"
    bind:selectedInstance
    bind:selectedLog
    bind:filter
    bind:useRegex
/>

<div class="outer">
    {#if !selectedInstance || !selectedLog}
        <h1 class="high-emphasis">No Logs</h1>
    {:else}
        {#await retrieveLogLines(useRegex, selectedInstance, selectedLog, filter)}
            <TextLoader --color="var(--medium-black)" />
        {/await}
    {/if}
    <div class="terminal" bind:this={terminalDiv} />
</div>

<!-- <div>
    {#if !selectedInstance || !selectedLog}
        <h1 class="high-emphasis">No Logs</h1>
    {:else if selectedLog === "Running"}
        <VirtualList items={runningLogLines} let:item>
            <p class=" line high-emphasis">
                {item}
            </p>
        </VirtualList>
    {:else}
        {#await retrieveLogLines(useRegex, selectedInstance, selectedLog, filter)}
            <TextLoader --color="var(--medium-black)" />
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
</div> -->

<style>
    .outer {
        grid-area: var(--grid-area);
        margin: 8px;
        margin-bottom: 0px;
        overflow:hidden;
    }

    .terminal {
        height: 100%;
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
