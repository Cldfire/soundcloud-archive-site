<script>
    import { Link } from 'yrv';

    import { signedIn, userId } from './stores.js';

    async function logOut() {
        const response = await fetch(
            "/api/logout",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );
        if (response.ok) {
            signedIn.set(false);
            userId.set(-1);
        } else {
            alert(await response.text());
        }
    }
</script>

{#if $signedIn}
    <p>Hi! You are signed in.</p>

    <button on:click="{logOut}">Log Out</button>
{:else}
    <p>You are not signed in.</p>

    <Link href="login">Log In</Link>
    <Link href="register">Register</Link>
{/if}
