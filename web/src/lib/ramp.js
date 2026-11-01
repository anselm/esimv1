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
