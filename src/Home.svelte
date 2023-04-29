<script lang="ts">
    import HamburgerButton from "./components/HamburgerButton.svelte";
    import MenuModal from "./modal/MenuModal.svelte";

    let isSideModalOpen: boolean = false;

    function close() {
        isSideModalOpen = false;
    }
</script>

<main>
    <div class="menu-header">
        <HamburgerButton bind:checked={isSideModalOpen} />
    </div>
    <!-- <div class="side-menu high-emphasis">
        <img src="svg/Grid.svg" alt="Instances">
    </div> -->
    {#if isSideModalOpen}
        <MenuModal on:close={close}/>
    {/if}
</main>

<style>
    main {
        display: grid;
        grid-template-columns: 0.2fr 1.8fr;
        grid-template-rows: 0.2fr 1.8fr;
        gap: 0px 0px;
        grid-template-areas: 
        "burger header"
        "side content";
        width: 100%;
        height: 100%;
        background-color: var(--dark-black);
    }

    .menu-header {
        grid-area: burger;
        margin: 8px 0 0 8px;
    }

    .side-menu {
        grid-area: side;
        width: 100%;
        height: 100%;
        margin: 8px 0 0 8px;
    }

    .side-menu > img {
        padding: 7.5px;
        width: 33px;
    }
</style>
