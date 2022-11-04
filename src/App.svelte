<script lang="ts">
    import { listen } from '@tauri-apps/api/event'
    import { onMount } from 'svelte';
    import { link, Router, Route, Link, navigate } from "svelte-navigator";
    import Login from './components/Login.svelte';
    import Home from "./components/Home.svelte";
    import NewInstance from "./components/NewInstance.svelte";
    import { invoke } from '@tauri-apps/api/tauri';

    // const unlisten =  listen("auth_result", (event) => {
    //     console.log(event);
    //     console.log("Here");
    //     navigate("/");
    // })
    
    // window.onload = function() {
    //     // TODO: Check the state's account manager, if theres an active account redirect to `/home` else, stay at '/'.
    //     console.log("On Load");
    // };

</script>

<main>
    <Router>
        <Route path="/login" component={Login}/>
        <Route path="/" component={Home}/>
        <Route path="/new-instance" component={NewInstance}/>
    </Router>
</main>

<style>
    main {
        width: 100%;
        height: 100%;
    }
</style>