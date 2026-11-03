Observed in ghost/phase3/phase3.compact.clarc.json: the edge "ro (email preflight)" (bg -> content, edge 21) leaves the bottom of bg, runs left to x=50 (past the leftmost node), down to y=1390, then right to x=2810 and enters content from its left side. It breaks the left-to-right flow rule and still crosses ~10 edges (6, 11, 16, 19, 22, 25, 26, 28, 29, 31) according to the layout check.

## Cause (src/route.rs)
- Cost is lexicographic: CROSS = 1_000_000, TURN = 10_000, length in px. A detour of thousands of px is nearly free next to one crossing.
- No cost for travelling against the flow (leftwards for data edges).
- `intrusions` charges one CROSS per step inside a group holding neither end, so the direct path down from bg (through g_admin / g_readers) is expensive; the outer ring is the only path with zero intrusions (not yet confirmed as the main driver).
- The router's crossing model (straight-shot cuts + straight-through junctions) does not see all crossings that check.rs reports, so the ring looks free when it is not. Each edge is re-routed against a fixed snapshot of the others, 3 rip-up rounds only.

## Spec
1. Add a backward-travel cost to `route_edge` so a leftward step on a data edge costs more than a crossing or two.
2. Make the router's crossing count agree with check.rs (count crossings from turns/shared lanes at junctions, not only straight-through), so detours around the ring are not mistaken for crossing-free.

## Tests
- Regression test (breaks before, green after): bg -> content style case from the phase3 compact diagram; the route must not travel left of its source column / must not exceed a bound relative to Manhattan distance, and the router's crossing count for the edge must match check.rs.
- After the change, re-render ghost/phase3/phase3.compact.clarc.json with clarc (bump c43-layout rev in clarc, update ~/.local/share/clarc/clarc).

----- AI agent updates -------

## Specification (clarified)

Goal: a data edge is never routed against the left-to-right flow just to dodge crossings, and routes avoid piling onto lanes other routes already use, because that is where the crossings that survive routing come from. Both are costs in the router's per-edge shortest-path search; the cost order stays crossings >> turns >> length, with the two new terms priced well below a crossing (backward) or below a turn (sharing).

### Part 1: flow direction in routing (implemented)
- Every step that moves against the flow (towards smaller x) on a data edge is charged half a crossing. Non-functional (nf) edges are exempt: they leave at the bottom and are not part of the story flow.
- Magnitude, decided by measurement: many backward steps (a long detour round the diagram, 6+ steps in phase3-compact) must lose against a route with a crossing or two, while one short hop back must still beat a crossing. At one crossing per step or more, phase2 and azure-phase2 got a crossing (the IAM role -> data bucket edge cut the SMTP edge instead of hopping back one step). At 0.25 to 0.9 of a crossing they stay at 0 and phase3-compact still loses its ring. Half a crossing sits in the middle of that range. The cost is finite, so an edge whose target really lies to the left remains routable.
- Rejected: two crossings per step (first attempt; broke the phase2 reference diagrams); a hard ban on leftward steps (an edge whose target is left of its source would have no route); a per-pixel penalty (needs tuning against the crossing cost, and the problem is the number of backward steps, not their length); cutting the group-intrusion cost (crossing a group that holds neither end should stay expensive; the issue is direction, not groups).

### Part 2: routes that share lanes (implemented as a shared-lane cost)
- Original idea: make the router's crossing count equal the final check's. Measured on phase3-compact (45 crossing pairs in the final layout): the router already prices the 2 crossings that are certain from the lane routes alone (straight-through junctions, straight shots). The other 43 pairs share at least one lane; whether they cross depends on the track order inside the shared lane, which is assigned after routing. So equal counts are not reachable by adding a term; the router cannot know the track order.
- Of the 43, 21 cross whatever the track order (the two routes enter and leave a shared stretch in flipped order: forced); 22 can be uncrossed when the pair is considered alone but cross in the final layout because of the other edges on the same lanes. The track assignment is not the weak part: more tracks, a larger exhaustive search and simulated annealing over all tracks did not get below the floor it already reaches. The crossings are structural and the remedy is in routing.
- Adopted: each step along a lane is charged a small cost per other route's segment already occupying it, so routes prefer free lanes. Priced below a turn so it only breaks ties between routes of similar length; any value from 5 to 200 gave the same result, 60 chosen. Tracks untouched.
- Effect (all cases, cost 60): phase3-compact crossings 45 -> 31 with the stronger backward cost, 35 with the final half-crossing one (48 originally), length +2%, leftward nodes unchanged. rebob-container 24 -> 20 crossings (area 72 -> 81). rebob-system 1 -> 3 crossings but a smaller, shorter layout (area 24 -> 20, length 231 -> 185; the placement search picks a different layout). star16 unchanged crossings, one nf edge loops wider (+30 length). aws and groups unchanged in metrics.
- Rejected: modelling track order inside the router (a redesign of routing plus tracks); building exact order-flip detection on shared stretches now (it would only address the 21 forced pairs and needs maximal common stretches with entry/exit sides per route; kept as a possible later step).
- Accepted regression: rebob-system gets 2 more crossings in exchange for a more compact layout; reconsidering would mean fixing the order in which the placement search judges layouts (crossings, then length and area).

### Verification
- phase3-compact regression case (graph and hints as clarc composes them): the bg -> content polyline never runs left of its start and the diagram has at most 35 crossings. Fails without the backward cost (93.5 units left of its start).
- Reference architectures (clarc): phase1, phase2 and azure-phase2 render with no crossings. Fails when the backward cost is a full crossing per step or more.
- The golden layouts of the engine's cases (aws, star16, rebob-system, rebob-container) are regenerated for the new costs; the whole test suite and the release build are green.

### Remaining
- The c43-layout branch has to be pushed; then bump the c43-layout rev in clarc, commit clarc's regenerated goldens (phase1, phase2) and its reference-architecture test, and re-render. Until then clarc builds against the local checkout and ~/.local/share/clarc/clarc is built from it (already updated).
- Optional later: exact forced-crossing pricing on shared stretches; the placement search's judging order (rebob-system).
