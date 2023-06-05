<script lang="ts">
    import Modal from "../Modal.svelte";

    let modal: Modal;
</script>

<Modal bind:this={modal} on:close />
