<script>
    import { Link } from 'yrv';
    import { get } from 'svelte/store';
    import { onDestroy, onMount } from 'svelte';

    import TracksList from './TracksList.svelte'
    import PlaylistsList from './PlaylistsList.svelte'
    import { signedIn, evtSource } from './stores.js';
    import { updateStoresAfterLogout } from './util.js'

    class ScrapingState {
        constructor() {
            this.numTracksToDownload = -1;
            this.numPlaylistsToDownload = -1;
            this.numTracksDownloaded = 0;
            this.numPlaylistsDownloaded = 0;

            this.finishedDownloadingTracks = false;
            this.finishedDownloadingPlaylists = false;
        }
    }

    var likedTracks = [];
    var likedAndOwnedPlaylists = [];
    var ss = new ScrapingState();

    async function logOut() {
        const response = await fetch(
            "/api/logout",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );
        if (response.ok) {
            await updateStoresAfterLogout();
        } else {
            alert(await response.text());
        }
    }

    async function startScraping() {
        ss = new ScrapingState();

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

    let unsubEvtSource = evtSource.subscribe((es) => {
        if (es != null) {
            es.addEventListener('update', (e) => {
                let data = JSON.parse(e.data);

                if (data.LikesScrapingEvent) {
                    let d = data.LikesScrapingEvent;

                    if (d.NumLikesInfoToDownload) {
                        ss.numTracksToDownload = d
                            .NumLikesInfoToDownload
                            .num;
                    } else if (d.MoreLikesInfoDownloaded) {
                        ss.numTracksDownloaded += d
                            .MoreLikesInfoDownloaded
                            .count;
                    }
                } else {
                    let d = data.PlaylistsScrapingEvent;

                    if (d.NumPlaylistInfoToDownload) {
                        ss.numPlaylistsToDownload = d
                            .NumPlaylistInfoToDownload
                            .num;
                    } else if (d.FinishPlaylistInfoDownload) {
                        ss.numPlaylistsDownloaded += 1;
                    }
                }

                if (
                    !ss.finishedDownloadingTracks &&
                    ss.numTracksToDownload == ss.numTracksDownloaded
                ) {
                    getLikedTracks();
                    ss.finishedDownloadingTracks = true;
                }

                if (
                    !ss.finishedDownloadingPlaylists &&
                    ss.numPlaylistsToDownload == ss.numPlaylistsDownloaded
                ) {
                    getLikedAndOwnedPlaylists();
                    ss.finishedDownloadingPlaylists = true;
                }
            });
        }
    });

    let unsubSignedIn = signedIn.subscribe((val) => {
        if (val == true) {
            getLikedTracks();
            getLikedAndOwnedPlaylists();
        }
    });

    onDestroy(unsubEvtSource);
    onDestroy(unsubSignedIn);
</script>

{#if $signedIn}
    <p>Hi! You are signed in.</p>
    <button on:click="{logOut}">Log Out</button>
    <br>

    <Link href="set-soundcloud-credentials">Set SoundCloud Credentials</Link>
    <br>

    <button on:click="{startScraping}">Scrape SoundCloud</button>

    <TracksList tracks={likedTracks}/>
    <PlaylistsList playlists={likedAndOwnedPlaylists}/>
{:else}
    <p>You are not signed in.</p>

    <Link href="login">Log In</Link>
    <Link href="register">Register</Link>
{/if}
