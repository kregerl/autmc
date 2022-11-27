<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/tauri";
    
    let versions;
    let selected;
    let instanceName;
    let showSnapshots = false;

    onMount(async () => {
        invoke("obtain_manifests", { showSnapshots: showSnapshots })
        .then(payload => {
            versions = payload;
            selected = versions[0];
        })
        .catch(error => console.log(error));
    });

    function next() {
        console.log("Selected: ", selected);
        console.log("Show Snapshots: ", showSnapshots);
        invoke("obtain_version", { selected: selected, instanceName: instanceName })
        .then(payload => {
            console.log(payload);
        })
        .catch(error => console.log(error));
    }

</script>

<input type="checkbox" id="show-snapshots" bind:checked={showSnapshots}>
<input type="text" bind:value={instanceName}/>
<label for="show-snapshots">Show Snapshot Versions</label>
<select id="vanilla-versions" bind:value={selected}>
    {#each versions || [] as v} 
        <option>{v}</option>
    {/each}
</select>    
<button on:click={next}>Next</button>

<style>

</style>