<script>
    export let artist;

    // sc_user_id: sc_u.sc_user_id,
    // avatar_url: sc_u.avatar_url,
    // full_name: sc_u.full_name,
    // username: sc_u.username,
    // permalink_url: sc_u.permalink_url
    export let playback;
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
    p{
        color: white;
    }
</style>
<table>
    <tr>
        <th>Avatar</th>
        <th>Full Name</th>
        <th>Favorite Artist</th>
        <th>Average Playback Count of all Tracks</th>
    </tr>
    <tr>
        <th><img src="{artist.avatar_url}" alt="Not Found"></th>
        <th>{artist.full_name}</th>
        <th><a href="{artist.permalink_url}">{artist.username}</a></th>
        <th>{playback}</th>
    </tr>
</table>