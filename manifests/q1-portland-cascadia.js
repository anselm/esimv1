// Portland through 2027, with a Cascadia earthquake on 1 February.
//
// The magnitude 9.0 rupture of the Cascadia subduction zone that the State of Oregon plans for,
// set at two in the afternoon on Monday 1 February 2027 (2 PM is the state geology agency's
// daytime case) on soils saturated by the winter rains. The date is a scenario choice, not a
// prediction. The run is the calendar year, in Pacific time.
//
// Each export is one entity of the scenario and its name is the entity's id. Everything specific
// to Portland is in one folder of tables, data/events/portland-cascadia: the ground maps, the
// building inventory, the tanks, bridges, hospitals, routes and the published studies the result
// is held against. The modules are general; this file points them at that folder.

const folder = "portland-cascadia";
const soils = "wet";          // saturated soil: the higher-loss case, with liquefaction
const time_of_day = "day";    // people where the inventory places them at 2 PM

// a module of a kind, reading the scenario's folder
const m = (kind, params = {}) => ({ kind, params: { folder, ...params } });

export const run = {
  name: "q1-portland-cascadia",
  start: "2027-01-01T08:00:00Z",
  end: "2028-01-01T08:00:00Z",
  seed: 1,
  close_every: "1d",
  snapshot_every: "30d",
  notes: "A magnitude 9.0 rupture of the Cascadia subduction zone as it affects Portland, set at 2 PM PST on Monday 1 February 2027 as a scenario choice and not a prediction. The run covers the calendar year 2027 in Pacific time, so it begins a month before the earthquake and ends eleven months after it. Ground motion, soils (saturated) and the building inventory with daytime occupants are DOGAMI's; tanks, bridges, transmission structures and route segments are damaged as components; water follows an assumed schedule. Every constant is published, recalled or declared a guess in its module's description.",
};

export const place = { lat_s: 45.43, lat_n: 45.66, lon_w: -122.85, lon_e: -122.47, res_arcsec: 1 };

// what is left that is Portland's own: drinking water's schedule, the story, the day's state
export const lifelines = m("portland.lifelines", { storage_cut: 0.0, retrofit: false, inputs: "computed" });

// the things that break, one module per subject
export const tanks = m("fuel.tanks", { storage_cut: 0.0, retrofit: false });
export const bridges = m("roads.bridges");
export const power = m("power.structures", { soils });
export const routes = m("roads.routes", { soils });

// where the water is, and where the tanks' releases toward water arrive on it (no flow yet)
export const water = { kind: "water.surface" };

// the fuel account and the hospitals; the `ranges` variant switches both off and lets
// portland.lifelines run its older version, which keeps them itself
export const depot = m("fuel.depot", { storage_cut: 0.0 });
export const hospitals = m("health.hospitals");

// people: what happens to them afterwards, and the residents as agents of ten people each
export const aftermath = m("people.aftermath", { summary_day: 333 });
export const agents = m("people.agents", { scale: 10, time_of_day });

// the earthquake: published ground maps, announced at the moment set for it
export const ground = m("quake.ground", { at: "2027-02-01T22:00:00Z", tz_offset_h: -8, tz_name: "PST", soils });

// Buildings. Six values are fitted to the state's own loss estimate for the same ground, wet and
// by day: the shaking medians divided by 1.2, the complete median at 1.1 g, ground failure counted
// as complete with a 25 cm median, a quarter of complete shaking damage collapsing, every casualty
// rate tripled, and 65 percent of residents of badly damaged buildings displaced. So the toll is
// close to that estimate because it was made to be.
export const buildings = m("quake.buildings", {
  time_of_day,
  fragility_scale: 1.2,
  median_complete_g: 1.1,
  gf_median_cm: 25,
  gf_complete_share: 1.0,
  collapse_given_complete: 0.25,
  casualty_scale: 3.0,
  displaced_share: 0.65,
});

// the published study the buildings' result is held against
export const losses = m("obs.losses", { record: soils, time_of_day });

export const audit = { shock_topics: ["quake.rupture", "lifeline.failed"] };
