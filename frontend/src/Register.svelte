<script>
    import { navigateTo } from 'yrv';

    import { signedIn, userId } from './stores.js';

    async function handleSubmit(event) {
        if(!event.target.checkValidity()) {
            return;
        }

        const response = await fetch(
            "/api/register",
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
            const userInfo = await response.json();

            signedIn.set(true);
            userId.set(userInfo.user_id);
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

    <button type="submit">Create account</button>
</form>
