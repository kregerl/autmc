<script lang="ts">
    import type { ModpackInformation } from "./BrowseCurseforge.svelte";
    import CanvasImage from "../../components/CanvasImage.svelte";
    import { formatDownloads } from "../../downloadfmt";

    export let modpackInformation: ModpackInformation;
    export let index: number;

    $: sortedCateogories = modpackInformation.categories.sort((a, b) =>
        a.name.localeCompare(b.name, "en", { numeric: true })
    );

    $: authors = modpackInformation.authors
        .map((author) => author.name)
        .join(", ");
</script>

<div
    id={modpackInformation.id.toString()}
    class="row flex-row {index % 2 == 0 ? 'even' : 'odd'}"
    on:click
    on:keydown
>
    <div class="img-wrapper">
        <CanvasImage src={modpackInformation.logo.url} size={[100, 100]} />
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
        {#each sortedCateogories as category}
            <div class="category flex-row">
                <CanvasImage src={category.iconUrl} size={[32, 32]} />
                <p class="high-emphasis">{category.name}</p>
            </div>
        {/each}
    </div>
</div>

<style>
    #test {
        width: 32px;
        height: 32px;
    }
    img {
        width: 100px;
        height: 100px;
    }

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

    .summary {
        word-wrap: break-word;
    }
</style>
