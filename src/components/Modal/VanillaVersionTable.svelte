<script lang="ts">
    import type { VersionEntry } from "../../manifest";

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
            <tr id={entry.version} class={selected === entry.version ? "selected" : ""} on:click={setActive}>
                <td>{entry.version}</td>
                <td>{entry.versionType}</td>
            </tr>
        {/each}
    </tbody>
</table>

<style>
    /* FIXME: Scroll bars are not selectable when on the header. Scrollbars should stop at bottom of header and not go over them. */
    table {
        background-color: #4e4e4e;
        width: 100%;
        position: sticky;
        table-layout: fixed;
        border-collapse: collapse;
        border-spacing: 0px;
    }

    thead {
        position: sticky;
        top: 0;
        background-color: #4e4e4e;
        height: 8vh;
        font-size: 2vw;
    }

    th:first-child {
        border-right: 1px solid #444;
    }

    th {
        height: 8vh;
        font-size: 1.5vw;
        text-align: left;
        border-left: 1px solid #444;
    }

    td {
        font-size: 1vw;
        text-align: left;
    }

    tr {
        cursor: pointer;
    }

    tr > td, th {
        padding-left: 8px;
    }

    tbody > tr:nth-child(odd) {
        background-color: #444;
    }

    tbody > tr:nth-child(even) {
        background-color: #333;
    }
</style>
