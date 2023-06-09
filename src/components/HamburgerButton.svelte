<script lang="ts">
    export let checked: boolean = false;
</script>

<div class="hamburger-menu">
    <div class="hamburger-container">
        <input id="checkbox" type="checkbox" bind:checked />
        <label id="checkbox-label" for="checkbox">
            <div class="line1 high-emphasis" />
            <div class="line2 high-emphasis" />
            <div class="line3 high-emphasis" />
        </label>
    </div>
    <div class="hamburger-hover" />
</div>

<style>
    .hamburger-menu {
        width: 55.5px;
        height: 55.5px;
        margin: var(--margin);
        -webkit-user-select: none;
        user-select: none;
    }

    .hamburger-container {
        z-index: 6;
        position: absolute;
    }

    .hamburger-hover {
        z-index: 5;
        position: absolute;
        width: 55.5px;
        height: 55.5px;
        cursor: pointer;
        background-color: var(--light-black);
        border-radius: 50%;
        transform: translate(calc(-1 * 7.5px / 2), calc(-1 * 7.5px / 2));
        display: none;
    }

    .hamburger-container:hover + .hamburger-hover {
        display: block;
    }

    #checkbox {
        display: none;
    }

    #checkbox-label {
        padding: 7.5px;
        width: 33px;
        height: 33px;
        display: block;
        cursor: pointer;
    }

    #checkbox-label > div {
        width: 33px;
        height: 3px;
        background-color: white;
        position: absolute;
    }

    .line1 {
        margin-top: 6px;
        transition: all 0.3s;
    }

    .line2 {
        margin-top: 15px;
        transition: 0.3s;
    }

    .line3 {
        margin-top: 24px;
        transition: 0.3s;
    }

    #checkbox:checked + #checkbox-label .line1 {
        transform: rotate(-33deg) scaleX(0.55) translate(-28.5px, -4.5px);
        border-radius: 50px 50px 0 50px;
    }

    #checkbox:checked + #checkbox-label .line3 {
        transform: rotate(33deg) scaleX(0.55) translate(-28.5px, 4.5px);
        border-radius: 50px 0 50px 50px;
    }

    #checkbox:checked + #checkbox-label .line2 {
        border-top-left-radius: 50px;
        border-bottom-left-radius: 50px;
        width: 24px;
        transform: translateX(8px);
    }
</style>
