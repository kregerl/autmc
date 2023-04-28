<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import AccountContainer from "./AccountContainer.svelte";
    import Head from "../Head.svelte";
    import Loader from "../Loader.svelte";

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

    const promise: Promise<AccountInformation> = invoke("get_accounts");
</script>

<div />
<AccountContainer>
    <h1 class="high-emphasis">Switch Accounts</h1>

    {#await promise then accountInfo} 
        {#each Object.values(accountInfo.accounts) as account, index}
            <div id={accountInfo.active_account === account.uuid ? "active-account" : ""} class="button account">
                <Head skinUrl={account.skin_url} />
                <div class="flex-col text">
                    <span class="high-emphasis" id="account-name">{account.name}</span>
                    <span class="medium-emphasis" id="account-uuid">{account.uuid}</span>
                </div>
            </div>
        {/each}
    {/await}

    <div class="button add-account">
        <img src="svg/PlusSign.svg" alt="Plus sign">
        <p class="high-emphasis">Add Account</p>    
    </div>
</AccountContainer>

<style>
    h1, p, span {
        color: white;
    }

    h1 {
        text-align: center;
        font-weight: bold;
        font-size: 3.2rem;
    }

    .button {
        margin: 12px;
        padding: 12px;
        border-radius: 8px;
        border: 2px solid #3D2E5B;
        background-color: #573993;
        display: flex;
        justify-content: center;
    }

    .button:hover {
        cursor: pointer;
        background-color: #6A51B9;
    }

    .add-account > p {
        font-size: 2.2rem;
    }

    .add-account > img {
        margin-left: 8px;
        margin-right: 8px;
        width: 2.2rem;
        aspect-ratio: 1;
    }

    .text {
        margin-left: 12px;
    }

    #account-name {
        font-size: 2.2rem;
    }

    #account-uuid {
        font-size: 1.4rem;
    }

    #active-account {
        border: 2px solid #5EB3E8;
    }
</style>
