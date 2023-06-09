<script lang="ts">
    import HamburgerButton from "./components/HamburgerButton.svelte";
    import MenuModal from "./modal/MenuModal/MenuModal.svelte";
    import SettingsModal from "./modal/SettingsModal/SettingsModal.svelte";
    import { MenuId, OpenModalType } from "./menu";
    import Instances from "./Instances.svelte";
    import SvgCircleHoverButton from "./components/buttons/SvgCircleHoverButton.svelte";
    import { fade } from "svelte/transition";
    import Logs from "./logs/Logs.svelte";
    import Screenshots from "./screenshots/Screenshots.svelte";

    let activeMenuId: MenuId = MenuId.Instances;
    let openModal: OpenModalType = OpenModalType.None;

    let isSideModalOpen: boolean = false;
    $: if (isSideModalOpen) {
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

<main out:fade={{duration: 100}}>
    <div class="side-menu flex-col">
        <div class="side-menu-top">
            <HamburgerButton
                bind:checked={isSideModalOpen}
                --margin="8px 0 0 8px"
            />
            <SvgCircleHoverButton
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

            <SvgCircleHoverButton
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

            <SvgCircleHoverButton
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
            <SvgCircleHoverButton
                src="svg/Settings.svg"
                alt="Settings"
                on:click={() => (openModal = OpenModalType.Settings)}
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
        <SettingsModal on:close={closeSettingsModal} />
    {/if}

    {#if activeMenuId == MenuId.Instances}
        <Instances --grid-area="content" />
    {:else if activeMenuId == MenuId.Screenshots}
        <Screenshots --grid-area="header / header / content / content"/>
    {:else if activeMenuId == MenuId.Logs}
        <Logs --grid-area="content"/>
    {/if}
</main>

<style>
    main {
        display: grid;
        grid-template-columns: 64px 1fr;
        grid-template-rows: 64px 1fr;
        gap: 0px 0px;
        grid-template-areas:
            "side header"
            "side content";
        width: 100%;
        height: 100%;
        background-color: var(--medium-black);
        overflow-y: scroll;
        overflow-x: hidden;
    }

    .side-menu {
        position: absolute;
        grid-area: side;
        justify-content: space-between;
        width: 64px;
        height: 100%;
        background-color: var(--dark-black);
    }
</style>
