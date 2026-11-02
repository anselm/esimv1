<script>
  // The corner badge on every page: name, version, and the commit the page was built from.
  const version = typeof __ESIM_VERSION__ === 'string' ? __ESIM_VERSION__ : ''
  const commit = typeof __ESIM_COMMIT__ === 'string' ? __ESIM_COMMIT__ : ''
</script>

<span class="muted" title="esim {version}{commit ? ` · ${commit}` : ''}">esim {version}{#if commit}<span class="opacity-60"> · {commit}</span>{/if}</span>
