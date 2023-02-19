<script lang="ts">
    import { Router, Route, navigate } from "svelte-navigator";
    import Login from "./components/Login.svelte";
    import Home from "./components/Home/Home.svelte";
    import NewInstance from "./components/Modal/NewInstanceModal/NewInstance.svelte";
    import RightClickMenu from "./components/RightClickMenu.svelte";
    import { listen, UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";

    // let unlistener: UnlistenFn;
    let authErrorUnlistener: UnlistenFn;
    onMount(async () => {
        authErrorUnlistener = await listen("authentication-error", event => {
            console.log("Here", event);
        });

        // unlistener = await listen("auth_result", (event) => {
        //     console.log(event);
        //     console.log("Here");
        //     navigate("/");
        // });
    });

    onDestroy(() => {
        authErrorUnlistener();
        // unlistener();
    });
</script>

<Router>
    <Route path="/" component={Home} />
    <Route path="/login" component={Login} />
    <Route path="/new-instance" component={NewInstance} />
    <!-- TODO: Only for testing, remove this -->
    <Route path="/test" component={RightClickMenu} />
</Router>

<style>
</style>
