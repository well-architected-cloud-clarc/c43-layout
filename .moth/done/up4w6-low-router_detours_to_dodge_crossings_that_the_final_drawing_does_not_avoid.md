Some edges take many legs to avoid a crossing that the detour does not avoid. Found on ghost/phase3/phase3.compact.clarc.json: the API gateway -> reader lambda group edge leaves the gateway, drops into the gap above the reader group, runs left, goes down the left side, runs right under the group and comes up into the reader lambda (6 legs). The straight route (out, down, in) has 3 legs.

Cause: the route to the readers must cross the CloudFront -> content bucket edge, which runs along the lane above the reader group. The router counts a crossing at a junction only when another route's segment passes strictly through it. That segment starts exactly in the lane where the detour goes down, so the router sees the detour as crossing-free (about 60 000 cost) and the straight drop as one crossing (1 000 000). After tracks are assigned the detour's vertical still lies inside the other edge's run, so it crosses it anyway, and carries the extra legs. The router's crossing model and the final check disagree where a segment ends in a lane another route passes through.

Wanted: no detour whose only gain is a crossing that the final drawing does not avoid. Two options, to be measured (not exclusive):
1. Price a junction where another route's segment ends in the same lane at about half a crossing (it crosses or not depending on track order) instead of zero.
2. An after-pass over the routed edges: for each edge, try the cheaper candidate (fewer legs/shorter) with the real track order and polylines and keep it when the check's crossings do not increase (and the legs decrease). Handles every disagreement between router and check, not just this one, at the price of extra builds.

Measure either on the goldens (aws, star16, rebob-system, rebob-container, groups) and the phase3 compact snapshot (crossings now 26 with the newest hints); the straightened phase3 edge must come out with at most 3 legs. Related: ke8fg (backward cost, shared-lane cost), whose notes list this as the unreachable-equal-counts remainder.

----- AI agent updates -------

## Specification (clarified)

Goal: no route keeps extra legs whose only gain is a crossing the final drawing does not avoid. The router prices crossings from lane segments alone; where another route's segment ends in a lane this route passes, only the track order (assigned after routing) decides, so router and final check disagree. Fixed by option 2, an after-pass using the real tracks and polylines.

### Straightening pass (implemented)
- Runs only on the final build (not during the placement/port search, which builds hundreds of times), after routing, before the layout is reported.
- For each routed edge, most legs first: the candidate is that edge routed alone on an empty diagram (no other route to dodge; group-intrusion and flow costs still apply). If the candidate has fewer legs than the routed one, build the whole layout with it (tracks, polylines, check). Keep it when violations and crossings do not increase; otherwise keep the routed one. Accepted candidates are the base for the next edge.
- Cost: one extra full build per edge whose lone route is shorter in legs than its routed one; edges already minimal cost nothing.

### Decisions
- Option 1 (price a junction where another segment ends in the crossed lane at half a crossing) was tried on phase3-compact and rejected: the detour stayed at 7 legs and crossings rose 26 -> 40.
- Rejected: running the pass inside the search builds (too slow); accepting on legs alone (could add crossings).

### Verification
- phase3-compact: apigw -> readers 7 legs -> 3, crossings 26 -> 24. Regression test: that edge has at most 3 legs and the diagram at most 26 crossings (fails before). Goldens (aws, star16, rebob-system, rebob-container, groups) unchanged; whole suite and release build green.

### Remaining
- Done: branch merged to main and pushed; clarc's c43-layout rev points at the merge commit, clarc's goldens and tests are green with it, ~/.local/share/clarc/clarc is built from it, and ghost/phase3/phase3.compact.{drawio,png} are re-rendered (the straightened edge is out, down, in).
- Optional later: the pass only tries the edge's lone route as the cheaper candidate; other candidates (e.g. routes with the other edges' obstacles but a different cost weighting) are not tried.
