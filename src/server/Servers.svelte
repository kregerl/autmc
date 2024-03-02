<script lang="ts">
    import { onMount } from "svelte";
    import { Terminal } from "xterm";
    import ServerStatus from "./ServerStatus.svelte";

    // let div: HTMLDivElement;
    // let history = [];
    // let historyIndex = -1;
    // let line: string = "";
    // const terminal = new Terminal();

    // function onKeyDown(event: KeyboardEvent) {
    //     switch (event.key) {
    //         case "Enter": {
    //             history.push(line);
    //             historyIndex = history.length - 1;
    //             line = "";
    //             // TODO: Send to rust
    //             event.preventDefault();
    //             break;
    //         }
    //         case "ArrowUp": {
    //             if (historyIndex > 0 && historyIndex >= history.length) {
    //                 historyIndex -= 1;
    //             }
    //             line = history[historyIndex];
    //             historyIndex = Math.max(historyIndex - 1, 0);
    //             event.preventDefault();
    //             break;
    //         }
    //         case "ArrowDown": {
    //             if (historyIndex + 1 >= history.length) {
    //                 historyIndex = history.length
    //                 line = "";
    //             } else {
    //                 historyIndex = Math.min(historyIndex + 1, history.length - 1);
    //                 line = history[historyIndex];
    //             }
    //             event.preventDefault();
    //             break;
    //         }
    //     }
    // }

    // onMount(() => {
    //     terminal.open(div);
    //     for (let i = 0; i < 100; i++) {
    //         terminal.writeln(`Line ${i}`);
    //     }

    // });

    // function test() {
    //     terminal.writeln("\x1b[31mNewLine");
    // }
</script>

<div class="outer">
    <div class="server-info">
        <ServerStatus />
    </div>
    <div class="console">

    </div>
</div>

<!-- <div class="outer">
    <div id="terminal" bind:this={div}></div>
    <input type="text" bind:value={line} on:keydown={onKeyDown}/>
    <button on:click={test}/>
</div> -->

<style>
    .outer {
        grid-area: var(--grid-area);
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: row;
    }

    .server-info {
        background-color: var(--lightest-black);
        width: 40%;
        height: auto;
        margin: 24px 8px 24px 24px;
    }

    .console {
        background-color: red;
        width: 100%;
        height: auto;
        margin: 24px 24px 24px 8px;
    }
</style>
