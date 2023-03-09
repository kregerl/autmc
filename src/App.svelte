<script lang="ts">
    import { Router, Route, navigate } from "svelte-navigator";
    import Login from "./components/Login.svelte";
    import Home from "./components/Home/Home.svelte";
    import NewInstance from "./components/Modal/NewInstanceModal/NewInstance.svelte";
    import RightClickMenu from "./components/RightClickMenu.svelte";
    import { listen, UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import { writableMap, LogInformation } from "./logstore";
    import { invoke } from "@tauri-apps/api/tauri";

    interface Payload {
        instance_name: string;
        line: string;
    }

    let unlistener: UnlistenFn;
    let authErrorUnlistener: UnlistenFn;
    onMount(async () => {
        authErrorUnlistener = await listen("authentication-error", (event) => {
            console.log("Here", event);
        });

        unlistener = await listen("instance-logging", (event) => {
            let payload = event.payload as Payload;
            console.log("payload", payload);
            if ($writableMap.has(payload.instance_name)) {
                let current = $writableMap.get(payload.instance_name);
                $writableMap = $writableMap.set(payload.instance_name, [...current, payload.line]);
            } else {
                $writableMap = $writableMap.set(payload.instance_name, [payload.line]);
            }
        });

        let logs: Map<string, Map<string, string[]>> = await invoke("get_logs");
        // $writableMap = new Map([...$writableMap, ...logs]);
        // unlistener = await listen("auth_result", (event) => {
        //     console.log(event);
        //     console.log("Here");
        //     navigate("/");
        // });
    });

    onDestroy(() => {
        authErrorUnlistener();
        unlistener();
    });
</script>

<Router>
    <Route path="/">
        <Home />
    </Route>
    <Route path="/login" component={Login} />
    <Route path="/new-instance" component={NewInstance} />
    <!-- TODO: Only for testing, remove this -->
    <Route path="/test" component={RightClickMenu} />
</Router>

<style>
</style>
