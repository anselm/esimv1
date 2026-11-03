// attest as the identity service (https://attest.monster; the code is ~/projects/2027/attest). A person signs in on attest with
// their passkey; the passkey signs a 30-day delegation to a key the browser made for this origin. The socket handshake carries
// that delegation and a fresh signature by the delegated key. We check the shape, the origin, the expiry, the signature, and then
// ask attest that it accepted this delegation and has not seen it revoked. Nothing about the person is stored here but their DID
// and handle, in the area.toml of an area they claim. Adapted from matchmaker/lib/attest.mjs on 2026-11-03.
import { idOf, verifyObject, didFromJwk, isAccountDid, isDid } from "./attest-verify.mjs";
export const ATTEST_URL = (process.env.ATTEST_URL || "https://attest.monster").replace(/\/+$/, "");
const TEN_MIN = 10 * 60e3, cache = new Map(); // delegation id -> { at, handle } once attest confirmed it
// v2 delegations (attest, 2026-10-02) name their site only as originHash = idOf({origin, salt}), so attest's public log does not
// say where anyone signed in. v1 named the origin in clear; still accepted until those sessions expire.
export async function checkShape(d, origin, salt) {
  if (!d || (d.v !== 1 && d.v !== 2) || d.type !== "delegation") throw new Error("not an attest delegation");
  if (!isAccountDid(d.root) || !isDid(d.device)) throw new Error("root must be an account did, device a did:key");
  if (!d.devKey || typeof d.devKey.x !== "string" || typeof d.devKey.y !== "string") throw new Error("devKey {x,y} required");
  if (didFromJwk(d.devKey) !== d.device) throw new Error("device did does not match devKey");
  if (d.v === 1 && d.origin !== origin) throw new Error(`delegation was issued for ${d.origin}, not ${origin || "this origin"}`);
  if (d.v === 2 && !/^[0-9a-f]{64}$/.test(d.originHash || "")) throw new Error("bad originHash");
  if (d.v === 2 && salt && (await idOf({ origin, salt })) !== d.originHash) throw new Error("delegation was issued for another site");
  const from = Date.parse(d.from), until = Date.parse(d.until); if (!(from > 0) || !(until > from)) throw new Error("bad from/until");
  if (Date.now() > until) throw new Error("delegation expired; sign in again");
  const fields = d.v === 1 ? "devKey,device,from,origin,root,type,until,v" : "devKey,device,from,originHash,root,type,until,v";
  if (Object.keys(d).sort().join(",") !== fields) throw new Error("unexpected delegation fields");
}
// Ask attest that this delegation exists, stands, and was issued for this origin: GET /delegation/:id?origin= answers "match" or
// "mismatch" without naming the site. Older attest servers lack it; then fall back to the public /by listing (v1 only). Cached ten
// minutes per id.
export async function confirmed(id, root, origin) {
  const c = cache.get(id); if (c && Date.now() - c.at < TEN_MIN) return c;
  try {
    const r = await fetch(`${ATTEST_URL}/delegation/${id}?origin=${encodeURIComponent(origin || "")}`, { signal: AbortSignal.timeout(8000) });
    const j = (r.headers.get("content-type") || "").includes("json") ? await r.json() : null;
    if (j?.id === id) {
      if (j.root !== root) throw new Error("attest has this delegation under another account");
      if (j.revoked) throw new Error("this delegation was revoked on attest; sign in again");
      if (j.origin !== "match") throw new Error("attest says this sign-in was issued for another site");
      const entry = { at: Date.now(), handle: j.handle || "" }; cache.set(id, entry); return entry;
    }
    if (j?.error === "unknown delegation") throw new Error("attest does not know this delegation; sign in again");
  } catch (e) { if (/attest (does not|has|says)|revoked/.test(e.message)) throw e; if (c) return c; }
  let j; try { const r = await fetch(`${ATTEST_URL}/by/${encodeURIComponent(root)}`, { signal: AbortSignal.timeout(8000) }); if (!r.ok) throw new Error("attest answered " + r.status); j = await r.json(); }
  catch (e) { if (c) return c; throw new Error("cannot reach attest to confirm your sign-in (" + e.message + ")"); }
  const d = (j.delegations || []).find((x) => x.id === id); if (!d) throw new Error("attest does not know this delegation; sign in again");
  if (d.revoked) throw new Error("this delegation was revoked on attest; sign in again");
  const entry = { at: Date.now(), handle: j.handle || "" }; cache.set(id, entry); return entry;
}
// The handshake: { del, delegation, at, sig } where sig is the device key over {type:"esim-auth", del, origin, at}.
export async function verifyHandshake(a, origin) {
  if (!a || typeof a !== "object" || !a.delegation) return null;
  await checkShape(a.delegation, origin, typeof a.salt === "string" ? a.salt : null); const id = await idOf(a.delegation); if (id !== a.del) throw new Error("delegation id does not match");
  const at = Date.parse(a.at); if (!(at > 0) || Math.abs(Date.now() - at) > TEN_MIN) throw new Error("handshake time is not near now");
  if (!(await verifyObject(a.delegation.devKey, { type: "esim-auth", del: id, origin, at: a.at }, a.sig))) throw new Error("handshake signature does not verify");
  const { handle } = await confirmed(id, a.delegation.root, origin);
  return { del: id, root: a.delegation.root, device: a.delegation.device, devKey: a.delegation.devKey, handle, until: a.delegation.until };
}
// A signed post: sig = { v:3, kind, del, at, hash, sig } with sig.sig the device key over {kind, hash, at, del}. The delegation must
// be the one this socket signed in with, so the key is already known to belong to the person.
export async function verifyContent(ctx, kind, hash, sig) {
  if (!ctx) return { ok: false, reason: "not signed in through attest" }; if (sig.del !== ctx.del) return { ok: false, reason: "signed under another delegation" };
  const at = Date.parse(sig.at); if (!(at > 0) || Math.abs(Date.now() - at) > 15 * 60e3) return { ok: false, reason: "statement time is not recent" };
  const ok = await verifyObject(ctx.devKey, { kind, hash, at: sig.at, del: sig.del }, sig.sig); return ok ? { ok } : { ok, reason: "signature does not verify" };
}
export async function resolveAccount(handleOrDid) { const path = isAccountDid(handleOrDid) ? "/by/" + encodeURIComponent(handleOrDid) : "/handle/" + encodeURIComponent(handleOrDid); const r = await fetch(ATTEST_URL + path, { signal: AbortSignal.timeout(8000) }); if (!r.ok) throw new Error("attest answered " + r.status + " for " + handleOrDid); const j = await r.json(); if (!j.did) throw new Error("no such account on attest"); return { did: j.did, handle: j.handle || "" }; }
