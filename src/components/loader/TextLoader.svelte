<script lang="ts">
    export let text: string = "Loading...";
</script>


<div class="loader">
    {#each text as c, i}
        <div class="text" style="--order: {i + 1}">{c}</div>
    {/each}
</div>

<style>
    .loader {
        --line-height: 3.2rem;
        background-color: var(--color, var(--dark-black));
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .loader > div.text {
        height: var(--line-height);
        width: var(--line-height);
        font-size: var(--line-height);
        line-height: var(--line-height);
        text-align: center;
        color: white;
        opacity: 88%;
        animation: loading 0.6s infinite alternate;
        animation-delay: calc(var(--order) * 100ms);
    }

    @keyframes loading {
        to {
            transform: translateY(calc(-1 * var(--line-height)));
        }
        from {
            text-shadow: 2px 6px 4px var(--light-purple);
        }
    }
</style>