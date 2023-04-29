<script lang="ts">
    import HamburgerButton from "./components/HamburgerButton.svelte";
    import SvgHoverButton from "./components/buttons/SvgHoverButton.svelte";
    import MenuModal from "./modal/MenuModal.svelte";
    import { MenuId } from "./menu";

    let activeMenuId: MenuId = MenuId.Instances;

    $: console.log("activeMenuId", activeMenuId);

    let isSideModalOpen: boolean = false;

    function close() {
        isSideModalOpen = false;
    }
</script>

<main>
    <div class="side-menu">
        <HamburgerButton
            bind:checked={isSideModalOpen}
            --margin="8px 0 0 8px"
        />
        <SvgHoverButton
            src="svg/Grid.svg"
            alt="Instances"
            active={activeMenuId === MenuId.Instances}
            on:click={() => (activeMenuId = MenuId.Instances)}
            --svg-size="33px"
            --hover-size="55.5px"
            --margin-top="8px"
            --margin-left="8px"
            --padding="7.5px"
        />

        <SvgHoverButton
            src="svg/Screenshot.svg"
            alt="Screenshots"
            active={activeMenuId === MenuId.Screenshots}
            on:click={() => (activeMenuId = MenuId.Screenshots)}
            --svg-size="33px"
            --hover-size="55.5px"
            --margin-top="8px"
            --margin-left="8px"
            --padding="7.5px"
        />

        <SvgHoverButton
            src="svg/Logs.svg"
            alt="Logs"
            active={activeMenuId === MenuId.Logs}
            on:click={() => (activeMenuId = MenuId.Logs)}
            --svg-size="33px"
            --hover-size="55.5px"
            --margin-top="8px"
            --margin-left="8px"
            --padding="7.5px"
        />
    </div>
    {#if isSideModalOpen}
        <MenuModal
            bind:activeMenuId
            on:close={close}
            on:menuchanged={(event) => (activeMenuId = event.detail.id)}
        />
    {/if}

    {#if activeMenuId == MenuId.Instances}
        <h1>TODO: Instances</h1>
    {:else if activeMenuId == MenuId.Screenshots}
        <h1>TODO: Screenshots</h1>
    {:else if activeMenuId == MenuId.Logs}
        <h1>TODO: Logs</h1>
    {/if}
</main>

<style>
    /* TODO: Remove this  */
    h1 {
        color: white;
    }

    main {
        display: grid;
        grid-template-columns: 0.1fr 1.9fr;
        grid-template-rows: 0.2fr 1.8fr;
        gap: 0px 0px;
        grid-template-areas:
            "side header"
            "side content";
        width: 100%;
        height: 100%;
        background-color: var(--dark-black);
    }

    .side-menu {
        grid-area: side;
        width: 64px;
        height: 100%;
    }
</style>
