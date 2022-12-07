<script lang="ts">
    import Menu from "./Menu.svelte";
    import Tab from "../Tabbar/Tab.svelte";
    import Instances from "./Instances.svelte";
    import Transition from "../Transition.svelte";
    import TabBar from "../Tabbar/TabBar.svelte";

    let selected;

    const navTabs = [
        {text: "Instances", fs: "1.25em", component: Tab},
        {text: "Logs", fs: "1.25em",component: Tab},
    ]
</script>

<Transition>
    <div class="container">
        <div class="menu">
            <Menu/>
        </div>
        <div class="header">
            <TabBar --min-width=180px --font-size=1.25em tabs={navTabs} bind:selected></TabBar>
        </div>
        <div class="content">
            {#if selected !== undefined && selected === "instances"}
                <Instances/>
            {:else if selected !== undefined && selected === "logs"}
                <h1>Log</h1>
            {/if}
        </div>

    </div>
</Transition>
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
