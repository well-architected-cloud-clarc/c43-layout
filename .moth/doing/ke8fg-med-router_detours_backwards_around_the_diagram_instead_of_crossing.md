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

Goal: a data edge is never routed against the left-to-right flow just to dodge crossings; the router's idea of "crossing-free" must match what the final layout check counts.

### Part 1: flow direction in routing (implemented)
- Routing cost stays lexicographic (crossings >> turns >> length). A new term charges every step that moves against the flow (towards smaller x) on a data edge, worth two crossings per step. Non-functional (nf) edges are exempt: they leave at the bottom and are not part of the story flow.
- Rationale for the magnitude: a backward step must lose against any route that needs at most a couple of crossings, so a long detour round the diagram cannot win on length-free "zero crossings"; it is still finite, so an edge whose target really lies to the left remains routable.
- Rejected: a hard ban on leftward steps (breaks edges whose target is left of the source, no route possible); a per-pixel penalty (detour length is otherwise free, and a per-pixel scale would have to be tuned against CROSS).
- Rejected: cutting the group-intrusion cost. Crossing a group that holds neither end stays expensive; the fix is about direction, not about groups.

### Part 2: router crossing model vs. the final check (measured; open)
- Measurement on phase3-compact (final layout, 45 crossing pairs): the router's model (strict perpendicular crossings of the lane routes, which is what it prices) predicts only 2 of the 45. The other 43 are pairs of routes that share at least one lane; in the lane model they never cross strictly, the crossing arises from the track order inside the shared lane (or a touch at a shared-arm junction). So the undercount is not a bug in the cheap cases (straight-through junctions, straight shots, stubs are all counted); it is the whole class of crossings decided after routing by track assignment.
- What the router could compute without knowing tracks: for two routes sharing a contiguous lane stretch, the crossing is forced when the order of the two routes flips between where they enter and where they leave the stretch (entry sides opposite, exit sides the same, or vice versa). This is track-independent and could be priced like a crossing. Not built; sized as a separate piece of work (maximal common stretches across lane steps, entry/exit sides per route).
- Second measurement (same diagram): for each of the 43 shared-lane crossing pairs, both routes alone were tried with every track order on their shared lanes. 21 pairs cross whatever the order (forced: a router that prices shared-stretch order flips would see them); 22 pairs can be uncrossed in isolation but still cross in the final layout (the track assignment, which weighs all edges at once, lost them: a different fix, in tracks, not in routing). Forced pairs by number of shared lanes: 1 lane 5 of 17, 2 lanes 11 of 16, 3 lanes 5 of 6, 4 lanes 0 of 4. The two pairs with no shared lane are the ones the router already prices.
- The 22 "avoidable in isolation" pairs are not lost by a weak track search. Tried on the same diagram: (a) allowing one or two more tracks than the greedy range used changed the final crossing count 45 -> 43 -> 44 (noise); (b) raising the exhaustive per-lane limit from 243 to 2000/6000 left it at 45 and only made the run slower; (c) all 22 share lanes within one band, so lane bands are not the cause; (d) simulated annealing over every track (4000 moves, on 208 search states) improved only 19 states, by 2.6 on average, and never went below the floor the improver already reaches. Conclusion: given the routes, the track assignment is close to optimal; the crossings are structural, forced by how the other edges occupy the same lanes (the isolation test left them out). The remedy is in routing (price shared-lane order flips, or route with knowledge of lane occupancy), not in tracks.rs.
- Decision: do not model track order inside the router. Open question whether to build the forced-crossing-on-shared-stretch pricing.
- The router counts only straight shots cutting a lane and straight-through junctions. The final check also counts crossings made where edges turn at junctions or share lanes. The router therefore believes some routes (the outer ring) are crossing-free when they are not. Goal: the router's per-edge crossing count equals the check's for the same geometry, so route choice is made on true counts.
- After part 1 the symptom for the observed edge is gone (crossings of that edge 10 -> 3, diagram 48 -> 45 crossing warnings), but the undercount remains and can show up on other edges.

### Verification
- Regression case tests/cases/phase3-compact (the compact phase3 graph and hints as clarc composes them): the bg -> content polyline never runs left of its start. Fails without the backward cost (93.5 units left), passes with it. Whole test suite and release build green.
- Part 2 test, when done: for a case where the ring looks free to the router, the router's crossing count of the chosen route equals the check's count.

### Remaining
- Part 2.
- Bump the c43-layout rev in clarc, rebuild, update ~/.local/share/clarc/clarc, re-render ghost/phase3/phase3.compact.clarc.json.




