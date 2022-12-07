<script lang="ts">
    export let text;
</script>

<div class="tab" id={text.toLowerCase()} on:click on:keydown>
    <h3 class="dropshadow">{text}</h3>
</div>

<style>
    .tab {
        padding: 8px;
        min-width: var(--min-width);
        width: var(--width);
        background-color: #333;
        text-align: center;
    }

    h3 {
        color: white;
        font-size: var(--font-size);
    }
</style>