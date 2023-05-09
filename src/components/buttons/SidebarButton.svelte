<script lang="ts">
    export let text: string;
    export let src: string;
    export let active: boolean = false;
</script>

<div
    id={text}
    class="button flex-row {active ? 'active' : ''}"
    on:click
    on:keydown>
    <img class="high-emphasis" {src} alt={text} />
    <h1 class="high-emphasis">{text}</h1>
</div>

<style>
    h1 {
        color: white;
    }

    img {
        width: 33px;
        height: 33px;
        padding: 7.5px;
    }

    .button {
        width: 100%;
        height: 50px;
        align-items: center;
        margin-bottom: 6px;
        border-radius: 8px;
        cursor: pointer;
    }

    .button:hover {
        background-color: var(--light-black);
    }

    .active {
        box-shadow: -4px 0px var(--medium-purple);
        border-top-left-radius: 0px;
        border-bottom-left-radius: 0px;
    }
</style>
