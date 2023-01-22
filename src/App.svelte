<script lang="ts">
    import { Router, Route } from "svelte-navigator";
    import Login from './components/Login.svelte';
    import Home from "./components/Home/Home.svelte";
    import NewInstance from "./components/Modal/NewInstance.svelte";
    import NewInstanceModal from "./components/Modal/NewInstanceModal.svelte";

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

<Router>
    <Route path="/" component={Home}/>
    <Route path="/login" component={Login}/>
    <Route path="/new-instance" component={NewInstance}/>
    <!-- TODO: Only for testing, remove this -->
    <Route path="/test" component={NewInstance}/>
    <Route path="/modal" component={NewInstanceModal}/>
</Router>

<style>
</style>