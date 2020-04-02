<script>
    import { navigateTo } from 'yrv';

    import { signedIn, userId } from './stores.js';

    async function handleSubmit(event) {
        if(!event.target.checkValidity()) {
            return;
        }

        const response = await fetch(
            "/api/set-auth-creds",
            {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'same-origin',
                body: JSON.stringify({
                    client_id: event.target.clientId.value,
                    oauth_token: event.target.oauthToken.value
                })
            }
        );

        if (response.ok) {
            alert("Successfully set SoundCloud credentials")
            navigateTo('/')
        } else {
            alert(await response.text());
        }
    }
</script>

<form on:submit|preventDefault="{handleSubmit}">
    <label for="clientId">Client ID</label>
    <input required type="password" id="clientId"/>

    <label for="oauthToken">OAuth Token</label>
    <input required type="password" id="oauthToken"/>

    <button type="submit">Set Credentials</button>
</form>
