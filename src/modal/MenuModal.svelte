<script lang="ts">
    import { fly } from "svelte/transition";
    import Modal from "./Modal.svelte";
    import Head from "../components/Head.svelte";

    let modal: Modal;
    let checked = true;

    $: if (!checked) {
        modal.close();
    }
</script>

<div class="background" transition:fly={{ x: -350, duration: 350 }}>
    <div class="header flex-row">
        <div class="head-wrapper">
            <!-- TODO: Replace with real head -->
            <Head
                skinUrl="http://textures.minecraft.net/texture/2d1349ef4b20d188c789464e978e847cb3fe332ddb23f3df3cf870f6dc3e32f"
                size={48}
                --border-radius=4px
            />
        </div>
        <div class="flex-col">
            <span class="medium-emphasis">Logged in as</span>
            <!-- TODO: Replace with real name -->
            <p class="high-emphasis">AreUThreatningMe</p>
        </div>
    </div>
</div>
<Modal bind:this={modal} on:close />

<style>
    span {
        color: white;
        margin-left: 8px;
        margin-top: 4px;
        font-size: 1.4rem;
    }
    
    p {
        color: white;
        font-size: 2.4rem;
        margin: 4px 0 0 8px;
    }

    .header {
        margin: 8px 0 0 64px;
    }

    .background {
        position: absolute;
        width: 350px;
        height: 100%;
        background-color: var(--light-black);
        transition-duration: 200ms;
    }
</style>
