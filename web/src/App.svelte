<script>
  import { route } from './lib/router.svelte.js'
  import RunList from './lib/RunList.svelte'
  import RunView from './lib/RunView.svelte'
</script>

<main class="mx-auto max-w-5xl px-4 pb-16 pt-14 sm:px-6">
  {#if route.run}
    {#key route.run}
      <RunView id={route.run} />
    {/key}
  {:else}
    <RunList />
  {/if}
</main>
