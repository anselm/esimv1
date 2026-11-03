// Subset of orbital-attest/verify.mjs (attest repo, packages/orbital-attest), copied by way of matchmaker/lib/attest-verify.mjs
// on 2026-11-03: canonical JSON, ids, did:key for P-256 keys, and signature checks. No dependencies. The package is not on npm yet
// (no file: references in this house); replace with it once published, and keep in step until then.
const subtle = globalThis.crypto.subtle;
const B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
export function base58(bytes) {
  const digits = [0];
  for (const b of bytes) { let carry = b; for (let j = 0; j < digits.length; j++) { carry += digits[j] << 8; digits[j] = carry % 58; carry = (carry / 58) | 0; } while (carry) { digits.push(carry % 58); carry = (carry / 58) | 0; } }
  let s = ""; for (const b of bytes) { if (b) break; s += "1"; }
  for (let i = digits.length - 1; i >= 0; i--) s += B58[digits[i]];
  return s;
}
export const b64u = (bytes) => btoa(String.fromCharCode(...new Uint8Array(bytes))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
export const unb64u = (s) => Uint8Array.from(atob(String(s).replace(/-/g, "+").replace(/_/g, "/").padEnd(Math.ceil(s.length / 4) * 4, "=")), (c) => c.charCodeAt(0));
export const hex = (bytes) => [...new Uint8Array(bytes)].map((b) => b.toString(16).padStart(2, "0")).join("");
export function didFromJwk({ x, y }) {
  const X = unb64u(x), Y = unb64u(y); if (X.length !== 32 || Y.length !== 32) throw new Error("not a P-256 point");
  const out = new Uint8Array(2 + 33); out[0] = 0x80; out[1] = 0x24; out[2] = (Y[31] & 1) ? 0x03 : 0x02; out.set(X, 3);
  return "did:key:z" + base58(out);
}
export const isDid = (s) => typeof s === "string" && /^did:key:z[1-9A-HJ-NP-Za-km-z]{40,60}$/.test(s);
export const isAccountDid = (s) => isDid(s) || (typeof s === "string" && /^did:plc:[a-z2-7]{24}$/.test(s));
export function canonical(value) {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return "[" + value.map((v) => canonical(v === undefined ? null : v)).join(",") + "]";
  return "{" + Object.keys(value).filter((k) => value[k] !== undefined).sort().map((k) => JSON.stringify(k) + ":" + canonical(value[k])).join(",") + "}";
}
export async function sha256(bytesOrString) { const data = typeof bytesOrString === "string" ? new TextEncoder().encode(bytesOrString) : bytesOrString; return new Uint8Array(await subtle.digest("SHA-256", data)); }
export const idOf = async (obj) => hex(await sha256(canonical(obj)));
export async function verifyObject(jwk, obj, sigB64u) {
  const key = await subtle.importKey("jwk", { kty: "EC", crv: "P-256", x: jwk.x, y: jwk.y, ext: true }, { name: "ECDSA", namedCurve: "P-256" }, true, ["verify"]);
  return subtle.verify({ name: "ECDSA", hash: "SHA-256" }, key, unb64u(sigB64u), new TextEncoder().encode(canonical(obj)));
}
