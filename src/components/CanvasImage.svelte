<script lang="ts">
    import { onMount } from "svelte";
    import { canvasCache } from "../store/canvascache";

    export let src: string;
    export let size: [number, number];
    
    let canvas: HTMLCanvasElement;
    const image = new Image();

    $: if (src) test();

    function test() {
        image.src = src;
    }

    function reloadCanvas(imgSrc: string) {
        if (canvas === undefined)
            return;

        if ($canvasCache === undefined) {
            $canvasCache = new Map();
        }
        if ($canvasCache.has(imgSrc)) {
            const cachedCanvas = $canvasCache.get(imgSrc);

            let ctx = canvas.getContext("2d");
            canvas.width = cachedCanvas.width;
            canvas.height = cachedCanvas.height;
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            ctx.drawImage(cachedCanvas, 0, 0);
        } else {
            const tmpCanvas = document.createElement("canvas");
            const tmpCtx = tmpCanvas.getContext("2d");

            tmpCanvas.width = size[0];
            tmpCanvas.height = size[1];

            image.src = imgSrc;
            image.onload = () => {
                console.log("onload");
                const horizontalRatio = tmpCanvas.width / image.width;
                const verticalRatio = tmpCanvas.height / image.height;

                const ratio = Math.min(horizontalRatio, verticalRatio);
                const xShift = (tmpCanvas.width - image.width * ratio) / 2;
                const yShift = (tmpCanvas.height - image.height * ratio) / 2;
                tmpCtx.clearRect(0, 0, tmpCanvas.width, tmpCanvas.height);
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

                $canvasCache.set(imgSrc, tmpCanvas);

                if (canvas) {
                    let ctx = canvas.getContext("2d");
                    canvas.width = tmpCanvas.width;
                    canvas.height = tmpCanvas.height;
                    ctx.drawImage(tmpCanvas, 0, 0);
                }
            };
        }
    }

    // FIXME: This breaks gifs and any other video. Only the first frame is drawn but for now its good enough
    onMount(() => {
        reloadCanvas(src);

    });
</script>

<canvas id={src} bind:this={canvas} />
