<script>
  // Who you are, in the corner: sign in through attest, or your handle and a way out. Shown
  // only when the server names an attest; a local copy without one shows nothing.
  import { hello, reconnect } from './socket.js'
  import { setAttestOrigin, openPopup, signIn, signOut, returning } from './attest.js'

  let attest = $state('')
  let user = $state(null)
  let authError = $state('')
  let busy = $state(false)

  async function refresh(h) {
    h = h ?? (await hello())
    attest = h.attest ?? ''
    user = h.user ?? null
    authError = h.authError ?? ''
    if (attest) setAttestOrigin(attest)
  }
  $effect(() => {
    refresh().then(async () => {
      // back from attest's redirect flow: the session is in the fragment, taken by the library at load
      if (returning() && attest) { await new Promise((r) => setTimeout(r, 200)); await refresh(await reconnect()) }
    })
  })

  async function doSignIn() {
    // the popup must open synchronously inside the tap, before any await (Safari)
    const popup = openPopup()
    busy = true
    try {
      await signIn(popup)
      await refresh(await reconnect())
    } catch (e) { authError = e.message } finally { busy = false }
  }
  async function doSignOut() {
    await signOut()
    await refresh(await reconnect())
  }
</script>

{#if attest}
  {#if user}
    <span class="muted" title={authError || `signed in through ${attest} as ${user.did}`}>@{user.handle || user.did.slice(8, 16)}</span>
    <button class="hover:text-brand" onclick={doSignOut} title="sign out">out</button>
  {:else}
    <button class="hover:text-brand disabled:opacity-50" onclick={doSignIn} disabled={busy} title={authError || `sign in through ${attest}`}>{busy ? '…' : 'sign in'}</button>
  {/if}
{/if}
