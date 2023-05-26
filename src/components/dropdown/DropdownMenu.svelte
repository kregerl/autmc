<script lang="ts">
    // Inputs
    export let placeholder: string;
    export let options: string[] = [];
    export let disabled: boolean = false;

    // Output
    export let selected: string = options.at(0) ?? "";

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
    <img src="svg/Caret.svg" alt="Caret">    
    {placeholder}
</div>
{#if menuShown}
    <div class="menu">
        {#each options as option}
            <h1 class="menu-item" id={option} on:click={setSelected} on:keydown>{option}</h1>
        {/each}
    </div>
{/if}

<style>
    .dropdown {
        align-items: center;
        font-size: 1.6rem;
        line-height: 1.6rem;
        background-color: red;
        color: white;
        padding: 4px;
        cursor: pointer;
    }

    img {
        transform: rotate(90deg);
    }
    
    .menu {
        position: relative;
        background-color: white;
        max-height: var(--max-height, 100px);
        overflow-y: scroll;
        overflow-x: hidden;
    }
    
    .menu > .menu-item {
        cursor: pointer;
    }

    .menu > .menu-item:hover {
        background-color: blue;
    }

    .menu > * {
        margin-top: 0;
    }
</style>