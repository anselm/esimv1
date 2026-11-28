// F6: the Devil's Gulch scenario of F5 on the older engine: Rothermel's 1972 spread model on the
// same 30 m ground, fuel from land cover through the Scott and Burgan models, the wind as the ERA5
// reanalysis at one 25 km point, borrowed from the same Red Flag days (27 to 29 October 2019) and
// played hour for hour from midnight Pacific daylight time on 1 October 2029. The two engines on one
// scenario, as on the Palisades, so the reader can see what is the fire and what is the engine.
// Nothing here is a forecast. Nobody fights the fire.

// where: the window every module here works on, and its resolution
const place = { lat_s: 37.97, lat_n: 38.08, lon_w: -122.79, lon_e: -122.61, res_arcsec: 1 };

// what: the products of the data warehouse this scenario reads (https://data.earthlight.cloud), each named by
// its address there. A product never changes once made, so these lines fix the bytes a run reads.
const data = { roads: "roads/osm/nicasio/2026-11-27", names: "names/osm/nicasio/2026-11-28" };

const run = {
  name: "f6-nicasio-2029-rothermel",
  start: "2029-10-01T07:00:00Z",
  end: "2029-10-04T07:00:00Z",
  seed: 1,
  close_every: "1d",
  snapshot_every: "10d",
  notes: "A scenario, not a forecast: the Devil's Gulch ignition of 1 October 2029 on Rothermel's spread model with the ERA5 point wind of 27 to 29 October 2019 played hour for hour; the stake is 550 Nicasio Valley Road; unsuppressed; fuel parameters recalled from Scott and Burgan 2005 and not yet checked.",
};

// The parts of the ground and the air, each a module of its own, the same ones the other engine
// reads in F5. They stand before the fire because it takes the ground from them when the run starts.
const ground = { kind: "ground.terrain", params: { place } };
const cover = { kind: "ground.cover", params: { place } };
// the ERA5 hours of the borrowed days, one 25 km point for the whole window; midnight UTC of the
// first day falls on midnight UTC of 1 October 2029
const weather = {
  kind: "weather.surface",
  params: { place, source: "era5", start: "2019-10-27T00:00:00Z", end: "2019-10-31T00:00:00Z", at: "2029-10-01T00:00:00Z" },
};
const moisture = { kind: "fuel.moisture", params: {} };

// Animals on the wildland, a placeholder population that runs from the front.
const animals = { kind: "fauna.flee", params: { density_km2: 20 } };

const fire = {
  kind: "fire.rothermel",
  params: {
    place,
    data,
    inputs: "world",
    // Devil's Gulch Campsite: OpenStreetMap relation 17685328 (read 2026-11-08); 03:00 Pacific daylight time
    ignition_time: "2029-10-01T10:00:00Z",
    ignition_lat: 38.0316,
    ignition_lon: -122.7328,
  },
};

const places = {
  kind: "obs.places",
  params: {
    place,
    places: [
      { name: "550 Nicasio Valley Road", lat: 38.0283, lon: -122.6675, note: "the home of Jim Fournier's Planetwork and Solstice Grove, as Anselm describes it; position interpolated along the road by the US Census geocoder, 2026-11-08" },
      { name: "Devil's Gulch campsite", lat: 38.0316, lon: -122.7328, note: "the ignition; OpenStreetMap relation 17685328, 2026-11-08" },
    ],
  },
};

const suppression = {
  kind: "fire.suppression",
  params: {
    place,
    mode: "off",
    line_width_m: 30,
  },
};

// What the scenario is made of, in order. The order is the order of sections on the run page,
// and it settles which of two modules hears an event first when their priorities are equal.
export { run, ground, cover, weather, moisture, fire, animals, places, suppression };
