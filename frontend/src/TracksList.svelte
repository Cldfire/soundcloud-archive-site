<script>
    import VirtualList from '@sveltejs/svelte-virtual-list';

    // Array of track information to display
    export let tracks;
</script>

{#if tracks.length > 0}
    <h2>Liked Tracks:</h2>
{/if}

<VirtualList items={tracks} let:item>
  <p>{item.title}</p>
</VirtualList>
