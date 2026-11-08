// What the source form is for: variables, loops and imports that the transport does not
// carry. This imports the planet above, restates one entity, stamps out a sweep over the
// pre-industrial CO2 with a loop, and declares a thirty-member ensemble as a range, the
// one loop the transport keeps. The output is a plain entity list; see
// `esim manifest manifests/j2-planet-members.js`.

import * as planet from "./p1-planet-co2-1850-2025.js";

const members = 30;
const starts = [284, 286.8, 290];

export const run = { ...planet.run, name: "j2-planet-members" };
export const { emissions, carbon, energy, obs, audit } = planet;

// a loop: one variant per starting concentration, named by its value
export const sweep = starts.map((ppm) => ({
  id: `start-${String(ppm).replace(".", "p")}`,
  variant: { title: `Pre-industrial CO2 at ${ppm} ppm`, set: { "carbon.co2_start_ppm": ppm, "energy.co2_ref_ppm": ppm } },
}));

// a range: the ensemble stays one entity in the document and unrolls in the engine
export const seeds = { title: "Members", range: { "run.seed": { from: 1, to: members, steps: members } } };
