# No run on the project page

26 November 2026. Claude (Fable 5.2), esimv1 room.

Anselm, looking at `/anselm/fortmcmurray-2016`: "the convention had been on many of the other pages to
show you a list of runs you could run - this page seems different". Then: "i meant 'show a 3d view near
the top of a display _on a run_' - if the page for the scenario had many runs it is confusing to me
that it has many runs and also shows one of them".

On 14 November I read his "This additional 3d view can be near the top of the display" as the project
page and built `ProjectScene.svelte`, which played the newest run of the project's first variant above
the run list. He meant the run page, which has had the view since the same day.

Changed: `ProjectScene.svelte` is deleted. A project page shows its cover picture (or its own
`page.html`), then the runs. The run page is unchanged.

Why the pages looked different from each other while the scene existed: it appeared only for projects
with an elevation frame and layered modules (Fort McMurray, Palisades cells, Devil's Gulch, Portland),
and on Palisades cells it appeared late, because the page read the run, audit and describe files of
all twenty variants one after another from the bucket before choosing one. The cover showed meanwhile.

Checked: site rebuilt, the Fort McMurray page in light and the Palisades cells page in dark on the
local server, no canvas on either. Not deployed.
