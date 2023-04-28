<script lang="ts">
    import { Route, Router} from "svelte-navigator";

    import Loader from "./components/Loader.svelte";
    import Login from "./account/Login.svelte";
    import Home from "./Home.svelte";


    async function init() {
        await new Promise((resolve) => setTimeout(resolve, 200));
    }

</script>

<Router>
    <Route path="/test" component={Loader} />
    {#await init()}
        <Loader />
    {:then}
        <Route path="/" component={Home} />
        <Route path="/login" component={Login} />
    {/await}
</Router>
