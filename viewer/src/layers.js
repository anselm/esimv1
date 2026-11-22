// The layers of the world as a person would list them, in reading order, with the colour each
// takes in the 3D view. The ids match `LAYERS` in crates/modules/src/lib.rs; a module's describe
// carries its layer. A layer with no module in a run is still listed, as empty.
// The numbers order the layers and leave room between them: a new layer between water and air
// would be 3500. Disturbances and the record have no number, because they are not places in the order.
export const LAYERS = [
  { id: 'sun', n: 1000, name: 'Sun', color: '#ffd27a' },
  { id: 'ground', n: 2000, name: 'Ground', color: '#35b3a4' },
  { id: 'water', n: 3000, name: 'Water', color: '#4a9dff' },
  { id: 'air', n: 4000, name: 'Air and weather', color: '#7fdcff' },
  { id: 'built', n: 5000, name: 'Built structures', color: '#f2b25c' },
  { id: 'flows', n: 6000, name: 'Flows through the built', color: '#fff0b8' },
  { id: 'rules', n: 7000, name: 'Law and money', color: '#b9a0ff' },
  { id: 'plants', n: 8000, name: 'Plants', color: '#5fd58a' },
  { id: 'animals', n: 9000, name: 'Animals', color: '#b4e36a' },
  { id: 'people', n: 10000, name: 'People', color: '#ff8fb5' },
  { id: 'across', name: 'Disturbances', color: '#ff6a4d' },
  { id: 'record', name: 'The record', color: '#d9e2e8' },
]
// what a failed, closed or dark thing is drawn in, whatever its layer
export const HARM = '#ff4d3d'
