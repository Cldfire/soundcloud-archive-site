<script>
    import { Router, Route } from 'yrv';
    import { onMount } from 'svelte';

    import Index from './Index.svelte';
    import Register from './Register.svelte';
    import Login from './Login.svelte';
    import { signedIn, userId } from './stores.js';

    onMount(async () => {
        const response = await fetch(
            "/api/me",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            const userInfo = await response.json();

            signedIn.set(true);
            userId.set(userInfo.user_id);
        }
    });
</script>

<Router>
    <Route exact path="/" component={Index}/>
    <Route path="/register" component={Register}/>
    <Route path="/login" component={Login}/>
</Router>
