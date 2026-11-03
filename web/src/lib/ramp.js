// Sequential colour ramp for magnitude fields: the dataviz skill's one-hue
// blue ramp (steps 100..700), light to dark, identical in light and dark mode.
// NaN cells are transparent.

export const RAMP = [
  '#cde2fb', '#b7d3f6', '#9ec5f4', '#86b6ef', '#6da7ec', '#5598e7', '#3987e5',
  '#2a78d6', '#256abf', '#1c5cab', '#184f95', '#104281', '#0d366b',
]

const rgb = RAMP.map((h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16)))

// A 256-entry lookup table of [r,g,b], interpolated between the ramp steps.
export const LUT = Array.from({ length: 256 }, (_, i) => {
  const x = (i / 255) * (rgb.length - 1)
  const a = Math.floor(x), b = Math.min(a + 1, rgb.length - 1), f = x - a
  return rgb[a].map((c, k) => Math.round(c + (rgb[b][k] - c) * f))
})

// CSS gradient for the legend bar.
export const RAMP_CSS = `linear-gradient(to right, ${RAMP.join(', ')})`

// A warm ramp for fire: the hour a cell burned, early light, late dark.
export const FIRE = ['#fff3b0', '#ffd166', '#f7a541', '#ec6f3b', '#d6403c', '#a8233a', '#6a1230']
// Categorical colours for ESA WorldCover classes, keyed by code (sober, legible on both themes).
export const COVER = { 10: '#2f6b3a', 20: '#9c8a3c', 30: '#c9c37a', 40: '#d9b15b', 50: '#8a8a94', 60: '#b9a48c', 70: '#e8eef5', 80: '#2d5c9e', 90: '#5f9f8d', 95: '#2f7d6d', 100: '#a7b7a0' }
export const COVER_NAMES = { 10: 'tree cover', 20: 'shrubland', 30: 'grassland', 40: 'cropland', 50: 'built-up', 60: 'bare', 70: 'snow and ice', 80: 'water', 90: 'wetland', 95: 'mangroves', 100: 'moss and lichen' }
const lut = (ramp) => { const c = ramp.map((h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16))); return Array.from({ length: 256 }, (_, i) => { const x = (i / 255) * (c.length - 1); const a = Math.floor(x), b = Math.min(a + 1, c.length - 1), f = x - a; return c[a].map((v, k) => Math.round(v + (c[b][k] - v) * f)) }) }
export const FIRE_LUT = lut(FIRE)
export const FIRE_CSS = `linear-gradient(to right, ${FIRE.join(', ')})`
// Which ramp a field gets, from its name and unit: fire arrival hours warm, land-cover codes categorical, everything else blue.
export function rampFor(name, unit) {
  if (unit === 'worldcover code') return 'cover'
  if (/arrival|burn|fire/.test(name) && unit === 'h') return 'fire'
  return 'blue'
}
