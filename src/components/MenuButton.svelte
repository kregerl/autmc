<script lang="ts">
</script>

<button class="menu-button" on:click>
    <slot class="menu-button-text"/>
</button>

<style>
    .menu-button {
        color: white;
        background-color: #3E3E3E;
        width: 352px;
        height: 80px;
        border-radius: 20px;
        font-size: 36px;
        text-align: center;
        text-shadow: 2px 4px 4px #FFFFFF80;
    }
</style>