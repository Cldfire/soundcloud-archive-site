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

<style>
    .background{
        background-image: url("https://miro.medium.com/max/3200/1*NKoVsTnFExkyQBvnKK94Yg.jpeg");
        background-size: cover;
        min-height: 100%;
        min-width: 100%;
    }
    .Title{
        font-size: 40px;
        color: white;
    }
    .Title-div{
        text-align: center;
    }
</style>
<div class="background">
    <div class="Title-div">
        <h class="Title">Sound Cloud Archive Site</h>
<form on:submit|preventDefault="{handleSubmit}">
    <input required type="password" id="clientId" placeholder="Client ID"/>

    <input required type="password" id="oauthToken" placeholder="OAuth Token"/>

    <button type="submit">Set Credentials</button>
</form>
    </div>
</div>
