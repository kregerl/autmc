<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let image: string;

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
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="background" on:keydown={tabKeyDown} />
<div>
    <input
        type="image"
        src="svg/PlusSign.svg"
        alt="Close Instance Creation"
        on:click={close}
    />
    <img
        src={image}
        alt="screenshot"
        class="modal"
        role="dialog"
        aria-modal="true"
        bind:this={modal}
    />
</div>

<style>
    .background {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(51, 51, 51, 0.75);
    }

    .modal {
        display: grid;
        grid-template-columns: 1fr;
        grid-template-rows: 0.2fr 0.5fr 2.3fr;
        gap: 0px 0px;
        grid-template-areas:
            "header"
            "version-options"
            "modal-content";
        position: absolute;
        left: 50%;
        top: 50%;
        width: calc(100vw - 20%);
        height: calc(100vh - 20%);
        transform: translate(-50%, -50%);
        box-shadow: 5px 5px 16px 2px rgba(0, 0, 0, 0.75);
        z-index: 100;
    }

    input[type="image"] {
        position: absolute;
        top: 0;
        right: 0;
        height: 3vw;
        margin: 16px;
        transform: rotate(45deg);
        border: none;
    }
</style>
