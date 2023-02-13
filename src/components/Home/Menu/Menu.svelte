<script lang="ts">
    import { getName, getVersion } from "@tauri-apps/api/app";
    import { invoke } from "@tauri-apps/api/tauri";
    import { onMount } from "svelte";
    import MenuNavbar from "./MenuNavbar.svelte";

    export let selectedTab: string;
    let launcherName: string = "Example";
    let launcherVersion: string = "1.0.0";
    let username: string = "AreUThreateningMe";

    const buttons = [
        "Instances",
        "Servers",
        "Screenshots",
        "Logs"
    ];

    function getSkin(): Promise<string> {
        return new Promise((resolve) => {
            setTimeout(async () => {
                resolve(await invoke("get_account_skin") as string)
            });
        });
    }

    onMount(async () => {
        launcherName = await getName();
        launcherVersion = await getVersion();

        const image = new Image(64, 64);

        let skin = await getSkin();
        image.src = skin;
       
        const canvas = document.getElementById("head-canvas") as HTMLCanvasElement;
        const context = canvas.getContext("2d");
        canvas.width = image.width;
        canvas.height = image.height;
        context.imageSmoothingEnabled = false;

        image.onload = () => {
            context.drawImage(image, 8, 8, 8, 8, 0, 0, image.width, image.height);
        }
    });
</script>

<nav>
    <div class="launcher-header">
        <div class="title">
            <h1 class="header dropshadow">{launcherName}</h1>
            <span>v{launcherVersion}</span>
        </div>
    
        <div class="image-content">
            <canvas id="head-canvas"/>
            <h3 class="header dropshadow">{username}</h3>
        </div>
    
        <div class="image-content-small">
            <img src="svg/SwitchUser.svg" alt="Switch User">
            <h3 class="header dropshadow">Switch User</h3>
        </div>
    
        <div class="image-content-small">
            <img src="svg/Settings.svg" alt="Switch User">
            <h3 class="header dropshadow">Launcher Settings</h3>
        </div>
    </div>
    <div class="menu-buttons">
        <MenuNavbar buttons={buttons} bind:selected={selectedTab}></MenuNavbar>
    </div>
</nav>

<style>
    nav {
        width: 100%;
        height: 100vh;
        background: #333;
    }

    .launcher-header {
        padding-top: 8px;
        padding-bottom: 4px;
        background-color: #4E4E4E;
    }

    .image-content {
        width: 100%;
        overflow: hidden;
    }

    .image-content > canvas {
        float: left;
        width: 2.6vw;
        height: 2.6vw;
        border-radius: 8px;
        margin-left: 24px;
        margin-right: 8px;
    }

    .image-content h3 {
        color: white;
        font-size: 1vw;
    }

    .image-content-small {
        width: 100%;
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
        width: 100%;
        overflow: hidden;
        text-align: center;
        margin-bottom: 16px
    }

    .title * {
        display: inline;
        vertical-align: bottom;
        margin: 0px;
    }

    .title > h1 {
        font-size: 2vw;
        margin-right: 4px;
        color: white;
    }

    .title span {
        font-size: 0.75vw;
        color: rgb(26, 26, 26);
    }

    .dropshadow {
        color: white;
    }
</style>