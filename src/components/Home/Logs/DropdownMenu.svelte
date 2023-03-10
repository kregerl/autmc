<script lang="ts">
    export let options: string[];
    export let selected: string;
</script>

<select disabled={selected === undefined} bind:value={selected}>
    {#each options as option}
        <option value={option}>
            {option}
        </option>
    {/each}
</select>

<style>
</style>
