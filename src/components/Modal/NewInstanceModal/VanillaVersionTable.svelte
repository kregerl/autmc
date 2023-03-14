<script lang="ts">
    import type { VersionEntry } from "../../../store/manifeststore";

    export let versionEntries: VersionEntry[];

    export let selected = versionEntries.at(0).version;

    function setActive() {
        for (let entry of versionEntries) {
            let element = document.getElementById(entry.version);
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
            <th>Version</th>
            <th>Type</th>
        </tr>
    </thead>
    <tbody>
        {#each versionEntries as entry}
            <tr
                id={entry.version}
                class={selected === entry.version ? "selected" : ""}
                on:click={setActive}
            >
                <td>{entry.version}</td>
                <td>{entry.versionType}</td>
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
        width: 100%;
        height: 54vh;
        overflow-y: scroll;
    }

    table tr {
        display: block;
        width: 100%;
    }

    table th,
    table td {
        text-align: left;
        width: 200px;
    }

    tbody > tr:nth-child(odd) {
        background-color: #444;
    }

    tbody > tr:nth-child(even) {
        background-color: #333;
    }

    tbody > tr {
        cursor: pointer;
    }

    tr > td,
    th {
        padding-left: 8px;
    }

    th,
    tr {
        height: 3vh;
        font-size: 1.75vmin;
    }

    ::-webkit-scrollbar {
        width: 6px;
    }

    ::-webkit-scrollbar-track {
        background: transparent;
        box-shadow: inset 0 0 5px #4e4e4e;
    }

    ::-webkit-scrollbar-thumb {
        background: #8c6ec9;
        height: 6vh;
    }
</style>
