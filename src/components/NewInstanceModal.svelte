<script lang="ts">
    import {createEventDispatcher, onDestroy } from "svelte";
    import TabBar from "./Tabbar/TabBar.svelte";
    import Tab from "./Tabbar/Tab.svelte";

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
            const nodes = modal.querySelectorAll('*') as HTMLElement[];
			const tabbable = Array.from(nodes).filter(n => n.tabIndex >= 0);
            
            if (tabbable.length > 0) {
                let index = tabbable.indexOf(document.activeElement as HTMLElement);
                if (index === -1 && event.shiftKey) index = 0;

                index += tabbable.length + (event.shiftKey ? -1 : 1);
                index %= tabbable.length;

                tabbable[index].focus();
            }
            event.preventDefault();
        }
        
        console.log("Here", event.key);
    }

    function tabKeyDown() {
        // TODO: Implement "Enter" updating selection of tabs.
    }

    let selected;
    let tabs = [
        {text: "Vanilla", component: Tab},
        {text: "Forge", component: Tab},
        {text: "Fabric", component: Tab},
        {text: "Curseforge", component: Tab}
    ];
</script>

<svelte:window on:keydown={handleKeyDown}/>

<div class="background" on:keydown={tabKeyDown}></div>

<div class="modal" role="dialog" aria-modal="true" bind:this={modal}>
    <input type="image" src="PlusSign.svg" alt="Close Instance Creation" on:click={close}>
    <h1 class="modal-header">New Instance</h1>
    <ul class="version-options">
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
        <li>Vanilla</li>
    </ul>
    <div class="instance-name">
        <label class="dropshadow" for="name">Instance Name:</label>
        <input type="text" id="name">
    </div>
    <div class="modal-content">

    </div>

    <div class="modal-footer">
        <button>Done</button>
    </div>

    <div class="version-tabs">
        <TabBar --width=100% tabs={tabs} bind:selected/>
    </div>


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
        display: grid; 
        grid-template-columns: 0.8fr 2fr; 
        grid-template-rows: 0.5fr 0.4fr 2.6fr 0.5fr; 
        gap: 0px 0px; 
        grid-template-areas: 
        "modal-header modal-header"
        "version-tabs instance-name"
        "version-options modal-content"
        "modal-footer modal-footer"; 
        position: absolute;
        left: 50%;
        top: 50%;
        width: calc(100vw - 24%);
        height: calc(100vh - 24%);
        background-color: #333;
        transform: translate(-50%, -50%);
        border-radius: 1em;
    }

    .modal-header {
        grid-area: modal-header;
        text-align: center;
        margin-top: 8px;
        margin-bottom: 0px;
        color: white;
    }

    .version-options {
        grid-area: version-options;
        font-size: 1.5rem;
        color: white;
        background-color: red;
        margin-left: 8px;
        margin-top: 8px;
        margin-bottom: 8px;
        padding-left: 0;
        overflow: hidden;
        overflow-y: scroll;
    }

    .version-options > li {
        list-style-type: none;
        height: 40px;
    }

    .instance-name {
        grid-area: instance-name;
        margin-left: 8px;
        vertical-align: center;
        font-size: 32px;
    }

    .instance-name > label {
        display: inline-block;
        color: white;
        font-size: 28px;
    }

    .instance-name > input[type=text] {
        display: inline-block;
        border: none;
        height: 1em;
        padding: 0px;
        margin: 0px;
        width: 400px;
    }

    .modal-content {
        grid-area: modal-content;
        width: 100%;
        height: 100%;
        background-color: blue;
        margin-bottom: 8px;
    }

    .modal-footer {
        grid-area: modal-footer;
    }

    .modal-footer > button {
        float:right;
    }

    /* .instance-name > .instance-icon {
        display: inline-block;
        height: 100%;
        width: 100px;
        background-color: red;

    } */
    /* .flex-row, .header {
        justify-content: center;
        align-items: center;
        height: 4em;
    }

    .flex-row > h1 {
        margin: 0px;
        color: white;
    }

    .modal-body {
        height: calc(100% - 8em);
        justify-content: start;
    }
    
    .loader-types {
        position: relative;
        background-color: red;
        height: 100%; 
        width: 350px;
        margin-left: 16px;
    }

    .versions {
        height: 100%;
        width: 100%;
    } */

    input[type=image] {
        position: absolute;
        right: 0;
        height: 4em;
        margin: 4px;
        transform: rotate(45deg);
        border: none;
    }

</style>