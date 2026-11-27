// F5: a fire that has not happened. Marin County, California, 1 October 2029: an ignition at the
// Devil's Gulch campsite in Samuel P. Taylor State Park, on our own engine, under a wind borrowed
// from a real day. The stake is a house: 550 Nicasio Valley Road, which Anselm describes as the home
// of Jim Fournier's Planetwork and Solstice Grove. Nothing here is a forecast. The year has no
// weather record, so the wind is NOAA's 3 km analysis for 27 to 29 October 2019, the Red Flag
// Warning the Bay Area forecast office issued for the coastal North Bay from 03Z on the 27th to 18Z
// on the 28th (Iowa Environmental Mesonet VTEC archive, event MTR FW.W 6 of 2019), played hour for
// hour from midnight Pacific daylight time on 1 October 2029. The ground is today's: Copernicus
// elevation, WorldCover 2021 land cover, OpenStreetMap buildings and roads. Nobody fights the fire.

// where: the window every module here works on, and its resolution
const place = { lat_s: 37.97, lat_n: 38.08, lon_w: -122.79, lon_e: -122.61, res_arcsec: 1 };

const run = {
  name: "f5-nicasio-2029",
  start: "2029-10-01T07:00:00Z",
  end: "2029-10-04T07:00:00Z",
  seed: 1,
  close_every: "1d",
  snapshot_every: "10d",
  notes: "A scenario, not a forecast: an ignition at the Devil's Gulch campsite, Samuel P. Taylor State Park, Marin County, on 1 October 2029 under the Red Flag wind of 27 to 29 October 2019 (HRRR, hour for hour), on today's ground; the stake is 550 Nicasio Valley Road; unsuppressed; constants carried over from the Palisades.",
};

// The parts of the ground and the air, each a module of its own. They stand before the fire
// because it takes the ground from them when the run starts.
const ground = { kind: "ground.terrain", params: { place } };
const cover = { kind: "ground.cover", params: { place } };
const roads = { kind: "roads.major", params: { place } };
const buildings = { kind: "built.footprints", params: { place, source: "osm" } };
const weather = {
  kind: "weather.surface",
  params: { place, source: "hrrr", start: "2019-10-27T07:00:00Z", end: "2019-10-30T07:00:00Z", at: "2029-10-01T07:00:00Z" },
};
const moisture = { kind: "fuel.moisture", params: {} };

// Animals on the wildland, a placeholder population that runs from the front.
const animals = { kind: "fauna.flee", params: { density_km2: 20 } };

const fire = {
  kind: "fire.cells",
  params: {
    place,
    inputs: "world",
    // Devil's Gulch Campsite: OpenStreetMap relation 17685328, centre 38.0316, -122.7328 (read 2026-11-08)
    ignition_lat: 38.0316,
    ignition_lon: -122.7328,
    // 03:00 Pacific daylight time on 1 October 2029, in the strongest hours of the borrowed night
    ignition_time: "2029-10-01T10:00:00Z",
    form: "vector",
    cooling: "idle",
    reach_m: 45,
    ignite_scale: 5,
    slope_rate_gain: 20,
    frame_res_arcsec: 4,
  },
};

const places = {
  kind: "obs.places",
  params: {
    place,
    places: [
      // the US Census Bureau geocoder's interpolation of the street address along the TIGER line (2026-11-08); the parcel itself is not checked
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
export { run, ground, cover, roads, buildings, weather, moisture, fire, animals, places, suppression };
