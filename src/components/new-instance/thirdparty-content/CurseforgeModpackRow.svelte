<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { formatDownloads } from "../../../downloadfmt";
    import type { ModpackInformation } from "./BrowseCurseforge.svelte";
    import { formatImageUrl } from "../../../image";

    export let modpackInformation: ModpackInformation;
    export let index: number;

    $: authors = modpackInformation.authors
        .map((author) => author.name)
        .join(", ");
</script>

<!-- 
Webkitgtk is always going to perform worse, but first thing you might wanna look into is making the images an appropriate size for where they are being displayed. 
You can use the Rust backend in a couple different ways to provide image optimization at runtime. For example, you can add an image:// protocol to your app then 
get images using image://some/image?width=100&height=100 and have the Rust backend resize that image for you blazingly fast™️

Another "solution" is to just sit tight until we finish some alternative webview alternative for Linux. We're looking into both Chromium and Servo for it. So this issue wont' be there forever
 -->

<div
    id={modpackInformation.id.toString()}
    class="row flex-row {index % 2 == 0 ? 'even' : 'odd'}"
    on:click
    on:keydown
>
    <div class="img-wrapper">
        <img
            class="logo"
            src={formatImageUrl(modpackInformation.logo.url, 100, 100)}
            alt={modpackInformation.logo.title}
            width="100"
            height="100"
        />
        <span class="downloads high-emphasis"
            >{formatDownloads(modpackInformation.downloadCount)}</span
        >
    </div>
    <div class="info-wrapper">
        <h1 class="high-emphasis">{modpackInformation.name}</h1>
        <h3 class="high-emphasis">Created By: {authors}</h3>
        <p class="summary high-emphasis">{modpackInformation.summary}</p>
    </div>
    <div class="categories-wrapper flex-row">
        {#each modpackInformation.categories as category}
            <div class="category flex-row">
                <!-- <img src={category.iconUrl} alt={category.name}/> -->
                <p class="high-emphasis">{category.name}</p>
            </div>
        {/each}
    </div>
</div>

<style>
    .odd {
        background-color: var(--medium-light-black);
    }

    .even {
        background-color: var(--medium-black);
    }

    .row {
        align-items: center;
        color: white;
        /* background-color: red; */
        height: 100%;
        cursor: pointer;
    }

    .row:hover {
        background-color: var(--lightest-black);
    }

    .img-wrapper {
        margin: 12px;
    }

    .img-wrapper > span {
        position: absolute;
        background-color: var(--dark-black);
        color: white;
        top: 16px;
        left: 16px;
        padding: 4px;
        font-size: 1.2rem;
    }

    .info-wrapper {
        height: 100%;
        max-width: 50%;
    }

    .info-wrapper > h1 {
        margin: 8px 0 0 0;
    }

    .info-wrapper > h3 {
        margin-top: 4px;
    }

    .info-wrapper > p {
        font-size: 1.2rem;
    }

    .categories-wrapper {
        position: absolute;
        top: 0;
        right: 0;
        margin: 4px 12px 0 8px;
        height: 100%;
        align-items: start;
    }

    .category {
        margin: 4px;
        height: fit-content;
    }

    .category > p {
        height: 24px;
        line-height: 24px;
        margin: 4px;
        font-size: 1.2rem;
    }

    .category > img {
        margin: 4px;
        width: 24px;
        height: 24px;
        aspect-ratio: 1;
    }

    .logo {
        width: 100px;
        height: 100px;
        aspect-ratio: 1;
    }

    .summary {
        word-wrap: break-word;
    }
</style>
