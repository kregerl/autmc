<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/tauri";

    onMount(async () => {
        invoke("obtain_manifests")
            .then((payload) => console.log(payload))
            .catch((err) => console.log(err));
    });

</script>

<div>
    <p>New Instance Screen!</p>
</div>

<style>

</style>