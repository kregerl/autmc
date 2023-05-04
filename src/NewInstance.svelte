<script lang="ts">
    import { navigate } from "svelte-navigator";
    import { fly } from "svelte/transition";


    function back() {
        navigate(-1);
    }

</script>

<div on:click={back}>
    Test
</div>

<style>
    div {
        background-color: red;
        width: 50px;
        height: 50px;
    }
</style>