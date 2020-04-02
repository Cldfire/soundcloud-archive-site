<script>
    import { navigateTo } from 'yrv';

    import { getSseToken, updateStoresAfterLogin } from './util.js';

    async function handleSubmit(event) {
        if(!event.target.checkValidity()) {
            return;
        }

        const response = await fetch(
            "/api/login",
            {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'same-origin',
                body: JSON.stringify({
                    username: event.target.username.value,
                    password: event.target.password.value
                })
            }
        );

        if (response.ok) {
            await updateStoresAfterLogin(await response.json(), await getSseToken());
            navigateTo('/')
        } else {
            alert(await response.text());
        }
    }
</script>

<form on:submit|preventDefault="{handleSubmit}">
    <label for="username">Username</label>
    <input required id="username"/>

    <label for="password">Password</label>
    <input required type="password" id="password"/>

    <button type="submit">Log In</button>
</form>
