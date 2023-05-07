<script lang="ts">
    export let src: string;
    export let alt: string;
</script>

<button class="button flex-row" on:click>
    <span class="high-emphasis">{alt}</span>
    <img {src} {alt} />
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
        width: var(--img-width, 24px);
        height: 24px;
        margin-left: 4px;
    }
</style>
