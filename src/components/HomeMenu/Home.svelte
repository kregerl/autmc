<script lang="ts">
    import Menu from "./Menu.svelte";
    import Navbar from "./Navbar.svelte"

</script>


<div class="navs">
    <Menu></Menu>
    <Navbar></Navbar>
</div>
<main>
</main>

<style>
    .navs {
        display: flex;
        flex-direction: row;
        justify-content: start;
    }
</style>
