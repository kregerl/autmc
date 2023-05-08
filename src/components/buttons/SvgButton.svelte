<script lang="ts" context="module">
    export enum Order {
        ImgFirst,
        ImgLast,
    }
</script>

<script lang="ts">
    export let order: Order = Order.ImgLast;
    export let src: string;
    export let alt: string;
</script>

<button class="button flex-row" on:click>
    {#if order === Order.ImgFirst}
        <img {src} {alt} />
        <span class="high-emphasis">{alt}</span>
    {:else}
        <span class="high-emphasis">{alt}</span>
        <img {src} {alt} />
    {/if}
</button>

<style>
    button {
        line-height: 24px;
        font-size: 1.8rem;
        padding: 6px;
        width: fit-content;
        border: none;
        background-color: var(--dark-black);
        transition: 0.15s linear;
        box-shadow: 3px 3px 10px 2px rgba(0, 0, 0, 0.5);
        color: white;
        margin: 8px;
        border-radius: 4px;
        cursor: pointer;
    }

    button:hover {
        background-color: var(--light-black);
    }

    img {
        transform: rotate(var(--img-rotation));
        width: var(--img-width, 24px);
        height: 24px;
        margin-left: 4px;
    }
</style>
