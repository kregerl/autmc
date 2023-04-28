<script lang="ts">
    import HamburgerButton from "./components/HamburgerButton.svelte";
    import MenuModal from "./modal/MenuModal.svelte";

    let isSideModalOpen: boolean = true;

    function close() {
        isSideModalOpen = false;
    }
</script>

<main>
    <div class="menu-header">
        <HamburgerButton bind:checked={isSideModalOpen} />
    </div>
    {#if isSideModalOpen}
        <MenuModal on:close={close}/>
    {/if}
</main>

<style>
    main {
        background-color: var(--dark-black);
        width: 100%;
        height: 100%;
    }

    .menu-header {
        position: absolute;
        top: 8px;
        left: 8px;
    }
</style>
