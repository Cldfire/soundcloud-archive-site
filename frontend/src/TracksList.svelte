<script>
    import VirtualList from '@sveltejs/svelte-virtual-list';

    // Array of track information to display
    export let tracks;
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
</style>


{#if tracks.length > 0}
    <h2>Liked Tracks:</h2>
{/if}
<table width="100%">
    <tr>
        <th>Title</th>
        <th>Length(ms)</th>
        <th>Playback Count</th>
        <th>Username</th>
        <th>Date Created</th>
    </tr>
</table>
<VirtualList height="300px" items={tracks} let:item>
    <table>
        <tr>
            <th>{item.title}</th>
            <th>{item.length_ms}</th>
            <th>{item.playback_count}</th>
            <th>{item.username}</th>
            <th>{item.created_at}</th>
        </tr>
    </table>
</VirtualList>
