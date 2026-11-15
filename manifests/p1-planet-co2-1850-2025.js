// The same manifest as p1-planet-co2-1850-2025.toml, as a JavaScript module: the source
// form of RFC 0001. The exports are the entities; the export name is the id; an export
// named run, place or audit that carries no component is that component. The engine
// evaluates this with nothing but the language (no clock, no randomness, no network, no
// files beyond imports of its neighbours) and reads the result, so the document two
// engines compare is the output, not this file. `esim manifest <file> --id` gives the same
// id for this file and the TOML one.

const start = "1850-01-01";
const end = "2025-01-01";
const preindustrial_ppm = 286.8;

const run = {
  name: "p1-planet-co2-1850-2025",
  start, end, seed: 1, close_every: "30d",
  notes: "Zero-dimensional planet, 175 years, monthly steps.",
};

const emissions = { kind: "carbon.emissions", params: { include_land_use: true } };
const carbon = { kind: "carbon.pools", params: { co2_start_ppm: preindustrial_ppm } };
const energy = { kind: "energy.twolayer", params: { co2_ref_ppm: preindustrial_ppm } };
const obs = { kind: "obs.planet", params: { baseline_from: 1850, baseline_to: 1900 } };

const audit = {
  scores: [
    ["carbon.co2_ppm", "obs.co2_ppm"],
    ["energy.gmst_anom_c", "obs.gmst_hadcrut5_c"],
    ["energy.gmst_anom_c", "obs.gmst_gistemp_c"],
  ],
};

// What the scenario is made of, in order. The order is the order of sections on the run page,
// and it settles which of two modules hears an event first when their priorities are equal.
export { run, emissions, carbon, energy, obs, audit };
