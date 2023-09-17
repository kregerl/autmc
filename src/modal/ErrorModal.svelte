<script lang="ts">
    import Modal from "./Modal.svelte";
    let modal: Modal;

    function close() {
        modal.close();
    }
</script>

<div class="background">
    <div class="close" on:click={close} on:keydown>
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="32"
            height="32"
            viewBox="0 0 20 20"
            ><path
                fill="#393a37"
                d="m15.8333 5.34166-1.175-1.175-4.6583 4.65834-4.65833-4.65834-1.175 1.175 4.65833 4.65834-4.65833 4.6583 1.175 1.175 4.65833-4.6583 4.6583 4.6583 1.175-1.175-4.6583-4.6583z"
            /></svg
        >
    </div>
</div>
<Modal bind:this={modal} on:close --bg-opacity="0.25" />

<style>
    .background {
        position: absolute;
        background-color: #ef665b;
        z-index: 6;
        width: 42vw;
        height: 36vh;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }

    .close {
        width: 32px;
        height: 32px;
        cursor: pointer;
        margin-left: auto;
    }
    .close path {
        fill: white;
    }
</style>
