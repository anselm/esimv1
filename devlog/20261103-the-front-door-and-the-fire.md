# 2026-11-03 — Earth 2100, covers, the front door, accounts, and the Palisades fire

Anselm, after the push and deploy of the README work, five things: "1) rename 'The planet,
with people on it' to 'Earth 2100'. 2) allow a depiction or piece of art per scenario. 3) add
attest.monster login accounts; users will create real root folders with their handle -
meaning handles have to test if folders are available. a critical issue is to find my recent
earlier work on hashed swear / bad words filter [...]. 4) have the front page statically
feature scenarios that we like - feature just the earth 2100 scenario and the fire scenario
(below) when it is ready. 5) simulate the los angeles pallisades fires. this will require
substantial work around elevation, wind behavior. but since it is a historical event we can
test our simulation results. it may be nice to model evacuation and emergency firefighter
response. this may be substantial work but i feel this is the real challenge - the real kind
of scenario we need to handle." And, when the filter turned up: "if attest has the swear word
guard then we don't have to build it for the earth sim project whew".

Agent: Claude (Fable 5.1), the earth-simulator-1 room on canada1, session
`0bad2b71-90e5-4e88-978e-bd10ad8801f7`. Commits 49ba8d4 (rename, covers, front page),
cfd41d8 (accounts) and the fire commit after this entry; none pushed. A forked agent found
the fire's data doors (its notes are in the session scratchpad; what matters is in the
catalog). The live site is at ae61643.

## 1, 2, 4: the front door

Earth 2100 is the project's name now. A project may carry a picture: `cover` and
`cover_caption` in `project.toml`, a file beside the manifest, shown on the project page
and on the front page's card. The house rule holds: a picture drawn from the runs, a real
photograph, or art made for the scenario, never stock. Both covers today are drawn from the
runs' own frames: Earth 2100's is the held run's people outside the climate niche in 2091;
the fire's is where the model's fire had reached at 27 hours with the mapped perimeters over
it. `featured = true` puts a project on the front page, which now shows only the featured
scenarios as cards, with `?all` for the rest. A README may show a picture from its folder.

## 3: accounts

The filter he remembered is attest's own `packages/handle-guard` (twelve languages as SHA-256
hashes, reserved paths), and attest applies it when a handle is made, so this project does
not need it. What this project needed was the other half: on a public, writable copy
(`ESIM_PUBLIC=1`) the server verifies an attest delegation on the socket handshake
(matchmaker's pattern, 2026-09-29, adapted; attest's verifier copied until the package is on
npm), `hello` names the attest and the signed-in handle, and a person may create, run and
write only under the area that is their handle, claimed into `projects/<handle>/area.toml`
with the DID on first use. The server keeps its own short list of reserved area words. The
corner of every page shows "sign in" or "@handle"; the new-project form locks the area to
the handle and says whether it is free. `web/scripts/attest-e2e.mjs` runs the whole flow
against a local attest with a scratch database and Chrome's virtual authenticator: eighteen
checks, passing. The live esimv1 copy is read-only and none of this changes it; a public
writable copy needs a machine with Rust and the data cache, which is the deployment question
the todo already holds.

## 5: the Palisades fire

### The data, all anonymous

- Terrain: Copernicus GLO-30, tile N34_W119, at one arc-second (the DEM's own 30 m); slope and
  aspect by Horn's method, already in the registry.
- Fuel: ESA WorldCover 2021 v200, the N33W120 tile (96.5 MB, DEFLATE, Byte); a windowed reader
  (`cog::read_window_u8`) decodes only the tiles under the window; each 30 m cell takes its
  most common 10 m class. In the window: tree cover 196 km², shrubland 45, grassland 39,
  built-up 65, water 80. WorldCover calls most of the Santa Monica Mountains "tree cover".
- Wind: Open-Meteo's ERA5 archive at the window's centre, hourly, with direction and gusts.
  First night: 6 to 8 m/s from the north-north-east, gusts 12 to 16; news reports put the
  ridge gusts at two and three times that (recalled from the reporting, not fetched). A 25 km reanalysis reads a Santa Ana event mild.
- Perimeters: NIFC's Operational Data Archive 2025 (layer 6, Event_Polygon) by IRWIN id gives
  34 daily perimeters from 06:00Z on 8 January to the 21st; the WFIGS layer gives the
  perimeter of record (23,448 acres). The first "perimeter" (2,920 acres, 06:00Z) is a
  hand-drawn triangle with a typed acreage; the first infrared map is 27 hours in (15,824).
  Nothing is mapped between discovery (18:30Z on the 7th) and 06:00Z on the 8th.
- Ignition: 34.0723, -118.5451 at 18:30Z on 7 January, the California WFTIIC ignition point
  and CAL FIRE's start time. CAL FIRE's record: 23,448 acres, 6,845 structures destroyed,
  12 civilian deaths, contained 31 January.
- Not used: NASA FIRMS (needs a key), LANDFIRE fuels (needs an email).

### The model

`fire.rothermel`: Rothermel 1972 in Andrews 2018's two-category form, dead fine fuel moisture
by Simard from the hour's humidity and temperature, the Albini-Baughman unsheltered wind
adjustment by fuel depth, Rothermel's 0.9·IR wind limit, the wind and slope vectors summed to
a heading, Anderson's ellipse (the FARSITE coefficients, wind in m/s). Fire moves cell to
cell: a burning cell pushes progress into every unburned cell within three cells along the 32
primitive lattice directions, at the ellipse's rate in that direction over the straight-line
distance, provided the straight path crosses no unburnable cell; the cell ignites when fire
from any one direction has covered the distance. Progress is kept per approach direction
because summing over neighbours let a line front advance several times the rate (the first
run's mistake). Fifteen-second substeps inside an hourly weather step. Built-up land carries
fire at a guessed 1 m/min at 10 m/s, scaled by the wind, which a variant sets to zero.
`obs.perimeters` schedules an event at each mapped perimeter's time, rasterises it on the
run's grid (even-odd scanline) and reports the mapped acres, the burned acres, the Sørensen
overlap, the missed and the overrun acres; `[audit] scores` pairs sim.acres with obs.acres.

Every fuel parameter in `data/events/fuels.toml` is recalled from Scott and Burgan 2005 and
unchecked; every row says so. Nothing is tuned.

### The result

One run of four days takes about fifteen seconds, three of them the fire. Burned acres by
hour after ignition, the record in the last column:

| hours | reanalysis | gusts | built-barrier | SH7 | record |
|---|---|---|---|---|---|
| 4 | 18,907 | 7,149 | 18,420 | 14,682 | — |
| 12 | 70,250 | 55,772 | 63,688 | 67,591 | 2,920 (triangle) |
| 27 | 79,107 | 75,668 | 67,843 | 78,762 | 15,824 (first IR map) |
| 48 | 80,295 | 80,043 | 68,139 | 80,171 | 15,824 |
| 96 | 80,304 | 80,898 | 68,151 | 80,193 | 21,317 |

The model overruns the record about fourfold at 27 hours, more than twentyfold at the first
mapped figure (11.5 hours),
and fills every burnable cell in the window by the second day: Malibu to Brentwood. The
Sørensen overlap at 27 hours is a third, because the model's fire covers the real one and
goes far beyond it. None of the variants closes the gap; the barrier variant removes the
houses from the count. The gust variant is slower in its first hours and faster later: a
narrower ellipse spreads less area per unit of head advance.

What this is: an upper bound, the fire the land would allow with nobody fighting it, no roads,
no fuel breaks, no 2021 burn scar, on a landscape where every classified cell carries fuel,
with a fuel table nobody has checked. Rothermel's SH5 at the reanalysis wind gives a head
rate of about 50 m/min; the real fire's first night averaged under 10. What it is not: a
forecast, or evidence about the fire. The things it lacks are listed in the limits page and
the todo, in the order to build them: the fuel table checked, barriers from OpenStreetMap
and the fire history, suppression as an agent, then spotting, then evacuation and the people.

His speed question is answered on the way: a fire day at 30 m runs in three seconds, so
operator-in-the-loop is a surface question (live wind, a clock that advances with the wall,
decisions injected and replayed), not a model question.

### Found on the way

- The first run's front moved several times too fast because progress summed over burning
  neighbours; per-direction progress fixed it, and the model still overruns.
- With eight neighbours a narrow ellipse was starved: the gust variant burned less than the
  mean-wind variant. Thirty-two directions within three cells, with a barrier check along the
  path, made the gust variant behave; striping along lattice directions remains visible.
- A scratchpad `numbers.py` shadowed Python's `numbers` module when a script ran from that
  folder; renamed, and scripts now run with `-I`.
- The arrival-time frame was never written because `i64::MIN` arithmetic overflowed; fixed.

## Open

Push and deploy are his call. The held-low variant's place on the compare page is still his
call. The fire's next steps are in the todo under "The fire".
