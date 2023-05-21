<script lang="ts">
    export let placeholder: string;
    export let disabled: boolean = false;

    let menuShown: boolean = false;
    function showDropdown() {
        menuShown = true;
    }

    function closeDropdown() {
        menuShown = false;
    }
</script>

<div class="dropdown flex-row" on:click={showDropdown} on:keydown>
    <img src="svg/Caret.svg" alt="Caret">    
    {placeholder}
</div>
{#if menuShown}
    <div class="menu">
        <h1>Test</h1>
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
        top: 1.6rem;
        left: -10px;
        background-color: white;
    }
</style>