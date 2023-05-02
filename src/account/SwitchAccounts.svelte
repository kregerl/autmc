<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";

    import AccountContainer from "./AccountContainer.svelte";
    import Head from "../components/Head.svelte";
    import { navigate } from "svelte-navigator";

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

    function goBack() {
        navigate(-1);
    }
</script>

<div class="back-button flex-row" on:click={goBack} on:keydown>
    <img src="svg/LeftArrow.svg" alt="Back">
    <h3>Back</h3>
</div>
<AccountContainer>
    <h1 class="high-emphasis">Switch Accounts</h1>

    {#await promise then accountInfo}
        {#each Object.values(accountInfo.accounts) as account}
            <div
                id={accountInfo.active_account === account.uuid
                    ? "active-account"
                    : ""}
                class="button account"
            >
                <Head skinUrl={account.skin_url} />
                <div class="flex-col text">
                    <span class="high-emphasis" id="account-name"
                        >{account.name}</span
                    >
                    <span class="medium-emphasis" id="account-uuid"
                        >{account.uuid}</span
                    >
                </div>
            </div>
        {/each}
    {/await}

    <div class="button add-account">
        <img src="svg/PlusSign.svg" alt="Plus sign" />
        <p class="high-emphasis">Add Account</p>
    </div>
</AccountContainer>

<style>
    h1,
    p,
    span {
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
        border: 2px solid #3d2e5b;
        background-color: #573993;
        display: flex;
        justify-content: center;
    }

    .button:hover {
        cursor: pointer;
        background-color: #6a51b9;
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
        border: 2px solid #5eb3e8;
    }

    .back-button {
        position: absolute;
        justify-content: flex-start;
        color: white;
        width: 100px;
        height: 25px;
        text-align: center;
        line-height: 25px;
        margin: 8px 0 0 8px;
        border-radius: 4px;
        cursor: pointer;
        font-size: 1.4rem;
        border: 2px solid var(--medium-black);
        background-color: var(--light-black);
        box-shadow: 3px 3px 10px 2px rgba(0, 0, 0, 0.75);
    }

    .back-button > h3 {
        margin: 0;
        margin-left: 12px;
    }
</style>
