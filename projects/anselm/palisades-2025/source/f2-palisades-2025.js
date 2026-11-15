// F2: the Palisades fire, Los Angeles, January 2025, as a hindcast. Rothermel's spread
// model on the 30 m terrain of the Santa Monica Mountains with fuel from land cover and
// the hour's wind from the ERA5 archive, ignited where and when the record says, run with
// no suppression; the incident's mapped perimeters score the burned area as they were
// current. The window takes in the final perimeter with a margin on every side; the sea
// is water and does not burn.

// where: the window every module here works on, and its resolution
const place = { lat_s: 34.0, lat_n: 34.16, lon_w: -118.72, lon_e: -118.46, res_arcsec: 1 };

const run = {
  name: "f2-palisades-2025",
  start: "2025-01-07T18:00:00Z",
  end: "2025-01-12T00:00:00Z",
  seed: 1,
  close_every: "1d",
  snapshot_every: "10d",   // a snapshot holds every cell's progress in 32 directions, 335 MB as JSON; none inside a four-day run
  notes: "The Palisades fire's first four days under reanalysis wind, unsuppressed; scored against NIFC's daily perimeters. Fuel parameters recalled from Scott and Burgan 2005 and not yet checked; the reanalysis under-reads Santa Ana gusts.",
};

const fire = {
  kind: "fire.rothermel",
  params: {
    place,
    ignition_time: "2025-01-07T18:30:00Z",
    ignition_lat: 34.0723,
    ignition_lon: -118.5451,
    wind_start: "2025-01-06",
    wind_end: "2025-01-14",
  },
};

const obs = {
  kind: "obs.perimeters",
  params: {
    place,
    incident: "Palisades",
    year: 2025,
    irwin: "{A7EA5D21-F882-44B8-BF64-44AB11059DC1}",
  },
};

const structures = {
  kind: "obs.dins",
  params: {
    place,
    service: "https://services1.arcgis.com/jUJYIo9tSA7EHvfZ/arcgis/rest/services/DINS_2025_Palisades_Public_View/FeatureServer/0",
    key: "cal-fire-dins/2025-palisades",
  },
};

const record = {
  kind: "obs.record",
  params: {
    place,
    files: ["palisades-2025/nifc/palisades-event-points.geojson", "palisades-2025/nifc/palisades-event-lines.geojson"],
    time_field: "CreateDate",
  },
};

const audit = {
  scores: [["sim.acres", "obs.acres"]],
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
export { run, fire, obs, structures, record, suppression, audit };
