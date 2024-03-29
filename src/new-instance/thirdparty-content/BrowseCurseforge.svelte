<script context="module" lang="ts">
    export interface Author {
        name: string;
    }

    export interface Image {
        title: string;
        url: string;
    }

    export interface ModpackInformation {
        id: number;
        name: string;
        summary: string;
        downloadCount: number;
        authors: Author[];
        logo: Image;
        categories: CurseforgeCategory[];
    }
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";

    import { manifestStore } from "../../store/manifeststore";
    import {
        CurseforgeCategory,
        categoryStore,
    } from "../../store/categorystore";

    import VirtualList from "svelte-tiny-virtual-list";
    import InfiniteLoading from "svelte-infinite-loading";
    import CurseforgeModpackRow from "./CurseforgeModpackRow.svelte";
    import DropdownMenu from "../../components/dropdown/DropdownMenu.svelte";
    import TextBoxInput from "../../components/input/TextBoxInput.svelte";
    import CurseforgePreviewPage from "./curseforge/CurseforgePreviewPage.svelte";

    let selectedVersion: string;
    let selectedCategory: string;
    let selectedSort: string;

    let searchFilter: string = "";

    let page = 0;
    let listHeight: number = 0;
    let modpacks: ModpackInformation[] = [];

    $: reloadModpacks(
        searchFilter,
        selectedVersion,
        selectedCategory,
        selectedSort
    );

    const sortFields = [
        "Popularity",
        "Featured",
        "LastUpdated",
        "Name",
        "Author",
        "TotalDownload",
    ];

    function getVersionOptions(): string[] {
        let options = $manifestStore.vanilla_versions.flatMap(
            (versionEntry) => {
                if (versionEntry.versionType === "release")
                    return versionEntry.version;
                return [];
            }
        );
        return ["All Versions"].concat(options);
    }

    async function getCategoryId(selectedCategory: string): Promise<number> {
        return $categoryStore.find(
            (category) => category.name === selectedCategory
        ).id;
    }

    async function getCategories(): Promise<CurseforgeCategory[]> {
        $categoryStore = await invoke("get_curseforge_categories");
        $categoryStore.sort((a, b) =>
            a.name.localeCompare(b.name, "en", { numeric: true })
        );

        $categoryStore = [
            {
                id: 0,
                name: "All Categories",
                iconUrl: "",
            },
            ...$categoryStore,
        ];

        selectedVersion = getVersionOptions()[0];
        selectedCategory = $categoryStore.at(0).name;
        selectedSort = sortFields[0];
        return $categoryStore;
    }

    async function infiniteHandler({ detail: { loaded, complete, error } }) {
        console.log("Getting page", page);
        let newData = await invoke<ModpackInformation[]>("search_curseforge", {
            page: page,
            searchFilter: searchFilter,
            selectedVersion: selectedVersion,
            selectedCategory: await getCategoryId(selectedCategory),
            selectedSort: selectedSort,
        });

        if (newData.length) {
            page += 1;
            modpacks = [...modpacks, ...newData];
            loaded();
        } else {
            complete();
        }
    }

    async function reloadModpacks(
        searchFilter: string,
        selectedVersion: string,
        selectedCategory: string,
        selectedSort: string
    ) {
        if (
            selectedVersion !== undefined &&
            selectedCategory !== undefined &&
            selectedSort !== undefined
        ) {
            console.log("Reloaded");
            modpacks = await invoke("search_curseforge", {
                page: 0,
                searchFilter: searchFilter,
                selectedVersion: selectedVersion,
                selectedCategory: await getCategoryId(selectedCategory),
                selectedSort: selectedSort,
            });
            page = 0;
        }
    }

    let selectedModpackId: number | undefined;
    function selectModpack() {
        // TODO: Go to version selection
        selectedModpackId = this.id;
        console.log("modpackId", selectedModpackId);
    }

</script>

{#await getCategories() then categories}
    <div class="modpacks">
        <div class="header flex-row">
            <div>
                <DropdownMenu
                    bind:selected={selectedVersion}
                    options={getVersionOptions()}
                />
            </div>
            <div>
                <DropdownMenu
                    bind:selected={selectedCategory}
                    options={categories.map((category) => category.name)}
                />
            </div>
            <div>
                <DropdownMenu
                    bind:selected={selectedSort}
                    options={sortFields}
                />
            </div>
            <TextBoxInput
                id="modpacksearch"
                bind:value={searchFilter}
                label="Search Modpacks"
            />
        </div>

        <div class="list-wrapper" bind:offsetHeight={listHeight}>
            <VirtualList
                height={listHeight}
                itemCount={modpacks.length}
                itemSize={124}
            >
                <div
                    class="row-wrapper"
                    slot="item"
                    let:index
                    let:style
                    {style}
                >
                    <CurseforgeModpackRow
                        {index}
                        modpackInformation={modpacks[index]}
                        on:click={selectModpack}
                    />
                </div>
                <div slot="footer">
                    <InfiniteLoading on:infinite={infiniteHandler} />
                </div>
            </VirtualList>
        </div>
    </div>
{/await}
{#if selectedModpackId !== undefined}
    <CurseforgePreviewPage on:close={() => selectedModpackId = undefined}/>
{/if}

<style>
    .list-wrapper {
        z-index: 1;
        width: 100%;
        height: 100%;
        box-shadow: 3px 3px 10px 2px rgba(0, 0, 0, 0.5);
    }

    .modpacks {
        width: 100%;
        height: 100%;
        overflow-y: hidden;
    }

    .header {
        background-color: red;
        height: 48px;
    }

    .row-wrapper {
        width: 100%;
        height: 100%;
    }
</style>
