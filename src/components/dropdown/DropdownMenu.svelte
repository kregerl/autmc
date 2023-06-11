<script lang="ts">
    import { slide } from "svelte/transition";

    // Inputs
    export let options: string[];
    export let disabled: boolean = false;

    // Output
    export let selected: string = options.at(0) ?? "";
    $: selected = options.at(0);

    let menuShown: boolean = false;
    function toggleDropdown() {
        menuShown = !menuShown;
    }

    function closeDropdown() {
        menuShown = false;
    }

    function setSelected() {
        selected = this.id;
        console.log("selected", selected);
        closeDropdown();
    }
</script>

<div class="dropdown flex-row {disabled}" on:click={toggleDropdown} on:keydown>
    <img class="high-emphasis" src="svg/Caret.svg" alt="Caret">  
    <p class="high-emphasis placeholder">{selected}</p>
</div>
{#if menuShown}
    <div class="menu" in:slide={{duration: 350}}>
        {#each options as option}
            <p class="high-emphasis menu-item" id={option} on:click={setSelected} on:keydown>{option}</p>
        {/each}
    </div>
{/if}

<style>
    .dropdown {
        align-items: center;
        font-size: 1.6rem;
        line-height: 1.6rem;
        background-color: var(--color, var(--dark-black));
        color: white;
        padding: 6px;
        cursor: pointer;
        transition: 0.15s linear;
        border-radius: 4px;
    }

    .dropdown:hover {
        background-color: var(--hover-color, var(--light-black));
    }

    img {
        transform: rotate(90deg);
    }
    
    .menu {
        position: relative;
        background-color: var(--color, var(--dark-black));
        max-height: var(--max-height, 100px);
        overflow-y: scroll;
        overflow-x: hidden;
        z-index: 5;
    }
    
    .menu > .menu-item {
        cursor: pointer;
        word-wrap: break-word;
        color: white;
        font-size: 1.4rem;
        line-height: 1.4rem;
        margin: 0;
        padding: 6px;
    }

    .menu > .menu-item:hover {
        background-color: blue;
    }

    .menu > * {
        margin-top: 0;
    }

    p.placeholder {
       font-size: 1.8rem;
       color: white;
       margin: 0;
    }
</style>