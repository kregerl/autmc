<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { fly } from "svelte/transition";

    import { MenuId } from "../../menu";
    
    import Modal from "../Modal.svelte";
    import Head from "../../components/Head.svelte";
    import SidebarButton from "../../components/buttons/SidebarButton.svelte";
    import { navigate } from "svelte-navigator";

    const dispatch = createEventDispatcher();

    function changeMenuId(menuId: MenuId) {
        dispatch("menuchanged", {
            id: menuId
        });
        modal.close();
    }

    function switchAccounts() {
        modal.close();
        navigate("/switchaccounts");
    }

    export let activeMenuId: MenuId;

    let modal: Modal;
    let checked = true;

    $: if (!checked) {
        modal.close();
    }
</script>

<div class="background" transition:fly={{ x: -256, duration: 350 }}>
    <!-- TODO: On click open switch account -->
    <div class="header flex-row" on:click={switchAccounts} on:keydown>
        <div class="head-wrapper">
            <!-- TODO: Replace with real head -->
            <Head
                skinUrl="http://textures.minecraft.net/texture/2d1349ef4b20d188c789464e978e847cb3fe332ddb23f3df3cf870f6dc3e32f"
                size={48}
                --border-radius="4px"
            />
        </div>
        <div class="flex-col">
            <span class="medium-emphasis">Logged in as</span>
            <!-- TODO: Replace with real name -->
            <p class="high-emphasis">AreUThreatningMe</p>
        </div>
    </div>
    <div class="guide-buttons flex-col">
        <SidebarButton
            src="svg/Grid.svg"
            text="Instances"
            active={activeMenuId === MenuId.Instances}
            on:click={() => changeMenuId(MenuId.Instances)}
        />
        <SidebarButton
            src="svg/Screenshot.svg"
            text="Screenshots"
            active={activeMenuId === MenuId.Screenshots}
            on:click={() => changeMenuId(MenuId.Screenshots)}
        />
        <SidebarButton
            src="svg/Logs.svg"
            text="Logs"
            active={activeMenuId === MenuId.Logs}
            on:click={() => changeMenuId(MenuId.Logs)}
        />
    </div>
</div>
<Modal bind:this={modal} on:close />

<style>
    span {
        color: white;
        margin-left: 8px;
        margin-top: 12px;
        font-size: 1rem;
    }

    p {
        color: white;
        font-size: 1.4rem;
        margin: 4px 0 0 8px;
    }

    .head-wrapper {
        height: 48px;
        margin: 0 0 0 2px;
    }

    .header {
        padding: 4px 0px 4px 4px;
        margin: 4px 2px 0 64px;
        border-radius: 8px;
        cursor: pointer;
    }

    .header:hover {
        background-color: var(--light-black);
    }

    .background {
        z-index: 4;
        position: absolute;
        width: 256px;
        height: 100%;
        background-color: var(--dark-black);
        opacity: 95%;
        transition-duration: 200ms;
    }

    .guide-buttons {
        margin: 8px;
        margin-top: 10px;
    }
</style>
