<script lang="ts">
    import { appConfigDir } from "@tauri-apps/api/path";
    import { path } from "@tauri-apps/api";

    import { navigate, useLocation } from "svelte-navigator";

    import SvgButton, { Order } from "../buttons/SvgButton.svelte";
    import TextBoxInput from "../input/TextBoxInput.svelte";
    import type { VersionState } from "./NewInstanceVersion.svelte";
    import { Emphasis, ModloaderType } from "../../menu";
    import CheckboxInput from "../input/CheckboxInput.svelte";
    import SettingsHeader from "./SettingsHeader.svelte";

    let additionalJvmArguments: string = "";
    let shouldOverrideJavaPath: boolean = false;
    let javaPathOverride: string = "";



    let recordPlaytime: boolean = true;
    let showRecordedPlaytime: boolean = true;

    const location = useLocation();
    $: generateInstanceName($location.state as VersionState);
    let value: string;

    $: instancePath = getInstancePath(value);

    function back() {
        navigate("/newinstance-version", { state: $location.state });
    }

    // Generate the default instance name from the version state
    function generateInstanceName(state: VersionState) {
        console.log(state);
        let result = "Minecraft";
        if (state.modloaderType === ModloaderType.Forge) {
            result += " Forge";
        } else if (state.modloaderType === ModloaderType.Fabric) {
            result += " Fabric";
        }

        if (state.vanillaVersion) {
            result += ` ${state.vanillaVersion}`;
        }
        value = result;
    }

    async function getInstancePath(name: string): Promise<string> {
        const appDir = await appConfigDir();
        return path.join(appDir, "instances", name);
    }
</script>

<main>
    <div class="settings">
        <h1 class="high-emphasis">Instance Settings</h1>
        <div class="name-wrapper">
            <TextBoxInput id="instance" label="Instance Name" bind:value/>
            <p>
                {#await instancePath}
                    {"..."}
                {:then path}
                    {path}
                {/await}
            </p>
        </div>
        <h2 class="high-emphasis">JVM</h2>
        <TextBoxInput id="jvmargs" label="Additional JVM Arguments" emphasis={Emphasis.Medium} bind:value={additionalJvmArguments} --width="350px"/>
        <div class="flex-row">
            <CheckboxInput bind:checked={shouldOverrideJavaPath}/>
            <TextBoxInput id="jvmargs" label="Override Java Path" emphasis={shouldOverrideJavaPath ? Emphasis.Medium : Emphasis.Low} bind:value={javaPathOverride} --width="350px"/>
        </div>

        
        <h2 class="high-emphasis">Playtime</h2>
        <!-- Header -->
        <CheckboxInput text="Record Playtime" bind:checked={recordPlaytime}/>
        <CheckboxInput text="Display Recorded Playtime" bind:checked={showRecordedPlaytime}/>
        
    
    </div>

    <div class="footer flex-row">
        <SvgButton
            order={Order.ImgFirst}
            src="svg/RightArrow.svg"
            alt="Back"
            --img-rotation="180deg"
            on:click={back}
        />
        <SvgButton src="svg/Check.svg" alt="Done" />
    </div>
</main>

<style>
    main {
        width: 100%;
        height: 100%;
        /* Prevent margin collapse */
        padding-top: 0.05px;
    }

    p {
        color: white;
        font-size: 1.2rem;
    }

    h1 {
        text-align: center;
        font-size: 2.4rem;
    }

    h1, h2 {
        color: white;
    }

    .footer {
        position: absolute;
        right: 0;
        bottom: 0;
        justify-content: space-between;
        margin: 0 16px 16px 0;
    }

    .settings {
        padding: 16px;
        width: auto;
        border-radius: 4px;
        /* Prevent margin collapse */
        padding-top: 0.05px;
        height: calc(100% - 200px);
        margin: 80px 80px 0 80px;
        background-color: var(--light-black);
    }

    .settings > .name-wrapper {
        margin: 32px 0 0 0;
    }

    .settings > .name-wrapper > p {
        margin-top: 4px;
    }

</style>
