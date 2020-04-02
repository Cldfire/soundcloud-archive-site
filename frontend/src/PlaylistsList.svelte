<script>
    import VirtualList from '@sveltejs/svelte-virtual-list';

    // Array of playlist information to display
    export let playlists;
</script>

{#if playlists.length > 0}
    <h2>Liked and Owned Playlists:</h2>
{/if}

<VirtualList height="300px" items={playlists} let:item>
  <p>{item.title}</p>
</VirtualList>
