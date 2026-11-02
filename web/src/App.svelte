<script>
  import { route } from './lib/router.svelte.js'
  import ProjectList from './lib/ProjectList.svelte'
  import ProjectView from './lib/ProjectView.svelte'
  import NewProject from './lib/NewProject.svelte'
  import RunList from './lib/RunList.svelte'
  import RunView from './lib/RunView.svelte'
  import DocsList from './lib/DocsList.svelte'
  import DocView from './lib/DocView.svelte'
  import Kinds from './lib/Kinds.svelte'
  import Compare from './lib/Compare.svelte'
</script>

<main class="mx-auto max-w-6xl px-4 pb-16 pt-14 sm:px-6">
  {#if route.project && route.run}
    {#key route.project + route.run}
      <RunView id={route.run} base={`/projects/${route.project}/runs/${encodeURIComponent(route.run)}`} back={{ project: route.project }} backLabel={route.project} />
    {/key}
  {:else if route.run}
    {#key route.run}
      <RunView id={route.run} base={`/runs/${encodeURIComponent(route.run)}`} back={{ runs: true }} backLabel="runs" />
    {/key}
  {:else if route.project}
    {#key route.project}
      <ProjectView path={route.project} />
    {/key}
  {:else if route.isNew}
    <NewProject />
  {:else if route.doc}
    {#key route.doc}
      <DocView path={route.doc} />
    {/key}
  {:else if route.docs}
    <DocsList />
  {:else if route.kinds}
    <Kinds />
  {:else if route.compare}
    {#key route.compare}
      <Compare path={route.compare} />
    {/key}
  {:else if route.runs}
    <RunList />
  {:else}
    <ProjectList />
  {/if}
</main>
