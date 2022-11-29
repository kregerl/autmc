<script lang="ts">
    import Menu from "./Menu.svelte";
    import Navbar from "./Navbar.svelte"
    import Instances from "./Instances.svelte";

    let selected;


</script>


<div class="flex-row">
    <Menu></Menu>
    <div class="flex-column content">
        <Navbar bind:selected></Navbar>
        <div id="launcher-body" class="launcher-body">
            {#if selected !== undefined && selected.id === "instances"}
                <Instances/>
            {:else if selected !== undefined && selected.id === "log"}
                <h1>Log</h1>
            {/if}
        </div>
    </div>
</div>
<main>
</main>

<style>
    .launcher-body {
        /* border: 2px solid blue; */
        flex: 1 1 auto;
    }

    .content {
        width: 100%;
    }
</style>
