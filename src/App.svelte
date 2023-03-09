<script lang="ts">
    import { Router, Route, navigate } from "svelte-navigator";
    import Login from "./components/Login.svelte";
    import Home from "./components/Home/Home.svelte";
    import NewInstance from "./components/Modal/NewInstanceModal/NewInstance.svelte";
    import RightClickMenu from "./components/RightClickMenu.svelte";
    import { listen, UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import { writableMap } from "./logstore";
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
            if ($writableMap.has(payload.instance_name) && $writableMap.get(payload.instance_name).has("active")) {
                let current = $writableMap.get(payload.instance_name).get("active");
                $writableMap = $writableMap.set(payload.instance_name, new Map([["active", [...current, payload.line]]]));
            } else {
                $writableMap = $writableMap.set(payload.instance_name, new Map([["active", [payload.line]]]));
            }
        });

        let logs: Map<string, Map<string, string[]>> = new Map();
        for (let [key, value] of Object.entries(await invoke("get_logs"))) {
            let inner = new Map();
            for (let [k, v] of Object.entries(value)) {
                inner.set(k, v);
            }
            logs.set(key, inner);
        }
        $writableMap = new Map([...logs, ...$writableMap]);
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
