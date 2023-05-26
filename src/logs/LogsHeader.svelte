<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import DropdownMenu from "../components/dropdown/DropdownMenu.svelte";

    async function getLogs() {
        let logs: Map<string, Map<string, string[]>> = new Map();
        for (let [key, value] of Object.entries(await invoke("get_logs"))) {
            let inner = new Map();
            for (let [k, v] of Object.entries(value)) {
                inner.set(k, v);
            }
            logs.set(key, inner);
        }
        // TODO: Create $logstore.
    }
</script>


<div>
    <DropdownMenu placeholder="Placeholder" options={["1", "2", "3", "4", "5"]}>
    </DropdownMenu>
</div>

<style>
    div {
        grid-area: var(--grid-area);
    }
</style>