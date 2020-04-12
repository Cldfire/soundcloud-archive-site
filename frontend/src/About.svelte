<script>
    import Ian from './Ian.svelte'
    import Jarek from './Jarek.svelte'

    var select;
    function select_ian(){
        select = 1;
    }
    function select_jarek(){
        select = 2;
    }
</script>
<div>
    <button on:click={select_ian}>About Ian</button>
    <button on:click={select_jarek}>About Jarek</button>

    {#if select === 1}
        <Ian/>
    {/if}
    {#if select === 2}
        <Jarek/>
    {/if}
</div>