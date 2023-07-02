<script lang="ts">
    import Modal from "../../../modal/Modal.svelte";

    let modal: Modal;
    let checked = true;

    $: if (!checked) {
        modal.close();
    }
</script>

<div>
    Test
</div>
<Modal bind:this={modal} on:close />

<style>
    div {
        z-index: 2;
        position: absolute;
        top: 10vh;
        left: 10vw;
        width: 80vw;
        height: 80vh;
        background-color: red;
    }
</style>
