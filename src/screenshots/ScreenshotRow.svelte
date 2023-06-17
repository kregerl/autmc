<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { slide } from "svelte/transition";

    export let instance: string;
    export let screenshots: string[];
    $: formattedScreenshots = screenshots.map((value) => convertFileSrc(value));

    let shown: boolean = true;
    function hideElements() {
        console.log("HERE");
        shown = !shown;
    }
</script>

<div class="header flex-row">
    <label>
        <input id={instance} type="checkbox" on:click={hideElements} />
        <img id="caret" class="high-emphasis" src="svg/Caret.svg" alt="Caret" />
    </label>
    <h3 class="high-emphasis">{instance}</h3>
</div>

{#if shown}
    <div class="images" out:slide={{ duration: 350 }}>
        {#each formattedScreenshots as screenshot}
            <img src={screenshot} alt={screenshot} on:click on:keydown />
        {/each}
    </div>
{/if}

<style>
    .header {
        align-items: center;
        margin: 0 8px 0 8px;
    }

    input[type="checkbox"] {
        display: none;
    }

    #caret {
        margin-top: auto;
        justify-self: center;
        transition: all 0.25s;
        transform: rotate(90deg);
        z-index: 1;
    }

    input[type="checkbox"]:checked + #caret {
        transform: rotate(0);
    }

    h3 {
        color: white;
        margin: 0 0 2px 8px;
    }

    .images {
        display: grid;
        grid-template-columns: repeat(5, minmax(0, 1fr));
        gap: 8px;
        margin: 0 8px 0 8px;
    }

    .images > img {
        object-fit: cover;
        width: 100%;
        height: 100%;
        cursor: pointer;
    }

    .images > img:hover {
        opacity: 0.8;
    }
</style>
