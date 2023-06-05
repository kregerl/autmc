<div id="background"/>
<div class="login-container">
    <slot />
</div>

<style>
    #background {
        height: 100%;
        background: url("https://www.minecraft.net/content/dam/minecraft/backgrounds/msalogin-bg-glob.jpg")
            50% 50% / cover no-repeat;
    }

    .login-container {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        min-width: 25vw;
        background-color: var(--light-black);
        border: 2px solid var(--medium-black);
        border-radius: 6px;
        box-shadow: 5px 5px 16px 2px rgba(0, 0, 0, 0.75);
    }
</style>
