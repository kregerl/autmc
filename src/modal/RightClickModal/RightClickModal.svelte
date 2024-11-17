<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import Modal from "../Modal.svelte";
    import { slide } from "svelte/transition";

    type MouseEventCallback = (event: MouseEvent) => void;

    interface Point {
        x: number;
        y: number;
    }

    interface Button {
        src: string;
        alt: string;
        callback: MouseEventCallback;
    }

    // List of valid classes to show the menu on.
    export let validClasses: string[] = [];

    let buttons: Array<Button | undefined> = [
        {
            src: "svg/SingleGear.svg",
            alt: "Manage",
            callback: manageInstance,
        },
        {
            src: "svg/Folder.svg",
            alt: "Open Folder",
            callback: openFolder,
        },
    ];

    function manageInstance(event: MouseEvent) {
        // TODO: Got a instance management screen
        
        close();
    }

    function openFolder(event: MouseEvent) {
        if (lastTarget) {
            console.log("lastTarget.id", lastTarget.id)
            invoke("open_folder", { instanceName: lastTarget.id });
        }
        close();
    }

    let showMenu: boolean = false;
    let position: Point = { x: 0, y: 0 };
    let lastTarget: Element;

    function clickModal(element: Element, callback: MouseEventCallback) {
        function onClick(event: MouseEvent) {
            if (!element.contains(event.target as Element)) {
                // FIXME: Might not need to pass event here.
                callback(event);
            }
        }

        document.body.addEventListener("click", onClick);

        return {
            update(newCallbackFunction) {
                callback = newCallbackFunction;
            },
            destroy() {
                document.body.removeEventListener("click", onClick);
            },
        };
    }

    function close() {
        showMenu = false;
    }

    function showContextMenu(event: MouseEvent) {
        position = { x: event.clientX, y: event.clientY };
        showMenu = true;
    }

    async function onRightClick(event: MouseEvent) {
        let target = event.target as Element;
        if (showMenu && !target.matches("#right-click-menu *")) {
            showMenu = false;
            await new Promise((res) => setTimeout(res, 100));
        }

        if (validClasses.length == 0) {
            showContextMenu(event);
        } else {
            if (target.matches(".instance *")) {
                lastTarget = target.closest("div.instance");
                showContextMenu(event);
            }
        }
    }
</script>

{#if showMenu}
    <div
        id="right-click-menu"
        class="flex-col"
        style:--x-pos="{position.x}px"
        style:--y-pos="{position.y}px"
        use:clickModal={close}
        in:slide={{duration: 350}}
    >
        {#each buttons as button}
            <div class="button high-emphasis" on:click={button.callback} on:keydown>
                <img src={button.src} alt={button.alt}>
                {button.alt}    
            </div>
        {/each}
    </div>
    <Modal on:close={close} --bg-opacity="0"/>
{/if}
<!-- <svelte:window on:contextmenu|preventDefault={onRightClick} /> -->

<style>
    #right-click-menu {
        position: absolute;
        top: var(--y-pos);
        left: var(--x-pos);
        z-index: 100;
        background-color: var(--dark-black);
        width: 8vw;
        height: 100px;
    }

    .button {
        display: flex;
        flex-direction: row;
        font-size: 1vw;
        color: white;
        margin-bottom: 4px;
        cursor: pointer;
    }

    .button:hover {
        background-color: #3a3a3a;
    }

    .button > img {
        vertical-align: middle;
        margin: 4px;
        margin-right: 8px;
        max-width: 1vw;
    }
</style>
