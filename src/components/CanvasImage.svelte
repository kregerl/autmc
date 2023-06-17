<script lang="ts">
    import { onMount } from "svelte";
    import { canvasCache } from "../store/canvascache";

    export let src: string;
    export let size: [number, number];

    let canvas: HTMLCanvasElement;

    onMount(async () => {
        if ($canvasCache === undefined) {
            $canvasCache = new Map();
        }
        if ($canvasCache.has(src)) {
            const cachedCanvas = $canvasCache.get(src);

            let ctx = canvas.getContext("2d");
            canvas.width = cachedCanvas.width;
            canvas.height = cachedCanvas.height;
            ctx.drawImage(cachedCanvas, 0, 0);
        } else {
            const tmpCanvas = document.createElement("canvas");
            const tmpCtx = tmpCanvas.getContext("2d");

            tmpCanvas.width = size[0];
            tmpCanvas.height = size[1];

            const image = new Image();
            image.src = src;
            image.onload = () => {
                console.log("onload");
                const horizontalRatio = tmpCanvas.width / image.width;
                const verticalRatio = tmpCanvas.height / image.height;

                const ratio = Math.min(horizontalRatio, verticalRatio);
                const xShift = (tmpCanvas.width - image.width * ratio) / 2;
                const yShift = (tmpCanvas.height - image.height * ratio) / 2;
                tmpCtx.drawImage(
                    image,
                    0,
                    0,
                    image.width,
                    image.height,
                    xShift,
                    yShift,
                    image.width * ratio,
                    image.height * ratio
                );

                $canvasCache.set(src, tmpCanvas);

                let ctx = canvas.getContext("2d");
                canvas.width = tmpCanvas.width;
                canvas.height = tmpCanvas.height;
                ctx.drawImage(tmpCanvas, 0, 0);
            };
        }
    });
</script>

<canvas bind:this={canvas} />