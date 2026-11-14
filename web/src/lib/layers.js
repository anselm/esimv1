// The layers of the world as a person would list them, in reading order, with the colour each
// takes in the 3D view. The ids match `LAYERS` in crates/modules/src/lib.rs; a module's describe
// carries its layer. A layer with no module in a run is still listed, as empty.
export const LAYERS = [
  { id: 'sun', name: 'Sun', color: '#ffd27a' },
  { id: 'ground', name: 'Ground', color: '#35b3a4' },
  { id: 'water', name: 'Water', color: '#4a9dff' },
  { id: 'air', name: 'Air and weather', color: '#7fdcff' },
  { id: 'built', name: 'Built structures', color: '#f2b25c' },
  { id: 'flows', name: 'Flows through the built', color: '#fff0b8' },
  { id: 'rules', name: 'Law and money', color: '#b9a0ff' },
  { id: 'plants', name: 'Plants', color: '#5fd58a' },
  { id: 'animals', name: 'Animals', color: '#b4e36a' },
  { id: 'people', name: 'People', color: '#ff8fb5' },
  { id: 'across', name: 'Disturbances', color: '#ff6a4d' },
  { id: 'record', name: 'The record', color: '#d9e2e8' },
]
// what a failed, closed or dark thing is drawn in, whatever its layer
export const HARM = '#ff4d3d'
