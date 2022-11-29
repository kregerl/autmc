<script lang="ts">

</script>
<div class="content">
    <div class="content-header">

    </div>
    <div class="instances">
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
    </div>

    <div class="new-instance">
        <img src="PlusSign.svg" alt="Add new instance">
    </div>
</div>


<style> 

    .content {
        height: 0;
        background-color: grey;
    }

    .content-header {
        height: 10%;
        background-color: blue;
    }

    .instances {
        overflow: scroll;
    }

    .instance-grid {
        height: auto;
        display: grid;
        grid-template-columns: repeat(5, minmax(0, 1fr));
        gap: 16px;
        margin: 24px;
        overflow: scroll;
    }

    .instance {
        width: 100%;
        min-height: 180px;
        background-color: red;
        border-radius: 12px;
    }

    .new-instance {
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
    }

</style>