<script lang="ts">
    import { listen } from "@tauri-apps/api/event"
    import { invoke } from "@tauri-apps/api/tauri";
    import { link } from "svelte-navigator";
    import LoginButton from "./LoginButton.svelte";

    function microsoftLogin() {
        invoke("show_microsoft_login_page").catch((err) => console.log(err));
    }
</script>


<div class="flex">
    <div class="accounts">
        <h1>Switch Accounts</h1>
        <LoginButton class="menu-button" on:click={microsoftLogin}>Add Account</LoginButton>
    </div>
</div>

<style>
    .flex {
        display: flex;
        align-items: flex-start;
        margin-top: 20%;
        justify-content: center;
        width: 100%;
        height: 100%;
    }

    .flex div {
        background-color: gray;
        color: black;
    }

    .accounts {
        text-align: center;
        display: flex;
        flex-direction: column;
    }
    
    .accounts * {
        margin: 4px;
    }

    .accounts :global(.menu-button) {
        margin: 4px;
    }

</style>