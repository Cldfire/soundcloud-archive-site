<script>
    import VirtualList from '@sveltejs/svelte-virtual-list';

    // Array of playlist information to display
    export let playlists;
    let searchTerm;
    let playShow = false;
    let playDetails;
    let title = "";
    let user = "";
    let playlist_URL = "";
    let descript = "";
    let likes = 0;
    let avatar = "";
    let full_name = "";
    let user_URL = "";
    let tracksInfo = [];

    $: filterList = playlists.filter(
        item =>
                toUpper(item.title).indexOf(searchTerm) !== -1 ||
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

    function isAlbum(item){
        if(item.is_album)
            return "Yes";
        return "No";
    }

    async function getTrackInfo(playNo) {
        const response = await fetch(
                "/api/playlist-info/" + playNo,
                {
                    method: 'GET',
                    credentials: 'same-origin'
                }
        );
        if (response.ok) {
            playDetails = await response.json();
        } else {
            alert(await response.text());
        }

        let responses = [];
        for(let i = 0; i < playDetails.track_ids.length; ++i){
            const response = await fetch(
                    "/api/track-info/" + playDetails.track_ids[i],
                    {
                        method: 'GET',
                        credentials: 'same-origin'
                    }
            );
            responses.push(response);
        }
        for(let i = 0; i < responses.length; ++i){
            if (responses[i].ok) {
                tracksInfo.push(await responses[i].json());
            } else {
                alert(await response.text());
            }
        }
        openPlay();
    }

    function openPlay(){
        title = playDetails.brief_info.title;
        user = playDetails.brief_info.username;
        playlist_URL = playDetails.playlist_permalink_url;
        descript = playDetails.description;
        likes = playDetails.likes_count;
        avatar = playDetails.avatar_url;
        full_name = playDetails.full_name;
        user_URL = playDetails.user_permalink_url;
        playShow = true;
    }

    function finish(){
        playShow = false;
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
        width: 14%;
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
    p{
        color:white;
    }
</style>
{#if playShow}
<h2>Details on {title} by {user}</h2>
<table>
    <tr>
        <th>Likes</th>
        <th>Link to Playlist</th>
        <th>Avatar</th>
        <th>Creator's Name</th>
    </tr>
    <tr>
        <th>{likes}</th>
        <th><a href="{playlist_URL}">Link to Playlist</a></th>
        <th><img src="{avatar}" alt="Not Found"></th>
        <th><a href="{user_URL}">{full_name}</a></th>
    </tr>
</table>
<p>Description: {descript}</p>
<button on:click={finish}>Exit Info</button>
<table>
    <tr>
        <th>Track Name</th>
        <th>Track Artist</th>
        <th>Length</th>
    </tr>
</table>
<VirtualList height="400px" items={tracksInfo} let:item>
    <table>
        <tr>
            <th>{item.brief_info.title}</th>
            <th>{item.brief_info.username}</th>
            <th>{timeMstoReg(item.brief_info.length_ms)}</th>
        </tr>
    </table>
</VirtualList>
{:else}
<div width="100%" height="auto">
    <textarea id="SearchBar" placeholder="Search Bar: use only lowercase"  bind:value={searchTerm}></textarea>
</div>
<h2>Liked and Owned Playlists:</h2>
<table width="100%">
    <tr>
        <th>Title</th>
        <th>Length</th>
        <th>Username</th>
        <th>Date Created</th>
        <th>Number of Tracks</th>
        <th>Is Album</th>
    </tr>
</table>
<VirtualList height="400px" items={filterList} let:item>
    <table>
        <tr>
            <th><button on:click="{getTrackInfo(item.playlist_id)}">{item.title}</button></th>
            <th>{timeMstoReg(item.length_ms)}</th>
            <th>{item.username}</th>
            <th>{item.created_at}</th>
            <th>{item.num_tracks}</th>
            <th>{isAlbum(item)}</th>
        </tr>
    </table>
</VirtualList>
{/if}