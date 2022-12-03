<script lang="ts">
    import Menu from "./Menu.svelte";
    import Navbar from "./Navbar.svelte"
    import Instances from "./Instances.svelte";

    let selected;


</script>


<div class="container">
    <div class="menu">
        <Menu/>
    </div>
    <div class="header">
        <Navbar bind:selected></Navbar>
    </div>
    <div class="content">
        {#if selected !== undefined && selected.id === "instances"}
            <Instances/>
        {:else if selected !== undefined && selected.id === "log"}
            <h1>Log</h1>
        {/if}
    </div>

</div>

<!-- <div class="flex-row">
    <Menu></Menu>
    <div class="flex-column content">
        <Navbar bind:selected></Navbar>
        <div id="launcher-body" class="launcher-body">
            
        </div>
    </div>
</div> -->
<main>
</main>

<style>
    .container {
        display: grid; 
        grid-template-columns: 350px auto; 
        grid-template-rows: 80px calc(100vh - 80px); 
        gap: 0px 0px;
        grid-template-areas: 
        "menu header"
        "menu content"; 
    }

    .menu {
        grid-area: menu;
    }

    .header {
        grid-area: header;
    }

    .content {
        grid-area: content;
        overflow-y: scroll;
        background-color: #444;
    }

</style>
