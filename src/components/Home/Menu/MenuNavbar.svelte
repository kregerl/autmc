<script lang="ts">
    import {updateSelectionClasses} from "../../../selectable";
    import { onMount } from "svelte";

    export let buttons: string[];
    export let selected: string = buttons.at(0);


    function updateSelections() {
        selected = updateSelectionClasses(this.id, buttons);
    }

    onMount(() => {
        if (buttons.length > 0) {
            selected = updateSelectionClasses(selected, buttons);
        }
    });
</script>

<div>
    <!-- TODO: Fill the space below the buttons with alternating colors. Keep same button size. -->
    {#each buttons as button}
        <button class="menu-button" id={button} on:click={updateSelections}>{button}</button>
    {/each}
</div>

<style>
    button {
        color: white;
        font-size: 1.75vw;
        width: 100%;
        height: 80px;
        margin-top: 0px;
        margin-bottom: 0px;
        padding: 0px;
        border: 1px solid #333;
        border-top: 0px;
        border-right: 0px;
        background-color: #4e4e4e;
    }

    button:hover {
        background-color: #5E5E5E;
    }

    button:active {
        background-color: #6E6E6E;
    }

    div:first-child {
        border-top: 1px solid #333;
    }
</style>
