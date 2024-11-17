<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import DropdownMenu from "../components/dropdown/DropdownMenu.svelte";
    import { logStore } from "../store/logstore";
    import TextBoxInput from "../components/input/TextBoxInput.svelte";
    import CheckboxInput from "../components/input/CheckboxInput.svelte";

    export let selectedInstance: string;
    export let selectedLog: string;
    export let filter: string;
    export let useRegex: boolean;

    async function getLogMap(): Promise<Map<string, string[]>> {
        if ($logStore === undefined) $logStore = new Map();

        for (let [key, value] of Object.entries(await invoke("get_logs"))) {
            // Sort and reverse are done in-place
            value.sort((a, b) => a.localeCompare(b, "en", { numeric: true }));
            value.reverse();
            $logStore.set(key, value);
        }
        return $logStore;
    }

    function getLogs(logs: Map<string, string[]>): string[] {
        if (!selectedInstance) {
            selectedInstance = [...logs.keys()].at(0);
        } 
        return logs.get(selectedInstance);
    }
</script>

<div class="header flex-row">
    {#await getLogMap() then logs}
        <div class="wrapper">
            <DropdownMenu
                options={[...logs.keys()]}
                bind:selected={selectedInstance}
                --color="var(--dark-black)"
                --hover-color="var(--light-black)"
                --max-height="220px"
            />
        </div>
        <div class="wrapper">
            <DropdownMenu
                options={getLogs(logs)}
                bind:selected={selectedLog}
                --color="var(--dark-black)"
                --hover-color="var(--light-black)"
                --max-height="220px"
            />
        </div>
        <TextBoxInput id="filter" label="Filter lines" bind:value={filter} />
        <div class="regex-wrapper">
            <CheckboxInput text="Use Regex" bind:checked={useRegex}/>
        </div>
    {/await}
</div>

<style>
    div.header {
        z-index: 2;
        margin: 8px 0 0 8px;
        grid-area: var(--grid-area);
    }

    .wrapper {
        margin: 4px;
        justify-content: center;
    }

    .regex-wrapper {
        margin: 12px 0 0 8px;
    }
</style>
