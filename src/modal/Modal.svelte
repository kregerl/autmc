<script lang="ts">
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    export function close() {
        dispatch("close");
    }

    function onKeyDown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            close();
        }
    }
</script>

<slot />
<div class="modal-background" />
<svelte:window on:keydown|preventDefault={onKeyDown}/>

<style>
    .modal-background {
        z-index: 1;
        position: absolute;
        width: 100vw;
        height: 100vh;
        background-color: rgba(0, 0, 0, 0.65);
    }
</style>
