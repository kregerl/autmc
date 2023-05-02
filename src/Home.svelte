<script lang="ts">
    import HamburgerButton from "./components/HamburgerButton.svelte";
    import SvgHoverButton from "./components/buttons/SvgHoverButton.svelte";
    import MenuModal from "./modal/MenuModal/MenuModal.svelte";
    import SettingsModal from "./modal/SettingsModal/SettingsModal.svelte";
    import { MenuId, OpenModalType } from "./menu";

    let activeMenuId: MenuId = MenuId.Instances;
    let openModal: OpenModalType = OpenModalType.None;

    let isSideModalOpen: boolean = false;
    $: if(isSideModalOpen)  {
        openModal = OpenModalType.SideMenu;
    } else {
        openModal = OpenModalType.None;
    }

    function closeSideModal() {
        isSideModalOpen = false;
    }

    function closeSettingsModal() {
        openModal = OpenModalType.None;
    }
</script>

<main>
    <div class="side-menu flex-col">
        <div class="side-menu-top">
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
        <div class="side-menu-bottom">
            <SvgHoverButton
                src="svg/Settings.svg"
                alt="Settings"
                on:click={() => openModal = OpenModalType.Settings}
                --svg-size="33px"
                --hover-size="55.5px"
                --margin-bottom="8px"
                --margin-left="8px"
                --padding="7.5px"
            />
        </div>
    </div>
    {#if openModal === OpenModalType.SideMenu}
        <MenuModal
            bind:activeMenuId
            on:close={closeSideModal}
            on:menuchanged={(event) => (activeMenuId = event.detail.id)}
        />
    {:else if openModal === OpenModalType.Settings}
        <SettingsModal on:close={closeSettingsModal}/>
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
        background-color: var(--medium-black);
    }

    .side-menu {
        grid-area: side;
        justify-content: space-between;
        width: 64px;
        height: 100%;
        background-color: var(--dark-black);
    }
</style>