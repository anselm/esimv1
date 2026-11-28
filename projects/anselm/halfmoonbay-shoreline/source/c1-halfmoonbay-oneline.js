// C1: the Half Moon Bay shoreline south of Pillar Point Harbor, 1961 to 2025, on a one-line
// shoreline model. The Corps built the harbor's East Breakwater in 1959 to 1961; since then the
// bluffs south of it have retreated at up to 2 m a year where they had retreated at 8 cm (Griggs
// 2026), the beaches from the breakwater to Mirada Road lose 4,200 cubic yards a year, and sand
// surges through the breakwater into a shoal that blocks the boat launch (USACE 2015 draft EA,
// citing Lin et al. 2015). The model: today's shoreline from OpenStreetMap as transects, the
// ERA5-Ocean waves of the half-degree cell over the bay hour by hour, the headland and the
// breakwater as shadows with diffraction, the CERC transport, Pelnard-Considere's line, a bluff
// where the beach runs out. Two numbers are fitted and declared below. Not a fluid dynamics model,
// and it says so. The record is in data/events/halfmoonbay/record.toml.

// where: the window every module here works on, and its resolution
const place = { lat_s: 37.42, lat_n: 37.52, lon_w: -122.52, lon_e: -122.42, res_arcsec: 1 };

// what: the products of the data warehouse this scenario reads (https://data.earthlight.cloud), each named by
// its address there. A product never changes once made, so these lines fix the bytes a run reads.
const data = {
  shore: "shore/osm/half-moon-bay/2026-11-28",
  roads: "roads/osm/half-moon-bay/2026-11-27",
  names: "names/osm/half-moon-bay/2026-11-28",
};

const run = {
  name: "c1-halfmoonbay-oneline",
  start: "1961-01-01",
  end: "2025-01-01",
  seed: 1,
  close_every: "30d",
  snapshot_every: "3650d",
  notes: "The Half Moon Bay shoreline south of the Pillar Point Harbor breakwater, 1961 to 2025, on a one-line model with the reanalysis waves and the structures' shadows; today's shoreline, so rates are measured, not positions; the transport coefficient and the leak through the breakwater fitted to the Corps' rates and declared; everything else recalled or guessed; not a fluid dynamics model.",
};

const coast = {
  kind: "coast.oneline",
  params: {
    place,
    data,
    breakwater: "on",
    // fitted (2026-11-08, a sweep of 0.39, 0.05, 0.02, 0.01, 0.005): the one-line model's transport
    // coefficient, 80 times below the open-coast value, because the reanalysis cell's wave height is
    // the open ocean's and the point's refraction and the shelf's losses are not in the model; at this
    // value the as-built bluff behind Surfers and Vallejo Beaches retreats about 1 m a year against the
    // Corps' 1.64 ft (0.5 m) and Griggs's up to 2 m
    k_cerc: 0.005,
    // fitted: the share of the sand moving along the first 300 m that passes through the breakwater's
    // voids, set so the harbor takes about 1,300 m3 a year, the order of the Corps' "approximately
    // 2,000 CY" and of the 75,000 to 100,000 cubic yards the District dredged in 2025 as the
    // accumulation since 1961
    leak_frac: 0.2,
  },
};

// What the scenario is made of, in order. The order is the order of sections on the run page,
// and it settles which of two modules hears an event first when their priorities are equal.
export { run, coast };
