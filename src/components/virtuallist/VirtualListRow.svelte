<script lang="ts">
    export let id: string;
    export let items: string[];
    export let index: number;
    export let selected: string;
</script>

<div
    id={id}
    class="row flex-row
    {index % 2 == 0 ? 'even' : 'odd'}
    {selected === id ? 'selected' : ''}"
    on:click
    on:keydown
>
    {#each items as item} 
        <div class="high-emphasis">{item}</div>
    {/each}
</div>

<style>
    .row {
        color: white;
        font-size: 1.5vmin;
        padding: 4px 0 4px 0;
        cursor: pointer;
    }

    .row > div {
        width: 33%;
    }

    .odd {
        background-color: var(--medium-light-black);
    }

    .even {
        background-color: var(--medium-black);
    }

    .row:hover:not(.selected) {
        background-color: var(--lightest-black);
    }

    .selected {
        background-color: #573993 !important;
    }
</style>
