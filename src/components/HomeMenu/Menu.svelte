<script lang="ts">
    import { getName, getVersion } from "@tauri-apps/api/app";
    import { onMount } from "svelte";
    import { link } from "svelte-navigator";
    import MenuButton from "./MenuButton.svelte";

    let launcherName: string = "Example";
    let launcherVersion: string = "1.0.0";
    let username: string = "AreUThreateningMe";

    onMount(async () => {
        launcherName = await getName();
        launcherVersion = await getVersion();
    })

</script>

<nav>
    <div class="launcher-header">
        <div class="title">
            <h1 class="dropshadow">{launcherName}</h1>
            <p>v{launcherVersion}</p>
        </div>
    
        <div class="image-content">
            <img src="https://i.pinimg.com/originals/85/78/bf/8578bfd439ef6ee41e103ae82b561986.png" alt="Skin Head"/>
            <h3 class="dropshadow">{username}</h3>
        </div>
    
        <div class="image-content-small">
            <img src="SwitchUser.svg" alt="Switch User">
            <h3 class="dropshadow">Switch User</h3>
        </div>
    
        <div class="image-content-small">
            <img src="Settings.svg" alt="Switch User">
            <h3 class="dropshadow">Launcher Settings</h3>
        </div>
    </div>
    <div class="menu-buttons">
        <MenuButton><span class="dropshadow">Instances</span></MenuButton>
        <MenuButton><span class="dropshadow">Servers</span></MenuButton>
        <MenuButton><span class="dropshadow">Screenshots</span></MenuButton>
    </div>

    <div>
        <h1>Home</h1>
        <a href="/new-instance" use:link>Add Instance</a>
    </div>
</nav>

<style>
    nav {
        width: 350px;
        height: 100vh;
        background: #333;
    }

    .launcher-header {
        padding-top: 8px;
        padding-bottom: 4px;
        margin-bottom: 4px;
        background-color: #4E4E4E;
        box-shadow: 0px 6px 6px #111;
    }

    .image-content {
        width: 350px;
        overflow: hidden;
    }

    .image-content img {
        float: left;
        width: 40px;
        border-radius: 8px;
        margin-left: 24px;
        margin-right: 8px;
    }

    .image-content h3 {
        color: white;
        font-size: 1.25em;
    }

    .image-content-small {
        width: 350px;
        overflow: hidden;
        margin-bottom: 8px;
    }

    .image-content-small * {
        margin: 0px;
    }

    .image-content-small img {
        float: left;
        width: 24px;
        height: 24px;
        margin-left: 24px;
        margin-right: 8px;
    }

    .image-content-small h3 {
        color: white;
        font-size: 0.85em;
        line-height: 24px;
    }

    .title {
        width: 350px;
        overflow: hidden;
        text-align: center;
        margin-bottom: 16px
    }

    .title * {
        display: inline;
        vertical-align: bottom;
        margin: 0px;
    }

    .title p {
        font-size: 0.75em;
        color: rgb(26, 26, 26);
    }

    .title h1 {
        font-size: 3em;
        margin-right: 4px;
        color: white;
    }

</style>