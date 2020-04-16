<script>
    import VirtualList from '@sveltejs/svelte-virtual-list';

    // Array of playlist information to display
    export let playlists;
    let searchTerm;

    $: filterList = playlists.filter(
        item =>
                item.title.indexOf(searchTerm) !== -1 ||
                item.username.indexOf(searchTerm) !== -1 ||
                searchTerm === undefined
    );

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
</style>
<div width="100%" height="auto">
    <textarea id="SearchBar" placeholder="Search Bar"  bind:value={searchTerm}></textarea>
</div>
<h2>Liked and Owned Playlists:</h2>
<table width="100%">
    <tr>
        <th>Title</th>
        <th>Length</th>
        <th>Playback Count</th>
        <th>Username</th>
        <th>Date Created</th>
        <th>Number of Tracks</th>
        <th>Is Album</th>
    </tr>
</table>
<VirtualList height="400px" items={filterList} let:item>
    <table>
        <tr>
            <th>{item.title}</th>
            <th>{timeMstoReg(item.length_ms)}</th>
            <th>{item.playback_count}</th>
            <th>{item.username}</th>
            <th>{item.created_at}</th>
            <th>{item.num_tracks}</th>
            <th>{isAlbum(item)}</th>
        </tr>
    </table>
</VirtualList>
