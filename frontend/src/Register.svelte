<script>
    import { navigateTo } from 'yrv';

    import { getSseToken, updateStoresAfterLogin } from './util.js';

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
            await updateStoresAfterLogin(await response.json(), await getSseToken());
            navigateTo('/');
        } else {
            alert(await response.text());
        }
    }
</script>
<style>
    .background{
        background-image: url("https://miro.medium.com/max/3200/1*NKoVsTnFExkyQBvnKK94Yg.jpeg");
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
    </div>
    <div class="Title-div">
        <form on:submit|preventDefault="{handleSubmit}">
            <input required id="username" placeholder="Username"/>

            <input required type="password" id="password" placeholder="Password"/>

            <button type="submit">Create account</button>
        </form>
    </div>
</div>
