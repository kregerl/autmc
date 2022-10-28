<script lang="ts">
    import {invoke} from "@tauri-apps/api/tauri";
    import {open} from "@tauri-apps/api/shell"
    import MenuButton from "./components/MenuButton.svelte";

    const url = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?prompt=select_account&client_id=94fd28d0-faa6-4d85-920d-69a2abe16bcd&response_type=code&scope=XboxLive.signin%20offline_access&redirect_uri=autmc://localhost"

    function microsoftLogin() {
        // open(url)
        invoke('show_microsoft_login_page').catch((err) => console.log(err));
        // console.log("Here")
        // window.location.replace('autmc://auth')
    }

</script>

<main>
    <MenuButton on:click={microsoftLogin}>Log In</MenuButton>
</main>

<style>
</style>