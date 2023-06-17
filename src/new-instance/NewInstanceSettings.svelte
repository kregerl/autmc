<script lang="ts">
    import { appConfigDir } from "@tauri-apps/api/path";
    import { path } from "@tauri-apps/api";
    import { open } from "@tauri-apps/api/dialog";
    import { invoke } from "@tauri-apps/api/tauri";

    import { navigate, useLocation } from "svelte-navigator";

    import type { VersionState } from "./NewInstanceVersion.svelte";
    import { Emphasis, ModloaderType, modloaderTypeToString } from "../menu";
    import { instanceStore } from "../store/instancestore";
    import TextBoxInput from "../components/input/TextBoxInput.svelte";
    import SvgCircleHoverButton from "../components/buttons/SvgCircleHoverButton.svelte";
    import CheckboxInput from "../components/input/CheckboxInput.svelte";
    import SvgButton, { Order } from "../components/buttons/SvgButton.svelte";

    interface InstanceSettings {
        instanceName: string;
        vanillaVersion: string;
        modloaderType: string;
        modloaderVersion: string;
        additionalJvmArguments: string;
        javaPathOverride: string;
        resolutionWidth: string;
        resolutionHeight: string;
        startWindowMaximized: boolean;
        recordPlaytime: boolean;
        showRecordedPlaytime: boolean;
        overrideOptionsTxt: boolean;
        overrideServersDat: boolean;
    }

    const location = useLocation();
    let settings: InstanceSettings = {
        instanceName: "",
        vanillaVersion: $location.state.vanillaVersion,
        modloaderType: modloaderTypeToString($location.state.modloaderType),
        modloaderVersion: $location.state.modloaderVersion,
        additionalJvmArguments: "",
        javaPathOverride: "",
        resolutionWidth: "800",
        resolutionHeight: "600",
        startWindowMaximized: false,
        recordPlaytime: true,
        showRecordedPlaytime: true,
        overrideOptionsTxt: false,
        overrideServersDat: false
    };

    let addMods: boolean = false;
    let addResourcepacks: boolean = false;

    $: generateInstanceName($location.state as VersionState);
    $: instancePath = getInstancePath(settings.instanceName);

    $: hasConflict =
        $instanceStore !== undefined &&
        $instanceStore
            .map((element) => element.instance_name)
            .includes(settings.instanceName);

    function back() {
        navigate("/newinstance-version", { state: $location.state });
    }

    // Generate the default instance name from the version state
    function generateInstanceName(state: VersionState) {
        console.log(state);
        let result = "Minecraft";
        if (!state) {
            return result;
        }

        let modloaderString = modloaderTypeToString(state.modloaderType);
        if (modloaderString) result += " ";
        result += modloaderString;

        if (state.vanillaVersion) {
            result += ` ${state.vanillaVersion}`;
        }
        settings.instanceName = result;
    }

    function finish() {
        let state = $location.state;
        console.log("state", state);
        invoke("obtain_version", {
            settings: settings,
        });
        navigate("/");
    }

    async function openJavaPathDialog() {
        const selected = await open({
            multiple: false,
            directory: false,
            title: "Select Java Binary",
        });
        // TODO: Do something with this "selected"
        console.log("selected", selected);
    }

    async function getInstancePath(name: string): Promise<string> {
        const appDir = await appConfigDir();
        return path.join(appDir, "instances", name);
    }
</script>

<main>
    <h1 class="title high-emphasis">Instance Settings</h1>
    <div class="grid-container">
        <div class="settings">
            <h3>Testing</h3>
            <hr />
            <div class="name-wrapper">
                <TextBoxInput
                    id="instance"
                    label="Instance Name"
                    bind:value={settings.instanceName}
                    bind:disabled={hasConflict}
                />
                <p>
                    {#await instancePath}
                        {"..."}
                    {:then path}
                        {path}
                    {/await}
                </p>
            </div>

            <h1 class="high-emphasis">JVM</h1>
            <TextBoxInput
                id="jvmargs"
                label="Additional JVM Arguments"
                emphasis={Emphasis.Medium}
                bind:value={settings.additionalJvmArguments}
                --width="350px"
            />
            <br />

            <br />
            <div class="flex-row">
                <TextBoxInput
                    id="jvmargs"
                    label="Override Java Path"
                    emphasis={Emphasis.Medium}
                    bind:value={settings.javaPathOverride}
                    --width="350px"
                />
                <SvgCircleHoverButton
                    src="svg/Folder.svg"
                    alt="Folder"
                    --hover-color="var(--dark-black)"
                    --svg-size="33px"
                    --hover-size="55.5px"
                    --margin-top="8px"
                    --margin-left="8px"
                    --padding="7.5px"
                    on:click={openJavaPathDialog}
                />
            </div>

            <h1 class="high-emphasis">Default Resolution</h1>
            <div class="resolution flex-row">
                <TextBoxInput
                    id="reswidth"
                    label="Width"
                    emphasis={Emphasis.Medium}
                    bind:value={settings.resolutionWidth}
                    --width="60px"
                />
                <TextBoxInput
                    id="resheight"
                    label="Height"
                    emphasis={Emphasis.Medium}
                    bind:value={settings.resolutionHeight}
                    --width="60px"
                />
            </div>
            <br />
            <CheckboxInput
                text="Start Window Maximized"
                bind:checked={settings.startWindowMaximized}
            />
            <h1 class="high-emphasis">Playtime</h1>
            <CheckboxInput
                text="Record Playtime"
                bind:checked={settings.recordPlaytime}
            />
            <CheckboxInput
                text="Display Recorded Playtime"
                bind:checked={settings.showRecordedPlaytime}
            />

            <h1 class="high-emphasis">Overrides</h1>
            <CheckboxInput
                text="Override Options.txt"
                bind:checked={settings.overrideOptionsTxt}
            />

            <CheckboxInput
                text="Override Servers.dat"
                bind:checked={settings.overrideServersDat}
            />
        </div>
        <div class="includes">
            <h3>Includes</h3>
            <hr />
            <div class="checkboxes">
                <CheckboxInput
                    disabled={$location.state.modloaderType ===
                        ModloaderType.None}
                    text="Add Mods"
                    bind:checked={addMods}
                />
                <CheckboxInput
                    text="Add Resourcepacks"
                    bind:checked={addResourcepacks}
                />
            </div>
        </div>
    </div>

    <div class="footer flex-row">
        <SvgButton
            order={Order.ImgFirst}
            src="svg/RightArrow.svg"
            alt="Back"
            --img-rotation="180deg"
            on:click={back}
        />
        <SvgButton src="svg/Check.svg" alt="Done" on:click={finish} />
    </div>
</main>

<style>
    main {
        width: 100%;
        height: 100%;
        /* Prevent margin collapse */
        padding-top: 0.05px;
        background-color: var(--medium-black);
    }

    .title {
        color: white;
        text-align: center;
        font-size: 3.2rem;
    }

    .grid-container {
        display: grid;
        grid-template-columns: 1fr 1fr;
        grid-template-rows: 1fr;
        gap: 0px 8px;
        grid-template-areas: "left right";
        width: 100%;
        height: 100%;
        box-sizing: border-box;
        padding: 0 80px 0 80px;
        background-color: var(--medium-black);
    }

    .settings {
        grid-area: left;
        height: 100%;
        width: 100%;
        padding-left: 32px;
        box-sizing: border-box;
    }

    .includes {
        color: white;
        grid-area: right;
        height: 100%;
        width: 100%;
        font-size: 1.8rem;
        padding-left: 32px;
        box-sizing: border-box;
    }

    .checkboxes {
        margin: 32px 0 0 0;
    }

    p {
        color: white;
        font-size: 1.2rem;
    }

    h1 {
        color: white;
    }

    h3 {
        color: white;
        font-size: 2.4rem;
        text-align: center;
    }

    .footer {
        position: absolute;
        right: 0;
        bottom: 0;
        justify-content: space-between;
        margin: 0 16px 16px 0;
    }

    .settings > .name-wrapper {
        margin: 32px 0 0 0;
    }

    .settings > .name-wrapper > p {
        margin-top: 4px;
    }

    .resolution {
        width: 140px;
        justify-content: space-between;
    }
</style>
