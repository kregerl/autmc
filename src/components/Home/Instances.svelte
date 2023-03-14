<script context="module" lang="ts">
    export const ID = "Instances";
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { instanceStore } from "../../store/instancestore";
    import NewInstance from "../Modal/NewInstanceModal/NewInstance.svelte";
    import RightClickMenu from "../RightClickMenu.svelte";

    let showNewInstanceModal = false;

    function createNewInstance() {
        showNewInstanceModal = true;
    }

    function closeModal() {
        showNewInstanceModal = false;
    }

    async function launchInstance() {
        await invoke("launch_instance", { instanceName: this.id });
        console.log(this);
    }
</script>

<div class="header">
    <button class="new-instance" on:click={createNewInstance}>
        <img src="svg/PlusSign.svg" alt="New instance" />
    </button>
    <input type="text" placeholder="Search Instances" />
</div>
<div class="instance-grid">
    <!-- FIXME: Remove this undefined check. -->
    {#if $instanceStore !== undefined}
        {#each $instanceStore as instance}
            <div
                id={instance}
                class="instance"
                on:click={launchInstance}
                on:keydown
            >
                {instance}
            </div>
        {/each}
    {/if}
</div>

<RightClickMenu validClasses={["instance"]} />

{#if showNewInstanceModal}
    <NewInstance on:close={closeModal} />
{/if}

<style>
    .header {
        display: flex;
        align-items: center;
        position: sticky;
        top: 0;
        background-image: linear-gradient(#444, #333);
        width: 100%;
        height: 60px;
    }

    .instance-grid {
        height: auto;
        display: grid;
        grid-template-columns: repeat(5, minmax(0, 1fr));
        gap: 16px;
        margin: 24px;
        overflow-y: scroll;
    }

    .instance {
        width: 100%;
        min-height: 180px;
        background-color: red;
        border-radius: 12px;
        cursor: pointer;
    }

    img {
        width: 20px;
        height: 20px;
    }

    .new-instance {
        background-color: #4e4e4e;
        border: 0px;
        width: 50px;
        height: 50px;
        margin-top: 12px;
        margin-left: 12px;
        margin-right: 12px;
    }

    .new-instance:hover {
        background-color: #5e5e5e;
    }

    .new-instance:active {
        background-color: #6e6e6e;
    }

    input[type="text"] {
        color: white;
        font-weight: bold;
        height: calc(100% - 8px);
        margin: 4px;
        margin-left: 8px;
        padding: 0px;
        background-color: rgba(0, 0, 0, 0);
        border: none;
        font-size: 1.5vw;
    }

    input[type="text"]::placeholder {
        color: white;
        font-weight: bold;
    }
</style>
