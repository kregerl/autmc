<script lang="ts">
</script>

<div class="loader medium-emphasis" />

<style>
    .loader {
        border: 4px solid white;
        border-left-color: transparent;
        border-radius: 50%;
        width: 36px;
        height: 36px;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        0% {
            transform: rotate(0deg);
        }

        100% {
            transform: rotate(360deg);
        }
    }
</style>
