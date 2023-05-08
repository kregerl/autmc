<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";

    import { navigate } from "svelte-navigator";

    import SvgButton from "../buttons/SvgButton.svelte";
    import VanillaContent from "./VanillaContent.svelte";
    import Loader from "../Loader.svelte";
    import { InstanceType } from "../../menu";
    import { VersionManifest, manifestStore } from "../../store/manifeststore";

    export let selectedInstanceType: InstanceType = InstanceType.Vanilla;

    function back() {
        navigate(-1);
    }

    function next() {
        navigate("/newinstance-settings");
    }

    function onClickTab() {
        let instanceType: InstanceType = Number(
            (this as Element).getAttribute("data-instance-type")
        );
        console.log(instanceType);
        selectedInstanceType = instanceType;
    }

    async function retrieveManifests(): Promise<VersionManifest> {
        if ($manifestStore === undefined) {
            $manifestStore = await invoke("obtain_manifests");
            $manifestStore.forge_versions = new Map(
                Object.entries($manifestStore.forge_versions)
            );
            for (let key of $manifestStore.forge_versions.keys())
                $manifestStore.forge_versions.get(key).reverse();
        }
        console.log("$manifestStore", $manifestStore);
        return $manifestStore;
    }
</script>

{#await retrieveManifests()}
    <Loader />
{:then versionManifest}
    <main>
        <div class="tabs flex-row">
            <div
                data-instance-type={InstanceType.Vanilla}
                class={selectedInstanceType == InstanceType.Vanilla
                    ? "selected"
                    : ""}
                on:click={onClickTab}
                on:keydown
            >
                Vanilla
            </div>
            <div
                data-instance-type={InstanceType.Curseforge}
                class={selectedInstanceType == InstanceType.Curseforge
                    ? "selected"
                    : ""}
                on:click={onClickTab}
                on:keydown
            >
                Curseforge
            </div>
            <div
                data-instance-type={InstanceType.Modrinth}
                class={selectedInstanceType == InstanceType.Modrinth
                    ? "selected"
                    : ""}
                on:click={onClickTab}
                on:keydown
            >
                Modrinth
            </div>
            <div
                data-instance-type={InstanceType.Zip}
                class={selectedInstanceType == InstanceType.Zip
                    ? "selected"
                    : ""}
                on:click={onClickTab}
                on:keydown
            >
                Import Zip
            </div>
        </div>

        <div class="instance-type-content">
            {#if selectedInstanceType === InstanceType.Vanilla}
                <VanillaContent {versionManifest} />
            {:else if selectedInstanceType === InstanceType.Curseforge}
                <h1>TODO: Curseforge</h1>
            {:else if selectedInstanceType === InstanceType.Modrinth}
                <h1>TODO: Modrinth</h1>
            {:else if selectedInstanceType === InstanceType.Zip}
                <h1>TODO: Zip</h1>
            {/if}
        </div>

        <div class="footer flex-row">
            <SvgButton
                src="svg/Close.svg"
                alt="Cancel"
                --img-width="18px"
                on:click={back}
            />
            <SvgButton src="svg/RightArrow.svg" alt="Next" on:click={next} />
        </div>
    </main>
{/await}

<style>
    /* TODO: REMOVE THIS */
    h1 {
        color: white;
    }

    main {
        background-color: var(--medium-black);
        width: 100%;
        height: 100%;
    }

    .tabs {
        justify-content: center;
        margin: 0 80px 0 80px;
    }

    .tabs > div {
        flex-grow: 1;
        line-height: 1.8rem;
        font-size: 1.8rem;
        padding: 8px 0 8px 0;
        text-align: center;
        color: white;
        background-color: var(--dark-black);
        cursor: pointer;
    }

    .tabs > div:hover {
        background-color: var(--light-black);
    }

    .tabs > div.selected {
        box-shadow: 0px 4px #573993;
    }

    .instance-type-content {
        margin: 32px 80px 0 80px;
        background-color: white;
        height: calc(100% - 120px);
    }

    .footer {
        position: absolute;
        right: 0;
        bottom: 0;
        justify-content: space-between;
        margin: 0 16px 16px 0;
    }
</style>
