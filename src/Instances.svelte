<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import {
        InstanceConfiguration,
        instanceStore,
    } from "./store/instancestore";
    import Loader from "./components/Loader.svelte";

    async function retrieveInstances(): Promise<InstanceConfiguration[]> {
        if ($instanceStore === undefined) {
            $instanceStore = await invoke("load_instances");
        }
        return $instanceStore;
    }
</script>

<div class="instances-wrapper">
    <div class="instances">
        {#await retrieveInstances()}
            <Loader />
        {:then instances}
            {#each instances as instance}
                <div class="instance">
                    <div class="background">
                        <div class="version-info high-emphasis">{instance.modloader_type} {instance.modloader_version}</div>
                    </div>
                    <div class="footer">
                        <h2 class="high-emphasis">{instance.instance_name}</h2>
                        <!-- TODO: Add actual author here. -->
                        <p class="medium-emphasis">Created By: You</p>
                    </div>
                </div>
            {/each}
        {/await}
    </div>
</div>

<style>
    h2, p {
        color: white;
        margin: 0 0 0 6px;
    }

    h2 {
        font-size: 1.8rem;
    }

    p {
        margin-top: 2px;
        font-size: 1.4rem;
    }

    .instances-wrapper {
        grid-area: var(--grid-area);
        margin: 0 24px 0 24px;
    }

    .instances {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        grid-template-rows: repeat(auto-fill, minmax(0, 250px));
        gap: 16px;
        width: 100%;
        height: 100%;
    }

    .instance {
        background-color: var(--light-black);
        width: 100%;
        border-radius: 4px;
        aspect-ratio: 3/2;
    }

    .instance:hover {
        background-color: #2f2f2f;
        cursor: pointer;
    }

    .instance > .background {
        position: relative;
        background: linear-gradient(
                0deg,
                rgba(0, 0, 0, 0.35),
                rgba(0, 0, 0, 0.35)
            ),
            url(https://media.forgecdn.net/avatars/611/496/637995823847751059.png)
                100% 20% / cover no-repeat;
        height: 75%;
        margin: 4px;
    }

    .version-info {
        position: absolute;
        right: 0;
        bottom: 0;
        margin: 4px;
        width: fit-content;
        height: fit-content;
        font-size: 1.4rem;
        padding: 3px;
        background-color: var(--dark-black);
        color: white;
    }
</style>
