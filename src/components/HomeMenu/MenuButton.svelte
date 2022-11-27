<script lang="ts"> 
</script>

<button on:click>
    <slot/>
</button>

<style>
    button {
        color: white;
        font-size: 1.75em;
        width: 100%;
        height: 80px;
        margin-top: 8px;
        margin-bottom: 0px;
        padding: 0px;
        border: 1px solid #333;
        border-radius: 20px;
        background-color: #4E4E4E;
    }

    button:hover {
        background-color: #5E5E5E;
        /* background-color: #8712fc; */
        /* border: 2px solid #5e5e5e; */
    }

    button:active {
        background-color: #6E6E6E;
    }
</style>