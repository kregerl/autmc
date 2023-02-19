<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import LoginButton from "./LoginButton.svelte";

    interface BasicAccount {
        uuid: string;
        name: string;
        skin_url: string;
    }

    interface AccountInformation {
        active_account: string;
        accounts: Map<string, BasicAccount>;
    }

    function microsoftLogin() {
        invoke("show_microsoft_login_page").catch((err) => console.log(err));
    }

    function loginToAccount(uuid: string) {
        invoke("login_to_account", {uuid: uuid}).catch((err) => console.log(err));
    }

    function getAccounts(): Promise<AccountInformation> {
        return invoke("get_accounts");
    }
</script>

<div class="flex">
    <div class="accounts">
        <h1>Choose Account</h1>
        {#await getAccounts() then accounts}
            {#each Object.values(accounts.accounts) as account}
                <!-- TODO: Add skin and more information to account buttons -->
                <LoginButton on:click={() => loginToAccount(account.uuid)}>
                    {account.name}
                </LoginButton>
            {/each}
        {/await}
        <LoginButton on:click={microsoftLogin}>Add Account</LoginButton>
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
        background-color: #333;
        color: white;
        border-radius: 8px;
    }

    .accounts {
        text-align: center;
        display: flex;
        flex-direction: column;
    }

    .accounts * {
        margin: 4px;
    }

    .accounts > h1 {
        background-color: #4e4e4e;
        margin: 0px;
        margin-bottom: 4px;
        border-top-left-radius: 8px;
        border-top-right-radius: 8px;
    }
</style>
