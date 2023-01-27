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
    /* FIXME: Scroll bars are not selectable when on the header. Scrollbars should stop at bottom of header and not go over them. */
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

    tr {
        cursor: pointer;
    }

    tbody > tr:nth-child(odd) {
        background-color: #444;
    }

    tbody > tr:nth-child(even) {
        background-color: #333;
    }
</style>
