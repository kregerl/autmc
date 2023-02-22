<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { onMount } from "svelte";
    import Head from "./Head.svelte";

    interface BasicAccount {
        uuid: string;
        name: string;
        skin_url: string;
    }

    interface AccountInformation {
        active_account: string;
        // <UUID, account>
        accounts: Map<string, BasicAccount>;
    }

    let accountInfo: AccountInformation;

    function microsoftLogin() {
        invoke("show_microsoft_login_page").catch((err) => console.log(err));
    }

    function loginToAccount(uuid: string) {
        invoke("login_to_account", { uuid: uuid }).catch((err) =>
            console.log(err)
        );
    }

    function getAccounts(): Promise<AccountInformation> {
        return invoke("get_accounts");
    }

    onMount(async () => {
        accountInfo = await getAccounts();
    });
</script>

<div class="flex">
    <div class="accounts">
        <h1>Choose Account</h1>
        {#if accountInfo}
            {#each Object.values(accountInfo.accounts) as account, index}
                <div
                    id={accountInfo.active_account === account.uuid ? "active" : ""}
                    class="button account"
                    on:click={() => loginToAccount(account.uuid)}
                    on:keydown
                >
                    <Head skinUrl={account.skin_url} />
                    <div class="text">
                        <span id="account-name">{account.name}</span>
                        <span id="account-uuid">{account.uuid}</span>
                    </div>
                </div>
            {/each}
        {/if}
        <button class="button add-account" on:click={microsoftLogin}>
            Add Account
        </button>
    </div>
</div>

<style>
    .flex {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        height: 100%;
        background-color: #333;
    }

    .flex div.accounts {
        color: white;
        border-radius: 8px;
        border: 2px solid #4e4e4e;
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
        border-top-left-radius: 6px;
        border-top-right-radius: 6px;
    }

    .button {
        background-color: #4e4e4e;
        cursor: pointer;
    }

    .button:hover {
        background-color: #5e5e5e;
    }

    .button:active {
        background-color: #6e6e6e;
    }

    .add-account {
        color: white;
        width: 20vw;
        height: 8vh;
        border-radius: 16px;
        border: none;
        font-size: 4vmin;
        text-align: center;
        margin: 4px;
    }

    .account {
        display: flex;
        flex-direction: row;
        color: white;
        width: 20vw;
        height: 8vh;
        border-radius: 16px;
        border: none;
        font-size: 2.5vmin;
        text-align: center;
        margin: 4px;
    }

    .account :global(canvas) {
        margin: 8px;
    }

    #active {
        border: 1px solid gold;
    }

    .text {
        display: flex;
        flex-direction: column;
        align-items: start;
    }

    .text > * {
        margin-left: 8px;
    }

    #account-uuid {
        margin-top: 0px;
        font-size: 1vmin;
    }
</style>
