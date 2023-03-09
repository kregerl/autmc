<script lang="ts">
    export let options: string[];
    export let selected: string;
</script>

<select bind:value={selected}>
    {console.log(options)}
    {#each options as option}
        <option>
            {option}
        </option>
    {/each}
</select>

<style>
    
</style>
