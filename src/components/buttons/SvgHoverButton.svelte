<script lang="ts">
    export let src: string;
    export let alt: string;
    export let emphasis: string = "high-emphasis";
    export let active: boolean = false;
</script>

<div class="wrapper" on:click on:keydown>
    <img class="{emphasis}  {active ? "active" : ""}" {src} {alt} />
    <div class="circle-hover" />
</div>

<style>
    img {
        z-index: 3;
        position: absolute;
        width: var(--svg-size);
        height: var(--svg-size);
        margin-top: var(--margin-top);
        margin-left: var(--margin-left);
        padding: var(--padding);
        cursor: pointer;
    }

    .active {
        box-shadow: -4px 0px #573993;
    }

    .wrapper {
        width: var(--hover-size);
        height: var(--hover-size);
    }

    .circle-hover {
        z-index: 2;
        position: absolute;
        width: var(--hover-size);
        height: var(--hover-size);
        margin-top: var(--margin-top);
        margin-left: var(--margin-left);
        background-color: var(--light-black);
        border-radius: 50%;
        transform: translate(
            calc(-1 * var(--padding) / 2),
            calc(-1 * var(--padding) / 2)
        );
        display: none;
    }

    img:hover + .circle-hover {
        display: block;
    }
</style>
