<script lang="ts">
    import AccountContainer from "./AccountContainer.svelte";

</script>

<AccountContainer>
    <h1 class="high-emphasis">AUTM Launcher</h1>

    <!-- TODO: Make these buttons actually do something. -->
    <div class="mslogin">
        <p class="high-emphasis">Login With</p>
        <img class="high-emphasis" src="svg/microsoft.svg" alt="Pixel art minecraft background">
        <p class="high-emphasis">Microsoft</p>
    </div>

    <div class="mclogin">
        <p class="low-emphasis">Login With Mojang</p>
    </div>
</AccountContainer>

<style>
    h1 {
        text-align: center;
        font-weight: bold;
        font-size: 3.2rem;
        color: white;
    }

    .mclogin {
        margin: 12px;
        padding: 12px;
        border-radius: 8px;
        background-color: #3D2E5B;
        display: flex;
        justify-content: center;
    }

    .mclogin:hover {
        cursor: not-allowed;
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