<script>
    import { navigateTo } from 'yrv';
    import { Link } from 'yrv';

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
<div style="float: right;">
    <form on:submit|preventDefault="{handleSubmit}">
    <table width="100%">
        <tr>
            <th>
                <input required id="username" placeholder="Username"/>
            </th>
            <th>
                <input required type="password" id="password" placeholder="Password"/>
            </th>
            <th>
                <button id="submit" type="submit">Log In</button>
            </th>
        </tr>
    </table>
    </form>
    <form action="register">
        <button>Register</button>
    </form>
</div>