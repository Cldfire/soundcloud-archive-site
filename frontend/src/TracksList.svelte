<script>
    import VirtualList from '@sveltejs/svelte-virtual-list';

    // Array of track information to display
    export let tracks;
    let searchTerm;
    let trackDetails;

    let song = "";
    let artist = "";
    let trackShow = false;
    let descript = "";
    let likes = 0;
    let art = ""
    let play = ""
    let avatar = "";
    let full_name = "";
    let profile = "";

    $: filterList = tracks.filter(
        item => toUpper(item.title).indexOf(searchTerm) !== -1 ||
        toUpper(item.username).indexOf(searchTerm) !== -1 ||
        searchTerm === undefined
    );

    function toUpper(obj){
        return obj.toLowerCase();
    }

    function timeMstoReg(input) {
        let ret = "";
        let h = Math.floor(input / (1000 * 60 * 60));
        let m = Math.floor((input / (1000 * 60)) % 60);
        let s = Math.floor((input / 1000) %60);
        if( h !== 0)
            ret += h + "h";
        ret += m + ":";
        if(s < 10) {
            ret += "0";
        }
        ret += s;
        return ret;
    }

    async function getTrackInfo(trackNo) {
        const response = await fetch(
                "/api/track-info/" + trackNo,
                {
                    method: 'GET',
                    credentials: 'same-origin'
                }
        );
        if (response.ok) {
            trackDetails = await response.json();
            openTrack();
        } else {
            alert(await response.text());
        }
    }

    function openTrack(){
        song = trackDetails.brief_info.title;
        artist = trackDetails.brief_info.username;
        descript = trackDetails.description;
        likes = trackDetails.likes_count;
        art = trackDetails.artwork_url;
        play = trackDetails.track_permalink_url;
        avatar = trackDetails.avatar_url;
        full_name = trackDetails.full_name;
        profile = trackDetails.user_permalink_url;
        trackShow = true;
    }

    function finish(){
        trackShow = false;
    }
</script>

<style>
    table{
        table-layout: fixed;
        width: 100%;
    }
    th, td {
        border: 1px solid white;
        color: white;
        width: 20%;
    }
    td {
    }
    h2 {
        color: white;
    }
    textarea{
        width: 100%;
        height: 40px;
    }
</style>

<div width="100%" height="auto">
    <textarea id="SearchBar" placeholder="Search Bar: use only lowercase"  bind:value={searchTerm}></textarea>
</div>
{#if trackShow}
<h2>Details on {song} by {artist}</h2>
<table>
    <tr>
        <th>Likes</th>
        <th>Artwork</th>
        <th>Link to Track</th>
        <th>Avatar</th>
        <th>Creator's Name</th>
    </tr>
    <tr>
        <th>{likes}</th>
        <th><img src="{art}" alt="Not Found"></th>
        <th><a href="{play}">Link to Music</a></th>
        <th><img src="{avatar}" alt="Not Found"></th>
        <th><a href="{profile}">{full_name}</a></th>
    </tr>
</table>
<p>Description: {descript}</p>
<button on:click={finish}>Exit Info</button>
{:else}
<h2>Liked Tracks:</h2>
<table width="100%">
    <tr>
        <th>Title</th>
        <th>Length</th>
        <th>Playback Count</th>
        <th>Username</th>
        <th>Date Created</th>
    </tr>
</table>
<VirtualList height="400px" items={filterList} let:item>
    <table>
        <tr>
            <th><button on:click="{getTrackInfo(item.track_id)}">{item.title}</button></th>
            <th>{timeMstoReg(item.length_ms)}</th>
            <th>{item.playback_count}</th>
            <th>{item.username}</th>
            <th>{item.created_at}</th>
        </tr>
    </table>
</VirtualList>
{/if}
