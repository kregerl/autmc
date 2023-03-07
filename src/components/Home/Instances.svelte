<script context="module" lang="ts">
    export const ID = "Instances";
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import NewInstance from "../Modal/NewInstanceModal/NewInstance.svelte";
    import RightClickMenu from "../RightClickMenu.svelte";

    // FIXME: All instances are hidden briefly when this gets changed since its being awaited on. 
    let promise = getInstances();
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

    async function getInstances(): Promise<string[]> {
        return invoke("load_instances");
    }
</script>


<div class="header">
    <button class="new-instance" on:click={createNewInstance}>
        <svg
            fill="#FFF"
            version="1.1"
            id="Layer_1"
            xmlns="http://www.w3.org/2000/svg"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            x="0px"
            y="0px"
            viewBox="0 0 512 512"
            style="enable-background:new 0 0 512 512;"
            xml:space="preserve">
            <g>
                <g>
                    <polygon
                        points="289.391,222.609 289.391,0 222.609,0 222.609,222.609 0,222.609 0,289.391 222.609,289.391 222.609,512 
			289.391,512 289.391,289.391 512,289.391 512,222.609 		"
                    />
                </g>
            </g>
        </svg>
    </button>
    <input type="text" placeholder="Search Instances" />
</div>
<div class="instance-grid">
    {#await promise then instances}
        {#each instances as instance}
            <div
                id={instance}
                class="instance"
                on:click={launchInstance}
                on:keydown
            >
                {instance}
            </div>
        {/each}
    {/await}
</div>

<!-- TODO: Use this  -->
<RightClickMenu validClasses={["instance"]}/>

{#if showNewInstanceModal}
    <NewInstance on:close={closeModal} bind:instances={promise}/>
{/if}

<style>
    .header {
        display: flex;
        align-items: center;
        position: sticky;
        top: 0;
        background-image: linear-gradient(
            #444,
            #333
        );
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

    svg {
        width: 20px;
        height: 20px;
    }

    .new-instance {
        background-color: #4E4E4E;
        border: 0px;
        width: 50px;
        height: 50px;
        margin-top: 12px;
        margin-left: 12px;
        margin-right: 12px;
    }

    .new-instance:hover {
        background-color: #5E5E5E;
    }

    .new-instance:active {
        background-color: #6E6E6E;
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
