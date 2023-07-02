<script lang="ts">
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    export function close() {
        dispatch("close");
    }

    function onKeyDown(event: KeyboardEvent) {
        console.log("onKeyDown");
        if (event.key === "Escape") {
            close();
        }
    }
</script>

<slot />
<div class="modal-background" />
<svelte:body on:keydown|preventDefault={onKeyDown}/>

<style>
    .modal-background {
        z-index: 1;
        position: absolute;
        top: 0; 
        left: 0;
        width: 100vw;
        height: 100vh;
        background-color: rgba(0, 0, 0, var(--bg-opacity, 0.65));
    }
</style>
