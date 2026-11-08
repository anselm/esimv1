// Eight draws of the wild-card deck on the planet with emissions held, as one ensemble.
//
// A JavaScript manifest, the source form of RFC 0001. It starts from the Earth 2100
// manifest as it stands (imported, includes folded), restates two entities on top of it,
// and declares the ensemble as a range. What the engine reads is the plain entity list
// this evaluates to; `esim manifest manifests/j3-planet-wild-members.js` prints it.
//
// Why this exists: the wild cards (data/events/future-speculative.toml) are drawn each year
// by the run's seed, so the spread across seeds is the only honest way to show what the deck
// does. Every card's chance is a recalled order of magnitude and its effect a placeholder;
// read docs/events/wild-cards before any number.

import planet from "./p4-planet-people.toml";

const members = 8;
const base = (id) => planet.find((e) => e.id === id);

// the Earth 2100 arrangement, entity by entity, in its own order
export default planet;

// a later entity with an earlier id replaces it component by component
export const run = {
  ...base("run").run,
  name: "j3-planet-wild-members",
  notes: "Earth 2100 with emissions held at the 2024 rate and the wild-card deck in play; one member per seed. " + base("run").run.notes,
};

export const wildcards = {
  kind: "events.shock",
  params: { ...(base("wildcards").params || {}), mode: "draw" },
};

// the one loop the transport keeps: a range unrolls to one variant per member
export const seeds = {
  title: "Deck drawn by seed",
  notes: "The same planet, the same held emissions, a different draw of the deck each member.",
  range: { "run.seed": { from: 1, to: members, steps: members } },
};
