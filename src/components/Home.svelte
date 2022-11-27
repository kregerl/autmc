<script lang="ts">
    import { link } from "svelte-navigator";
    import { getVersion, getName } from "@tauri-apps/api/app"
    import { onMount } from "svelte";
    import { resolveResource } from "@tauri-apps/api/path";

    let launcherName = "Example";
    let launcherVersion = "1.0.0";
    let username = "AreUThreateningMe";

    onMount(async () => {
        // launcherName = await getName();
        // launcherVersion = await getVersion();
    })

</script>

<nav>
    <h1>{launcherName}  <span>v{launcherVersion}</span></h1>
    <h3><img src="https://i.pinimg.com/originals/85/78/bf/8578bfd439ef6ee41e103ae82b561986.png" alt="Skin Head"/>   {username}</h3>
    <div>
        <h1>Home</h1>
        <a href="/new-instance" use:link>Add Instance</a>
    </div>
</nav>
<main>

</main>

<style>
    nav {
        position: fixed;
        top: 0;
        left: 0;
        width: 250px;
        height: 100vh;
        background: red;
    }

    nav > h1 {
        position: relative;
        text-align: center;
        margin-top: 4px;
        margin-bottom: 8px;
        font-size: 2em;
    }


    nav > h1 > span {
        font-size: 0.5em;
    }

    img {
        width: 40px;
        border-radius: 8px;
        vertical-align: text-bottom;
    }

</style>