<script lang="ts">
    import { Route, Router } from "svelte-navigator";

    import TextLoader from "./components/loader/TextLoader.svelte";
    import CircleLoader from "./components/loader/CircleLoader.svelte";
    import Login from "./account/Login.svelte";
    import Home from "./Home.svelte";
    import SwitchAccounts from "./account/SwitchAccounts.svelte";
    import NewInstanceVersion from "./new-instance/NewInstanceVersion.svelte";
    import NewInstanceSettings from "./new-instance/NewInstanceSettings.svelte";

    async function init() {
        await new Promise((resolve) => setTimeout(resolve, 200));
    }
</script>

<Router>
    <Route path="/test" component={CircleLoader} />
    {#await init()}
        <TextLoader />
    {:then}
        <Route path="/">
            <Home />
        </Route>
        <Route path="/login">
            <Login />
        </Route>
        <Route path="/switchaccounts">
            <SwitchAccounts />
        </Route>
        <Route path="/newinstance-version" primary={false}>
            <NewInstanceVersion />
        </Route>
        <Route path="/newinstance-settings" primary={false}>
            <NewInstanceSettings />
        </Route>
    {/await}
</Router>
