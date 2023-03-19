<script lang="ts">
    import { Router, Route, navigate } from "svelte-navigator";
    import Login from "./components/Login.svelte";
    import Home from "./components/Home/Home.svelte";
    import NewInstance from "./components/Modal/NewInstanceModal/NewInstance.svelte";
    import Loading from "./components/Loader/Loading.svelte";
    import { listen, UnlistenFn, Event } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import { logStore } from "./store/logstore";
    import { screenshotStore } from "./store/screenshotstore";
    import { invoke } from "@tauri-apps/api/tauri";
    import { manifestStore, VersionManifest } from "./store/manifeststore";
    import { instanceStore } from "./store/instancestore";

    interface Payload {
        instance_name: string;
        line: string;
    }

    let loggingUnlistener: UnlistenFn;
    let instanceUnlistener: UnlistenFn;
    let authErrorUnlistener: UnlistenFn;

    onMount(async () => {
        // unlistener = await listen("auth_result", (event) => {
        //     console.log(event);
        //     console.log("Here");
        //     navigate("/");
        // });
    });

    async function setup() {
        authErrorUnlistener = await listen("authentication-error", (event) => {
            console.log("Here", event);
        });

        // Instances must be loaded first, otherwise the logs and screenshots below will not get populated.
        $instanceStore = await invoke("load_instances");
        console.log("$instanceStore", $instanceStore);
        instanceUnlistener = await listen(
            "new-instance",
            (event: Event<string>) => {
                $instanceStore.push(event.payload);
            }
        );

        loggingUnlistener = await listen("instance-logging", (event) => {
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
            console.log("invoked: ", [key, value]);
            let inner = new Map();
            for (let [k, v] of Object.entries(value)) {
                inner.set(k, v);
            }
            logs.set(key, inner);
        }
        $logStore = new Map([...logs, ...$logStore]);
        console.log("$logStore", $logStore);

        // TODO: Use notify crate to emit event when screenshot is added to ss's dir.
        $screenshotStore = await invoke("get_screenshots");
        screenshotStore.sort();
        console.log("$screenshotStore", $screenshotStore);

        // Manifests are last and the least important
        $manifestStore = await invoke("obtain_manifests");
        $manifestStore.forge_versions = new Map(Object.entries($manifestStore.forge_versions));
        console.log("$manifestStore", $manifestStore);
    }

    onDestroy(() => {
        authErrorUnlistener();
        loggingUnlistener();
        instanceUnlistener();
    });
</script>

<Router>
    {#await setup()}
        <!-- TODO: Add a loading page. -->
        <h3>Loading...</h3>
    {:then}
        <Route path="/" component={Home} />
    {/await}
    <Route path="/login" component={Login} />
</Router>

<style>
</style>
