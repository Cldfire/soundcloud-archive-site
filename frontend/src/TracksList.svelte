<script>
    import VirtualList from '@sveltejs/svelte-virtual-list';

    // Array of track information to display
    export let tracks;
    let searchTerm;

    $: filterList = tracks.filter(
        item => item.title.indexOf(searchTerm) !== -1 ||
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
        ret += s + "";
        return ret;
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
        width: 33%;
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
{#if tracks.length > 0}
    <h2>Liked Tracks:</h2>
{/if}
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
            <th>{item.title}</th>
            <th>{timeMstoReg(item.length_ms)}</th>
            <th>{item.playback_count}</th>
            <th>{item.username}</th>
            <th>{item.created_at}</th>
        </tr>
    </table>
</VirtualList>
