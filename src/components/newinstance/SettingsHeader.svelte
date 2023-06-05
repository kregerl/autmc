<script lang="ts">
    import { Emphasis, classFromEmphasis } from "../../menu";

    export let text: string;
    export let emphasis: Emphasis = Emphasis.High;
</script>

<div class="header">
    <h1 class={classFromEmphasis(emphasis)}>{text}</h1>
</div>

<style>
    h1 {
        color: white;
    }
</style>
