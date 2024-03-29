<script lang="ts" context="module">
    export interface VersionState {
        vanillaVersion: string;
        modloaderVersion: string;
        modloaderType: ModloaderType;
    }
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";

    import { navigate, useLocation } from "svelte-navigator";

    import VanillaContent from "./VanillaContent.svelte";
    import { onMount } from "svelte";
    import ImportZip from "./ImportZip.svelte";
    import BrowseCurseforge from "./thirdparty-content/BrowseCurseforge.svelte";
    import { InstanceType, ModloaderType } from "../menu";
    import { VersionManifest, manifestStore } from "../store/manifeststore";
    import TextLoader from "../components/loader/TextLoader.svelte";
    import SvgButton from "../components/buttons/SvgButton.svelte";

    export let selectedInstanceType: InstanceType = InstanceType.Vanilla;

    const location = useLocation();
    onMount(() => {
        if ($location.state.vanillaVersion !== undefined)
            selectedVanillaVersion = $location.state.vanillaVersion;

        if ($location.state.modloaderType !== undefined)
            modloaderType = $location.state.modloaderType;

        if ($location.state.modloaderVersion !== undefined)
            selectedModloaderVersion = $location.state.modloaderVersion;
    });

    let selectedVanillaVersion: string;
    let selectedModloaderVersion: string;
    let modloaderType: ModloaderType;

    let zipPath: string | undefined = undefined;

    function back() {
        navigate("/");
    }

    function importZip() {
        invoke("import_zip", { zipPath: zipPath });
        back();
    }

    function next() {
        let state: VersionState = {
            vanillaVersion: selectedVanillaVersion,
            modloaderVersion: selectedModloaderVersion,
            modloaderType: modloaderType,
        };
        navigate("/newinstance-settings", { state: state });
    }

    function onClickTab() {
        let instanceType: InstanceType = Number(
            (this as Element).getAttribute("data-instance-type")
        );
        console.log("HERE", instanceType);
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
        return $manifestStore;
    }
</script>

{#await retrieveManifests()}
    <TextLoader />
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
                <VanillaContent
                    {versionManifest}
                    bind:selectedVanillaVersion
                    bind:selectedModloaderVersion
                    bind:modloaderType
                />
            {:else if selectedInstanceType === InstanceType.Curseforge}
                <BrowseCurseforge />
            {:else if selectedInstanceType === InstanceType.Modrinth}
                <h1>TODO: Modrinth</h1>
            {:else if selectedInstanceType === InstanceType.Zip}
                <ImportZip bind:zipPath />
            {/if}
        </div>

        <div class="footer flex-row">
            <SvgButton
                src="svg/Close.svg"
                alt="Cancel"
                --img-width="18px"
                on:click={back}
            />

            {#if selectedInstanceType === InstanceType.Zip && zipPath}
                <SvgButton
                    src="svg/Check.svg"
                    alt="Done"
                    on:click={importZip}
                />
            {:else if selectedInstanceType !== InstanceType.Curseforge}
                <SvgButton
                    disabled={selectedInstanceType === InstanceType.Zip &&
                        !zipPath}
                    src="svg/RightArrow.svg"
                    alt="Next"
                    on:click={next}
                />
            {/if}
        </div>
    </main>
{/await}

<style>
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
        box-shadow: 0px 4px var(--medium-purple);
    }

    .instance-type-content {
        margin: 16px 80px 0 80px;
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
