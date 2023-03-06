<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import ManageInstance from "./Modal/ManageInstanceModal/ManageInstance.svelte";

    interface Button {
        svg: string;
        svgAlt: string;
        text: string;
        callback: (event: MouseEvent) => void;
    }

    export let validClasses: string[] = [];
    let pos = { x: 0, y: 0 };
    let show: boolean = false;
    let lastTarget;

    // If button is undefined then it will be replaced with <hr>
    let buttons: Array<Button | undefined> = [
        {
            svg: "svg/ManageInstance.svg",
            svgAlt: "Manage",
            text: "Manage",
            callback: openManageInstanceModal,
        },
        {
            svg: "svg/Folder.svg",
            svgAlt: "Open Folder",
            text: "Open Folder",
            callback: openFolder,
        },
        {
            svg: "svg/Copy.svg",
            svgAlt: "Copy",
            text: "Copy",
            callback: () => {},
        },
        undefined,
        {
            svg: "svg/Repair.svg",
            svgAlt: "Repair",
            text: "Repair",
            callback: () => {},
        },
        {
            svg: "svg/Trash.svg",
            svgAlt: "Delete Instance",
            text: "Delete",
            callback: () => {},
        },
    ];

    export function clickOutside(element, callbackFunction) {
        function onClick(event) {
            if (!element.contains(event.target)) {
                callbackFunction();
            }
        }

        document.body.addEventListener("click", onClick);

        return {
            update(newCallbackFunction) {
                callbackFunction = newCallbackFunction;
            },
            destroy() {
                document.body.removeEventListener("click", onClick);
            },
        };
    }
    let showManageInstanceModal = false;

    function openManageInstanceModal() {
        showManageInstanceModal = true;
        close();
    }

    async function openFolder() {
        if (lastTarget) {
            await invoke("open_folder", { instanceName: lastTarget.id });
        }
        close();
    }

    function close() {
        show = false;
    }

    function showContextMenu(event) {
        pos = { x: event.clientX, y: event.clientY };
        show = true;
    }

    async function onRightClick(event) {
        if (show) {
            show = false;
            await new Promise((res) => setTimeout(res, 100));
        }

        if (validClasses.length == 0) {
            showContextMenu(event);
        } else {
            for (let i = 0; i < validClasses.length; i++) {
                let validClass = validClasses[i];
                if (event.target.classList.contains(validClass)) {
                    lastTarget = event.target;
                    showContextMenu(event);
                    break;
                }
            }
        }
    }

    function tabKeyDown() {
        // TODO: Implement "Enter" updating selection of tabs.
    }
</script>

{#if show}
    <div
        class="menu"
        style="top: {pos.y}px; left:{pos.x}px"
        use:clickOutside={close}
    >
        {#each buttons as button}
            {#if button === undefined}
                <hr />
            {:else}
                <div
                    class="button"
                    on:click={button.callback}
                    on:keydown={tabKeyDown}
                >
                    <img src={button.svg} alt={button.svgAlt} />
                    {button.text}
                </div>
            {/if}
        {/each}
    </div>
{/if}

{#if showManageInstanceModal}
    <ManageInstance
        on:close={() => (showManageInstanceModal = false)}
        targetInstance={lastTarget}
    />
{/if}

<svelte:body on:contextmenu|preventDefault={onRightClick} />

<style>
    .menu {
        display: flex;
        flex-direction: column;
        position: absolute;
        z-index: 1000;
        border-radius: 4px;
        background-color: #272727;
        width: 8vw;
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
        filter: invert(0.8);
    }

    hr {
        color: white;
        width: 95%;
        float: left;
        margin-top: 4px;
    }
</style>
