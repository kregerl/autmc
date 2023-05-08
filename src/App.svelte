<script lang="ts">
    import { Route, Router } from "svelte-navigator";

    import Loader from "./components/Loader.svelte";
    import Login from "./account/Login.svelte";
    import Home from "./Home.svelte";
    import SwitchAccounts from "./account/SwitchAccounts.svelte";
    import NewInstanceVersion from "./components/newinstance/NewInstanceVersion.svelte";
    import NewInstanceSettings from "./components/newinstance/NewInstanceSettings.svelte";

    async function init() {
        await new Promise((resolve) => setTimeout(resolve, 200));
    }
</script>

<Router>
    <Route path="/test" component={Loader} />
    {#await init()}
        <Loader />
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
        <Route path="/newinstance-version">
            <NewInstanceVersion />
        </Route>
        <Route path="/newinstance-settings">
            <NewInstanceSettings />
        </Route>
    {/await}
</Router>
