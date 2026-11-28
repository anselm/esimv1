// F4: the Fort McMurray (Horse River) fire of May 2016 on our own engine, the first fire outside
// the Palisades window and outside the United States. The ground is the same kind of ground
// (Copernicus elevation, WorldCover land cover), the buildings come from OpenStreetMap because no
// county footprint service exists here, the wind is the ERA5 reanalysis at one point because the
// 3 km HRRR analysis stops at the border, and the record is the satellite hotspots, because nobody
// mapped this fire hour by hour. Every constant is the Palisades' guess until the record says
// otherwise. Nobody fights the fire.

// where: the window every module here works on, and its resolution
const place = { lat_s: 56.55, lat_n: 56.85, lon_w: -111.75, lon_e: -111.15, res_arcsec: 1 };

// what: the products of the data warehouse this scenario reads (https://data.earthlight.cloud), each named by
// its address there. A product never changes once made, so these lines fix the bytes a run reads.
const data = {
  roads: "roads/osm/fort-mcmurray/2026-11-28",
  names: "names/osm/fort-mcmurray/2026-11-28",
  buildings: "buildings/massing/fort-mcmurray/2026-11-28",
};

const run = {
  name: "f4-fortmcmurray-cells",
  start: "2016-05-01T22:00:00Z",
  end: "2016-05-07T00:00:00Z",
  seed: 1,
  close_every: "1d",
  snapshot_every: "10d",
  notes: "Our own engine on the Fort McMurray fire: heat budget on 30 m cells, OpenStreetMap buildings, ERA5 wind at one point; scored against CWFIS satellite hotspots; unsuppressed; constants carried over from the Palisades.",
};

// The parts of the ground and the air, each a module of its own. They stand before the fire
// because it takes the ground from them when the run starts.
const ground = { kind: "ground.terrain", params: { place } };
const cover = { kind: "ground.cover", params: { place } };
const roads = { kind: "roads.major", params: { place, data } };
const buildings = { kind: "built.footprints", params: { place, data } };
const weather = {
  kind: "weather.surface",
  params: { place, source: "era5", start: "2016-05-01T22:00:00Z", end: "2016-05-07T00:00:00Z" },
};
const moisture = { kind: "fuel.moisture", params: {} };

// Animals on the wildland, a placeholder population that runs from the front.
const animals = { kind: "fauna.flee", params: { density_km2: 20 } };

const fire = {
  kind: "fire.cells",
  params: {
    place,
    data,
    inputs: "world",
    // detected on the afternoon of 1 May 2016 (local, MDT); the hour is recalled as about 16:00 and not
    // yet checked against the provincial review; the point is the Wikipedia infobox's
    ignition_time: "2016-05-01T22:00:00Z",
    ignition_lat: 56.66528,
    ignition_lon: -111.38333,
    form: "vector",
    cooling: "idle",
    reach_m: 45,
    ignite_scale: 5,
    slope_rate_gain: 20,
    frame_res_arcsec: 4,
  },
};

const obs = {
  kind: "obs.hotspots",
  params: {
    place,
    file: "fortmcmurray-2016/cwfis/hotspots.csv",
    start: "2016-05-01T00:00:00Z",
    end: "2016-05-07T00:00:00Z",
    every: "6h",
    frame_res_arcsec: 4,
  },
};

const audit = {
  scores: [["sim.acres", "obs.acres"]],
};

// What the scenario is made of, in order. The order is the order of sections on the run page,
// and it settles which of two modules hears an event first when their priorities are equal.
export { run, ground, cover, roads, buildings, weather, moisture, fire, animals, obs, audit };
