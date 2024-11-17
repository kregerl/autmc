<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import AccountContainer from "./AccountContainer.svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { navigate } from "svelte-navigator";

    interface DeviceCode {
        message: string;
        device_code: string;
    }

    let deviceCode: DeviceCode | undefined;
    $: deviceCode = undefined;
    $: authenticationError = undefined;


    // It shouldn't be possible to have more than one error here, but just incase.
    function formatErrors(): string {
        let result = "";
        for (const [key, value] of Object.entries(authenticationError)) {
            result = `${result}${key}: ${value}\n`
        }
        return result;
    }

    async function startAuthentication() {
        deviceCode = await invoke<DeviceCode>("start_authentication_flow");
        let x = await invoke("poll_device_code_authentication", {
            deviceCode: deviceCode.device_code,
        }).catch((err) => authenticationError = err);
        navigate("/");
    }
</script>

<AccountContainer>
    <h1 class="high-emphasis">AUTM Launcher</h1>

    {#if deviceCode === undefined}
        <div class="mslogin" on:click={() => startAuthentication()} on:keydown>
            <img
                class="high-emphasis"
                src="svg/microsoft.svg"
                alt="Pixel art minecraft background"
            />
            <p class="high-emphasis">Microsoft Login</p>
        </div>
    {:else}
        {#if authenticationError === undefined}
            <div class="mslogin">{deviceCode.message}</div>
        {:else}
            <div class="mslogin">{formatErrors()}</div>
        {/if}
    {/if}
</AccountContainer>

<style>
    h1 {
        text-align: center;
        font-weight: bold;
        font-size: 3.2rem;
        color: white;
    }

    .mslogin {
        margin: 12px;
        padding: 12px;
        border-radius: 8px;
        border: 2px solid var(--dark-purple);
        background-color: var(--medium-purple);
        display: flex;
        justify-content: center;
    }

    .mslogin:hover {
        background-color: var(--light-purple);
        cursor: pointer;
    }

    p {
        color: white;
        font-size: 2rem;
        font-weight: bold;
    }

    .mslogin > img {
        margin-right: 8px;
        margin-left: 8px;
        width: 2.2rem;
    }
</style>
