<script lang="ts">
    import { navigate } from "svelte-navigator";
    import SvgButton, { Order } from "../buttons/SvgButton.svelte";

    function back() {
        navigate(-1);
    }
</script>

<main>
    Hello

    <div class="settings">Test</div>

    <div class="footer flex-row">
        <SvgButton
            order={Order.ImgFirst}
            src="svg/RightArrow.svg"
            alt="Back"
            --img-rotation="180deg"
            on:click={back}
        />
        <SvgButton src="svg/RightArrow.svg" alt="Next" />
    </div>
</main>

<style>
    main {
        background-color: var(--medium-black);
        width: 100%;
        height: 100%;
    }

    .settings {
        height: calc(100% - 120px);
    }

    .footer {
        position: absolute;
        right: 0;
        bottom: 0;
        justify-content: space-between;
        margin: 0 16px 16px 0;
    }
</style>
