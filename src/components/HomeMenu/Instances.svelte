<script lang="ts">
    import { navigate } from "svelte-navigator";

    function createNewInstance() {
        navigate("/new-instance");
    }


</script>

<div class="header">
    <input type="image" src="PlusSign.svg" alt="New Instance" on:click={createNewInstance}>
    <input type="text" placeholder="Search Instances">
</div>
<div class="instance-grid">
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
    <div class="instance"></div>
</div>


<style> 
    .header {
        display: flex;
        align-items: center;
        position: sticky;
        top: 0;
        background-image: linear-gradient(rgba(51, 51, 51, 1.0), rgba(51, 51, 51, 0.5));
        width: 100%;
        height: 60px;
    }

    .instance-grid {
        height: auto;
        display: grid;
        grid-template-columns: repeat(5, minmax(0, 1fr));
        gap: 16px;
        margin: 24px;
        overflow-y: scroll;
    }

    .instance {
        width: 100%;
        min-height: 180px;
        background-color: red;
        border-radius: 12px;
    }

    input[type=image] {
        height: calc(100% - 8px);
        margin: 4px;
        background-color: #4e4e4e;
        border: 1px solid #333
    }

    input[type=text] {
        height: calc(100% - 8px);
        margin: 4px;
        margin-left: 8px;
        padding: 0px;
        background-color: rgba(0, 0, 0, 0);
        border: none;
    }

    input[type=text]::placeholder {
        color: white;
        font-weight: bold;
    }
    /* .new-instance {
        position: absolute;
        right: 0;
        bottom: 0;
        margin: 16px;
        width: 35px;
        height: 35px;
        background-color: #333;
        border-radius: 8px;
        box-shadow: 0px 2px 8px 4px black;
        border: 1px solid black;
    } */

</style>