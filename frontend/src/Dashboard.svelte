<script>
    import {Tabs, Tab, TabList, TabPanel } from 'svelte-tabs';
    import { Link } from 'yrv';
    import { get } from 'svelte/store';
    import { onDestroy } from 'svelte';

    import TracksList from './TracksList.svelte'
    import PlaylistsList from './PlaylistsList.svelte'
    import Login from './Login.svelte'
    import About from './About.svelte'
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
            loading();
        }

        getstat(){
            if(this.numPlaylistsToDownload < 0 || this.numTracksToDownload < 0)
                return 0;
            if(this.numTracksToDownload + this.numPlaylistsToDownload === 0)
                return 0;
            return ((this.numPlaylistsDownloaded + this.numTracksDownloaded) /
                    (this.numTracksToDownload + this.numPlaylistsToDownload));
        }
    }

    let totalDownloads = 0;
    let totalDownloaded = 0;
    let progress;


    var likedTracks = [];
    var likedAndOwnedPlaylists = [];
    var ss = new ScrapingState();

    function checknulltext(obj, output){
        if(obj != null){
            obj.text = output;
        }
    }
    function checknullvalue(obj, output) {
        if(obj != null){
            obj.value = output;
        }
    }

    function sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    async function loading(){
        if(ss === undefined){
            checknulltext(document.getElementById("loading"), "0%");
            checknullvalue(document.getElementById("loading_bar"), 0);
        } else{
            checknulltext(document.getElementById("loading"), ss.getstat().toString() + "%");
            checknullvalue(document.getElementById("loading_bar"), ss.getstat());
        }
        await sleep(1000);
        loading();
    }

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
            "/api/do-scraping?num_recent_likes=500&num_recent_playlists=5",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
        } else {
            alert(await response.text());
        }
    }

    function loadingbar(){
        document.getElementById("loading").style.width = "50%";
    }


    async function clearLikedTracks() {
        const response = await fetch(
            "/api/clear-liked-tracks",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            getLikedTracks();
        } else {
            alert(await response.text());
        }
    }

    async function clearPlaylists() {
        const response = await fetch(
            "/api/clear-playlists",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            getLikedAndOwnedPlaylists();
        } else {
            document.getElementById("loading").value = ss.getstat();
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
            document.getElementById("loading").value = ss.getstat();
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
                } else if (data.PlaylistsScrapingEvent) {
                    let d = data.PlaylistsScrapingEvent;

                    if (d.NumPlaylistInfoToDownload) {
                        ss.numPlaylistsToDownload = d
                            .NumPlaylistInfoToDownload
                            .num;
                    } else if (d.FinishPlaylistInfoDownload) {
                        ss.numPlaylistsDownloaded += 1;
                    }
                } else if (data == "Complete") {
                    getLikedTracks();
                    getLikedAndOwnedPlaylists();
                }

                if (
                    !ss.finishedDownloadingTracks &&
                    ss.numTracksToDownload == ss.numTracksDownloaded
                ) {
                    ss.finishedDownloadingTracks = true;
                }

                if (
                    !ss.finishedDownloadingPlaylists &&
                    ss.numPlaylistsToDownload == ss.numPlaylistsDownloaded
                ) {
                    ss.finishedDownloadingPlaylists = true;
                }
                document.getElementById("loading").value = ss.getstat();
            });
        }
    });


    let unsubSignedIn = signedIn.subscribe((val) => {
        if (val === true) {
            getLikedTracks();
            getLikedAndOwnedPlaylists();
        }
    });

    onDestroy(unsubEvtSource);
    onDestroy(unsubSignedIn);


</script>

<style>
    :global(.svelte-tabs),:global(.svelte-tabs li.svelte-tabs__tab){
        color: white;
    }
    :global(.svelte-tabs li.svelte-tabs__selected),:global(.svelte-tabs div.svelte-tabs__tab-panel){
        color: gray;
    }

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
    .Loading_Transparent{
        opacity: 40%;
        height: 10px;
        background: black;
        width: 100%;
        margin-right: auto;
        margin-left: auto;
    }
    .Loading_bar{
        opacity: 20%;
        height: 10px;
        background: white;
        width: 0.1%;
        margin-right: auto;
        margin-left: auto;
    }
</style>

<div class="background">
    <div class="Title-div">
        <h class="Title">Sound Cloud Archive Site</h>
    </div>
{#if $signedIn}
    <div style="float: right;">
    <button on:click="{logOut}">Log Out</button>
    </div>
    <br>

    <Link href="set-soundcloud-credentials">Set SoundCloud Credentials</Link>
    <br>
    <button on:click="{startScraping}">Scrape SoundCloud</button>
    <small class="mb-3" id="loading">0%</small>
    <progress id="loading_bar" value="0"></progress>
    <Tabs>
        <TabList>
            <Tab class="Tab">Tracks</Tab>
            <Tab class="Tab">Playlists</Tab>
        </TabList>

        <TabPanel>
            <TracksList tracks={likedTracks}/>
        </TabPanel>
        <TabPanel>
            <PlaylistsList playlists={likedAndOwnedPlaylists}/>
        </TabPanel>
    </Tabs>
    <button on:click="{clearLikedTracks}">Delete Liked Tracks</button>
    <button on:click="{clearPlaylists}">Delete Playlists</button>

{:else}
    <p>You are not signed in.</p>

    <Login/>
{/if}
    <About/>
</div>
