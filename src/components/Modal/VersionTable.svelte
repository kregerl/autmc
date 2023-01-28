<script context="module" lang="ts">
    export interface Row {
        id: string;
        entries: string[];
    }
</script>

<script lang="ts">
    export let headers: Row;
    export let body: Row[];

    export let selected = body.at(0).id;

    function setActive() {
        for (let entry of body) {
            let element = document.getElementById(entry.id);
            if (element.classList.contains("selected"))
                element.classList.remove("selected");
        }
        this.classList.add("selected");
        selected = this.id;
    }
</script>

<table>
    <thead>
        <tr>
            {#each headers.entries as header}
                <th>{header}</th>
            {/each}
        </tr>
    </thead>
    <tbody>
        {#each body as row}
            <tr
                id={row.id}
                class={selected === row.id ? "selected" : ""}
                on:click={setActive}
            >
                {#each row.entries as entry}
                    <td>{entry}</td>
                {/each}
            </tr>
        {/each}
    </tbody>
</table>

<style>
    table {
        width: 100%;
        table-layout: fixed;
        border-collapse: collapse;
    }

    thead tr th {
        text-align: left;
        background-color: #4e4e4e;
    }

    table tbody {
        display: block;
        overflow-y: scroll;
        width: 100%;
        height: 54vh;
    }

    table tr {
        display: block;
        width: 100%;
    }

    table th,
    table td {
        text-align: left;
        width: 400px;
    }

    th,
    tr {
        height: 3vh;
        font-size: 1.75vmin;
    }

    tbody > tr {
        cursor: pointer;
    }

    tr > td,
    th {
        padding-left: 8px;
    }

    tbody > tr:nth-child(odd) {
        background-color: #444;
    }

    tbody > tr:nth-child(even) {
        background-color: #333;
    }
</style>
