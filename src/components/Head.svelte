<script lang="ts">
    import { onMount } from "svelte";

    export let skinUrl: string;

    let canvas: HTMLCanvasElement;

    onMount(async () => {
        const image = new Image(64, 64);

        image.src = skinUrl;

        const context = canvas.getContext("2d");
        canvas.width = image.width;
        canvas.height = image.height;
        context.imageSmoothingEnabled = false;

        image.onload = () => {
            context.drawImage(
                image,
                8,
                8,
                8,
                8,
                0,
                0,
                image.width,
                image.height
            );
        };
    });
</script>

<canvas bind:this={canvas}/>