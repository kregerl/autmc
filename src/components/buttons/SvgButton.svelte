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
    export let disabled: boolean = false;
</script>

<button class="button flex-row" {disabled} on:click>
    {#if order === Order.ImgFirst}
        <img class="{disabled ? "low-emphasis" : "high-emphasis"}" {src} {alt} />
        <span class="{disabled ? "low-emphasis" : "high-emphasis"}">{alt}</span>
    {:else}
        <span class="{disabled ? "low-emphasis" : "high-emphasis"}">{alt}</span>
        <img class="{disabled ? "low-emphasis" : "high-emphasis"}" {src} {alt} />
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

    button:hover:enabled {
        background-color: var(--light-black);
    }

    button:disabled {
        cursor: not-allowed;
    }

    img {
        transform: rotate(var(--img-rotation));
        width: var(--img-width, 24px);
        height: 24px;
        margin-left: 4px;
    }
</style>
