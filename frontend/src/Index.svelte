<script>
    import { Link } from 'yrv';
    import { get } from 'svelte/store';

    import TracksList from './TracksList.svelte'
    import PlaylistsList from './PlaylistsList.svelte'
    import { signedIn, userId } from './stores.js';

    var likedTracks = [];
    var likedAndOwnedPlaylists = [];

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

    async function startScraping() {
        const response = await fetch(
            "/api/do-scraping?num_recent_likes=10&num_recent_playlists=2",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            alert("Started scraping process");
        } else {
            alert(await response.text());
        }
    }

    async function getLikedTracks() {
        const response = await fetch(
            "/api/liked-tracks",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            likedTracks = await response.json();
        } else {
            alert(await response.text());
        }
    }

    async function getLikedAndOwnedPlaylists() {
        const response = await fetch(
            "/api/liked-and-owned-playlists",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            likedAndOwnedPlaylists = await response.json();
        } else {
            alert(await response.text());
        }
    }
</script>

{#if $signedIn}
    <p>Hi! You are signed in.</p>
    <button on:click="{logOut}">Log Out</button>
    <br>

    <Link href="set-soundcloud-credentials">Set SoundCloud Credentials</Link>
    <br>

    <button on:click="{startScraping}">Scrape SoundCloud</button>
    <br>

    {#await getLikedTracks() then _}
        <TracksList tracks={likedTracks}/>
    {/await}

    {#await getLikedAndOwnedPlaylists() then _}
        <PlaylistsList playlists={likedAndOwnedPlaylists}/>
    {/await}
{:else}
    <p>You are not signed in.</p>

    <Link href="login">Log In</Link>
    <Link href="register">Register</Link>
{/if}
