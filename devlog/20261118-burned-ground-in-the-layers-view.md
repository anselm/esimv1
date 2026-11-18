# Burned ground in the layers view

18 November 2026. Written by Claude (Claude Code session in the esimv1 room on canada1), on Anselm's order.

Anselm: "i think we did a fairly recent change to the 3d rendering layer for the pallisades fire to bring it in line
with the general engine used on the portland-cascades example - in doing so however we definitely muddied the
distinction between unburned and burned/on-fire area itself."

He was right. Since 14 November the run page leads with the layers view (`web/src/lib/Layers3D.svelte`), which drew
the fire's arrival-hour frame like any other frame: faint additive points in the disturbance layer's orange, dimmest
where the fire arrived first. The roads the fire module carries were drawn in the same orange, so burned ground,
roads and contour lines ran together. The older scene (`Scene3D.svelte`) is still further down the page and was
never affected.

What changed, in the layers view only:

- A frame of arrival hours is drawn by how long ago the fire arrived. The latest hour in the frame is taken as the
  present. Ground reached in the last two hours is the front, bright yellow. Ground behind it is filled, cooling
  from orange to a steady dark red, and laid opaque so the contours do not show through. Unburned ground gets no
  point.
- Roads carried by a disturbance module are drawn in the flows layer's pale colour. Portland's roads are unchanged.
- The caption under the scene says what the fill and the bright edge mean when such a frame is shown.

Limits. "On fire" here is a drawing rule (reached within two hours), not something the engine reports: `fire.cells`
writes arrival hours and no burning-now frame. The rule applies to any frame whose name contains `arrival` with unit
hours. The fill is square sprites, so steep slopes facing away from the camera still show thin gaps. Checked by
screenshot on `anselm/palisades-cells/vector-5` at 8 and 27 hours and on Portland; the other fires (Fort McMurray,
Nicasio, the Rothermel runs) use the same frame name and were not looked at.
