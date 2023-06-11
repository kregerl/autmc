<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";

    import DropdownMenu from "../../dropdown/DropdownMenu.svelte";
    import { manifestStore } from "../../../store/manifeststore";
    import TextBoxInput from "../../input/TextBoxInput.svelte";
    import {
        CurseforgeCategory,
        categoryStore,
    } from "../../../store/categorystore";

    import VirtualList from "svelte-tiny-virtual-list";
    import InfiniteLoading from "svelte-infinite-loading";

    let selectedVersion: string;
    let selectedCategory: string;
    let selectedSort: string;

    let searchFilter: string = "";

    let page = 0;
    let listHeight: number = 0;
    let modpacks: string[] = [];

    $: reloadModpacks(searchFilter, selectedVersion, selectedCategory, selectedSort);

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
        let newData = await invoke<string[]>("search_curseforge", {
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
                itemSize={24}
            >
                <h1 slot="item" let:index let:style {style}>
                    {modpacks[index]}
                </h1>

                <div slot="footer">
                    <InfiniteLoading on:infinite={infiniteHandler} />
                </div>
            </VirtualList>
        </div>
    </div>
{/await}

<style>
    .list-wrapper {
        z-index: 1;
        width: 100%;
        height: 100%;
    }

    .modpacks {
        width: 100%;
        height: 100%;
    }

    .header {
        background-color: red;
        height: 48px;
    }

    h1 {
        color: white;
    }
</style>
