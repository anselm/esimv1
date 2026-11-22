// Where the viewer finds things that are not inside a run folder. A run folder is always handed
// to a component as `base`, its address; these are the few other addresses, with defaults that
// match the esimv1 hub. A page that hosts the viewer elsewhere calls `configure` once.
export const places = {
  projects: '/projects',                              // <projects>/<area>/<name>/{project.toml, README.md, runs/index.json}
  outlines: '/docs/outlines.json',                    // coastlines for maps: rings of [lon, lat]
  countryOutlines: '/docs/country-outlines.json',     // country borders for the globe
}
export function configure(over) { Object.assign(places, over) }
