<script lang="ts">
    import { Router, Route, navigate } from "svelte-navigator";
    import Login from "./components/Login.svelte";
    import Home from "./components/Home/Home.svelte";
    import NewInstance from "./components/Modal/NewInstanceModal/NewInstance.svelte";
    import Loading from "./components/Loader/Loading.svelte";
    import { listen, UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import { logStore } from "./store/logstore";
    import { screenshotStore } from "./store/screenshotstore";
    import { invoke } from "@tauri-apps/api/tauri";
    import { manifestStore } from "./store/manifeststore";

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
            let currentMap = $logStore.get(payload.instance_name) ?? new Map();
            if (
                $logStore.has(payload.instance_name) &&
                $logStore.get(payload.instance_name).has("running")
            ) {
                let currentLines = currentMap.get("running");
                $logStore = $logStore.set(
                    payload.instance_name,
                    currentMap.set("running", [...currentLines, payload.line])
                );
            } else {
                $logStore = $logStore.set(
                    payload.instance_name,
                    currentMap.set("running", [payload.line])
                );
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
        $logStore = new Map([...logs, ...$logStore]);
        console.log("$writableMap", $logStore);

        // TODO: Use notify crate to emit event when screenshot is added to ss's dir.
        $screenshotStore = await invoke("get_screenshots");
        screenshotStore.sort();
        console.log("$screenshotStore", $screenshotStore);

        $manifestStore = await invoke("obtain_manifests");
        console.log("$manifestStore", $manifestStore);
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
    <Route path="/" component={Home} />
    <Route path="/login" component={Login} />
    <Route path="/new-instance" component={NewInstance} />
    <Route path="/test" component={Loading} />
</Router>

<style>
</style>
