<script lang="ts">
    export let headers: string[];
    export let body: string[];
</script>

<table>
    <thead>
        <tr>
            {#each headers as header}
                <th>{header}</th>
            {/each}
        </tr>
    </thead>
    <tbody>
        {#each body as row}
            <tr>
                <td>{row}</td>
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
        height: var(--header-height);
        font-size: var(--font-size);
    }

    th {
        height: var(--header-height);
        font-size: var(--font-size);
        text-align: left;
    }

    td {
        font-size: var(--font-size);
        text-align: left;
    }
</style>
