<script lang="ts">
    import { Emphasis, classFromEmphasis } from "../../menu";

    export let id: string;
    export let value: string;
    export let label: string;
    export let disabled: boolean = false;
    export let emphasis: Emphasis = Emphasis.High;
</script>

<div class="text-box-wrapper">
    <input {disabled} {id} type="text" bind:value required />
    <label class={classFromEmphasis(emphasis)} for={id}>{label}</label>
</div>

<style>
    .text-box-wrapper {
        width: var(--width, 450px);
        white-space: nowrap;
    }

    .text-box-wrapper input {
        width: var(--width, 450px);
        height: auto;
        padding: 10px 4px;
        font-size: 1.6rem;
        color: white;
        background: transparent;
        outline: none;
        border: 2px solid transparent;
        opacity: var(--high-emphasis-opacity);
        border-bottom: 2px solid var(--medium-purple);
        box-sizing: border-box;
    }

    .text-box-wrapper label {
        position: relative;
        margin-left: 4px;
        right: var(--width, 450px);
        top: 0;
        padding: 10px 0;
        font-size: 1.6rem;
        color: white;
        transition: top 200ms cubic-bezier(0, 0, 0.2, 1) 0ms,
            color 200ms cubic-bezier(0, 0, 0.2, 1) 0ms;
        pointer-events: none;
    }

    .text-box-wrapper input:focus {
        border: 2px solid var(--light-blue);
    }

    .text-box-wrapper input:focus ~ label {
        font-size: 1.2rem;
        top: -23px;
        padding: 0 4px 0 4px;
        opacity: 100%;
        color: var(--light-blue);
        background-color: var(--dark-black);
    }

    .text-box-wrapper input:focus {
        border: 2px solid var(--light-blue);
    }

    .text-box-wrapper input:focus ~ label,
    .text-box-wrapper input:valid ~ label {
        font-size: 1.2rem;
        top: -23px;
        padding: 0 4px 0 4px;
        opacity: 100%;
        background-color: var(--medium-black);
    }

    .text-box-wrapper input:valid:not(:focus) ~ label {
        color: var(--light-purple);
    }

    .text-box-wrapper input:valid:not(:focus) {
        border: 2px solid var(--medium-purple);
    }

    input:disabled {
        cursor: not-allowed;
    }
</style>
