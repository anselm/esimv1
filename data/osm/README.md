# OpenStreetMap answers, kept

Reduced answers from the Overpass API: roads and place names for a window (`context`), bridge
lines (`bridges`), buildings and shorelines. They are kept here, in git, where every other fetched
file lives in `data/cache/` and is fetched again when missing. The reason is Overpass: it is slow
and unreliable (ten minutes for Portland's roads on 14 November 2026, one mirror holding a
connection open without answering, the other answering 502), and a run that takes three seconds
should not wait on it. A box that has pulled the repository reads these files and asks nothing.

A file's name is its request: the kind and the window (south, north, west, east). The raw
answer's hash, its URL and the day it was fetched are in `data/catalog.lock.toml` under the same
key. To refresh one, delete it and run a manifest that needs it. A new window is fetched once, on
the first run that asks for it (the fetcher gives up after 300 seconds), and the file should then
be committed.

© OpenStreetMap contributors, Open Database License.
