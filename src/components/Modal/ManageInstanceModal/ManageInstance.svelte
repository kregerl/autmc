<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";

    export let targetInstance;
    let modal;
    const dispatch = createEventDispatcher();

    function close() {
        dispatch("close");
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key == "Escape") {
            close();
            return;
        } else if (event.key == "Tab") {
            // Trap "tab" key focus
            const nodes = modal.querySelectorAll("*") as HTMLElement[];
            const tabbable = Array.from(nodes).filter((n) => n.tabIndex >= 0);

            if (tabbable.length > 0) {
                let index = tabbable.indexOf(
                    document.activeElement as HTMLElement
                );
                if (index === -1 && event.shiftKey) index = 0;

                index += tabbable.length + (event.shiftKey ? -1 : 1);
                index %= tabbable.length;

                tabbable[index].focus();
            }
            event.preventDefault();
        }
    }

    function tabKeyDown() {
        // TODO: Implement "Enter" updating selection of tabs.
    }

    onMount(async () => {
        console.log(targetInstance.id)
    });
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="background" on:keydown={tabKeyDown} />

<div class="modal" role="dialog" aria-modal="true" bind:this={modal}>
    <input
        type="image"
        src="svg/PlusSign.svg"
        alt="Close Manage Instance"
        on:click={close}
    />

</div>

<style>
    .background {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(51, 51, 51, 0.5);
    }
    .modal {
        position: absolute;
        left: 50%;
        top: 50%;
        width: calc(100vw - 12%);
        height: calc(100vh - 12%);
        transform: translate(-50%, -50%);
        background-color: #333;
        border: 2px solid red;
        border-radius: 1em;
        box-shadow: 5px 5px 16px 2px rgba(0,0,0,0.75);
    }

    input[type="image"] {
        position: absolute;
        top: 0;
        right: 0;
        height: 3vw;
        margin: 4px;
        transform: rotate(45deg);
        border: none;
    }
</style>
