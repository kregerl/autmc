<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { UnlistenFn, listen } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import { navigate } from "svelte-navigator";

    import {
        InstanceConfiguration,
        instanceStore,
    } from "./store/instancestore";
    import TextLoader from "./components/loader/TextLoader.svelte";
    import RightClickModal from "./modal/RightClickModal/RightClickModal.svelte";
    import TextBoxInput from "./components/input/TextBoxInput.svelte";
    import CheckboxInput from "./components/input/CheckboxInput.svelte";
    import {
        InstanceState,
        instanceStateStore,
    } from "./store/instancestatetore";
    import CircleLoader from "./components/loader/CircleLoader.svelte";
    import ErrorModal from "./modal/ErrorModal.svelte";

    let isErrorModalOpen = false;
    let useRegex: boolean = false;
    let instanceFilters: string = "";
    let promise: Promise<InstanceConfiguration[]>;

    $: promise = retrieveInstances(useRegex, instanceFilters);

    function newInstance() {
        navigate("/newinstance-version");
    }

    async function launchInstance() {
        await invoke("launch_instance", { instanceName: this.id });
        $instanceStateStore = new Map([
            ...$instanceStateStore,
            [this.id, InstanceState.Initializing],
        ]);
        console.log("launch_instance -- this", this);
        console.log("states", $instanceStateStore);
    }

    async function retrieveInstances(
        useRegex: boolean,
        filter: string,
        force: boolean = false
    ): Promise<InstanceConfiguration[]> {
        if ($instanceStore === undefined || force) {
            $instanceStore = await invoke("load_instances");
            $instanceStore.sort((a, b) =>
                a.instance_name.localeCompare(b.instance_name, "en", {
                    numeric: true,
                })
            );
        }

        if (filter) {
            if (useRegex) {
                return $instanceStore.filter((instance) =>
                    instance.instance_name.match(filter)
                );
            } else {
                return $instanceStore.filter((instance) =>
                    instance.instance_name.includes(filter)
                );
            }
        }

        return $instanceStore;
    }

    function closeErrorModal() {
        isErrorModalOpen = false;
    }

    let instanceCreatedListener: UnlistenFn;
    let loggingUnlistener: UnlistenFn;
    interface Logging {
        instance_name: string;
        category: string;
        line: string;
    }
    onMount(async () => {
        instanceCreatedListener = await listen("instance-done", (event) => {
            console.log("Here");
            promise = retrieveInstances(useRegex, instanceFilters, true);
        });

        loggingUnlistener = await listen<Logging>(
            "instance-logging",
            (event) => {
                const payload = event.payload;
                console.log("payload", payload);
                if (
                    $instanceStateStore.get(payload.instance_name) ===
                    InstanceState.Initialized
                )
                    return;

                if (
                    payload.line.includes("Setting user:") ||
                    payload.line.includes("Initializing LWJGL OpenAL")
                ) {
                    $instanceStateStore = new Map([
                        ...$instanceStateStore,
                        [payload.instance_name, InstanceState.Initialized],
                    ]);
                }
            }
        );

    });

    onDestroy(() => {
        instanceCreatedListener();
        loggingUnlistener();
    });
</script>

<div class="flex-row instances-header">
    <TextBoxInput
        id="searchinstances"
        bind:value={instanceFilters}
        label="Filter Instances"
    />
    <div class="regex-wrapper">
        <CheckboxInput text="Use Regex" bind:checked={useRegex} />
    </div>
</div>

<div class="instances-wrapper">
    <div class="instances">
        {#await promise}
            <TextLoader />
        {:then instances}
            {#each instances as instance}
                <div
                    id={instance.instance_name}
                    class="instance"
                    on:click={launchInstance}
                    on:keydown
                >
                    <div class="instance-image-wrapper">
                        {#if $instanceStateStore.get(instance.instance_name) !== undefined}
                            <div class="loader-overlay">
                                {#if $instanceStateStore.get(instance.instance_name) == InstanceState.Initializing}
                                    <CircleLoader />
                                {:else if $instanceStateStore.get(instance.instance_name) == InstanceState.Initialized}
                                    <img
                                        src="svg/Play.svg"
                                        alt="Running"
                                        width="36"
                                        height="36"
                                    />
                                {/if}
                            </div>
                        {/if}
                        <img src="grass.png" alt="" />
                        <div class="version-info high-emphasis">
                            {instance.modloader_type !== "None"
                                ? instance.modloader_type
                                : ""}
                            {instance.modloader_version}
                        </div>
                    </div>

                    <div class="footer">
                        <p class="high-emphasis instance-name">
                            {instance.instance_name}
                        </p>
                        <p class="author medium-emphasis">
                            Created By: {instance.author}
                        </p>
                    </div>
                </div>
            {/each}
        {/await}
    </div>
    <button class="flex-row" on:click={newInstance}>
        <img
            class="medium-emphasis"
            src="svg/PlusSign.svg"
            alt="New Instance"
        />
        <h3 class="medium-emphasis">New Instance</h3>
    </button>
    <RightClickModal validClasses={["instance"]} />
</div>
{#if isErrorModalOpen}
    <ErrorModal on:close={closeErrorModal}/>
{/if}

<style>
    button {
        position: absolute;
        bottom: 0;
        right: 0;
        margin: 0 16px 16px 0;
        border-radius: 4px;
        cursor: pointer;
        font-size: 1.6rem;
        background-color: var(--dark-black);
        border: none;
        color: white;
        box-shadow: 3px 3px 10px 2px rgba(0, 0, 0, 0.5);
        transition: 0.15s linear;
    }

    button > h3 {
        margin: 4px;
    }

    button > img {
        margin-top: 4px;
        width: 22px;
    }

    button:hover {
        background-color: var(--light-black);
    }

    h3,
    p {
        color: white;
        margin: 0 0 0 6px;
    }

    h3 {
        font-size: 1.8rem;
    }

    p {
        margin-top: 2px;
        font-size: 1.4rem;
    }

    .author {
        text-overflow: ellipsis;
    }

    .instances-wrapper {
        grid-area: var(--grid-area);
        margin: 0 24px 0 24px;
        -webkit-user-select: none;
        user-select: none;
    }

    .instance-name {
        width: 95%;
        display: inline-block;
        overflow: hidden;
        white-space: nowrap;
        margin-top: 4px;
        text-overflow: ellipsis;
        font-weight: bold;
    }

    .instances {
        /* Change these */
        --grid-layout-gap: 10px;
        --grid-column-count: 8;
        --grid-item--min-width: 100px;

        --gap-count: calc(var(--grid-column-count) - 1);
        --total-gap-width: calc(var(--gap-count) * var(--grid-layout-gap));
        --grid-item--max-width: calc(
            (100% - var(--total-gap-width)) / var(--grid-column-count)
        );

        display: grid;
        grid-template-columns: repeat(
            auto-fill,
            minmax(
                max(var(--grid-item--min-width), var(--grid-item--max-width)),
                1fr
            )
        );
        grid-gap: var(--grid-layout-gap);
        overflow-y: scroll;
    }

    @media only screen and (max-width: 1300px) {
        .instances {
            --grid-column-count: 6;
        }
    }

    .instance {
        display: flex;
        flex-direction: column;
        background-color: var(--light-black);
        width: 100%;
        aspect-ratio: 1/1.2;
        border-radius: 4px;
        -webkit-user-select: none;
        user-select: none;
        transition: 0.15s linear;
    }

    .instance > .instance-image-wrapper {
        position: relative;
        height: 75%;
        margin: 8px;
    }

    .instance-image-wrapper > img {
        width: 100%;
        image-rendering: crisp-edges;
    }

    .instance:hover {
        background-color: var(--lightest-black);
        cursor: pointer;
        border-radius: 0px;
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

    .instances-header {
        margin: 12px 0 0 24px;
        grid-area: header;
        z-index: 2;
    }

    .regex-wrapper {
        margin: 12px 0 0 8px;
    }

    .loader-overlay {
        display: flex;
        justify-content: center;
        align-items: center;
        position: absolute;
        width: 100%;
        height: 100%;
        z-index: 1;
        background-color: rgba(0, 0, 0, 0.65);
        padding-bottom: 5px;
    }
</style>
