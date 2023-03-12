<script lang="ts">
    import Menu from "./Menu/Menu.svelte";
    import Instances, { ID as INSTANCE_ID } from "./Instances.svelte";
    import Logs, { ID as LOGS_ID } from "./Logs/Logs.svelte";
    import Screenshots, {
        ID as SCREENSHOTS_ID,
    } from "../Screenshots/Screenshots.svelte";

    let selected;
    let element;
</script>

<div class="container">
    <div class="menu">
        <Menu bind:selectedTab={selected} />
    </div>
    <div class="content" bind:this={element}>
        {#if selected === INSTANCE_ID}
            <Instances />
        {:else if selected === LOGS_ID}
            <Logs />
        {:else if selected === SCREENSHOTS_ID}
            <Screenshots />
        {/if}
    </div>
</div>

<style>
    .container {
        display: grid;
        grid-template-columns: 0.4fr 1.6fr;
        gap: 0px 0px;
        grid-template-areas:
            "menu content"
            "menu content";
    }

    .menu {
        grid-area: menu;
    }

    .content {
        grid-area: content;
        overflow-y: scroll;
        background-color: #333;
    }

    .content:has(div.flex-row) {
        display: flex;
        flex-direction: column;
        justify-content: center;
    }
</style>
