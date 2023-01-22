<script lang="ts">
    import type { VersionEntry } from "../../manifest";

    export let versionEntries: VersionEntry[];
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
            <tr>
                <td>{entry.version}</td>
                <td>{entry.versionType}</td>
            </tr>
        {/each}
    </tbody>
</table>

<style>
    table {
        background-color: #4e4e4e;
        width: 100%;
        position: sticky;
        table-layout: fixed;
        border-spacing: 0px;
    }

    thead {
        position: sticky;
        top: 0;
        background-color: #4e4e4e;
        height: 8vh;
        font-size: 2vh;
    }

    th {
        height: 8vh;
        font-size: 2.6vh;
        text-align: left;
    }

    td {
        font-size: 2vh;
        text-align: left;
    }
</style>
