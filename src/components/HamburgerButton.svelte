<script lang="ts">
    export let checked: boolean = false;
</script>

<div class="hamburger-menu">
    <div class="hamburder-container">
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
    .hamburder-container {
        z-index: 1;
        position: absolute;
    }

    .hamburger-hover {
        z-index: -1;
        border-radius: 50%;
        width: 54px;
        height: 54px;
        display: none;
    }

    .hamburder-container:hover + .hamburger-hover {
        display: block;
        transform: translate(-3px, -3px);
        background-color: var(--light-black);
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
        z-index: 9;
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
