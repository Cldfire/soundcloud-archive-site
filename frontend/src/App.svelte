<script>
    import { Router, Route } from 'yrv';
    import { onMount } from 'svelte';

    import Dashboard from './Dashboard.svelte';
    import Register from './Register.svelte';
    import Login from './Login.svelte';
    import ScCredentials from  './ScCredentials.svelte';
    import { getSseToken, updateStoresAfterLogin } from './util.js'

    onMount(async () => {
        const response = await fetch(
            "/api/me",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            await updateStoresAfterLogin(await response.json(), await getSseToken());
        }
    });
</script>

<Router>
    <Route exact path="/" component={Dashboard}/>
    <Route path="/register" component={Register}/>
    <Route path="/login" component={Login}/>
    <Route path="/set-soundcloud-credentials" component={ScCredentials}/>
</Router>
