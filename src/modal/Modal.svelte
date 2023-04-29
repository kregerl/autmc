<script lang="ts">
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    export function close() {
        dispatch("close");
    }

</script>

<div class="modal-background" />
<slot />

<style>
    .modal-background {
        z-index: 1;
        width: 100vw;
        height: 100vh;
        background-color: rgba(0, 0, 0, 0.65);
    }
</style>
