use crate::geom::{box_of, cell_key, slot_offset, EndInfo};
use crate::groups::is_inner;
use crate::model::{Box as GBox, End, Graph, Kind, Placement, PortRef, Ports, Side, SidePorts, SIDES};
use indexmap::IndexSet;
use std::cmp::Ordering;

/**
 * exit: right, or bottom for a target below that is not to the right / for nf edges to targets below;
 * entry: left (preferred), or top when the target sits directly below or below-left of the source.
 * Groups are boxes: "below" means entirely below, "not to the right" means starting at or left of the source.
 */
pub fn choose_sides(kind: Kind, s: &GBox, t: &GBox) -> (Side, Side) {
    let below = t.row0 > s.row1;
    let not_right = t.col0 <= s.col0;
    if below && not_right {
        return (Side::Bottom, Side::Top);
    }
    if below && kind == Kind::Nf {
        return (Side::Bottom, Side::Left);
    }
    (Side::Right, Side::Left)
}

/// IDEA: fewer than 4 same-kind outs all leave on the right
const MAX_RIGHT_OUTS: usize = 3;

/// the far end's cell (col, row); a group counts as the centre of its block
pub fn far_cell(g: &Graph, p: &Placement, r: &PortRef) -> (f64, f64) {
    let e = &g.edges[r.edge];
    let b = box_of(p, if r.end == End::Src { &e.to } else { &e.from });
    ((b.col0 + b.col1) as f64 / 2.0, (b.row0 + b.row1) as f64 / 2.0)
}

/**
 * order slots by the far end's position to pre-empt crossings: the lower the far end, the lower
 * (further clockwise) the port. On a tie the far end that needs the longer detour goes outside.
 */
pub fn sort_sides(g: &Graph, p: &Placement, s: &mut SidePorts) {
    let cmp = |key: fn((f64, f64)) -> (f64, f64)| {
        move |a: &PortRef, b: &PortRef| {
            let (a0, a1) = key(far_cell(g, p, a));
            let (b0, b1) = key(far_cell(g, p, b));
            a0.partial_cmp(&b0).unwrap().then(a1.partial_cmp(&b1).unwrap()).then(a.edge.cmp(&b.edge))
        }
    };
    s.right.sort_by(cmp(|(c, r)| (r, -c))); // top→bottom; same row: farther target higher
    s.left.sort_by(cmp(|(c, r)| (r, c))); // top→bottom; same row: farther source higher
    s.top.sort_by(cmp(|(c, r)| (c, r))); // left→right; same column: lower target further right
    s.bottom.sort_by(cmp(|(c, r)| (c, -r))); // left→right; same column: lower target further left
}

pub fn assign_ports(g: &Graph, p: &Placement) -> Ports {
    let mut ports: Ports = g.nodes.iter().map(|n| &n.id).chain(g.groups.iter().map(|x| &x.id)).map(|id| (id.clone(), SidePorts::default())).collect();
    for e in &g.edges {
        // a group pointing at its own member starts inside its left border, flowing right
        let inner = is_inner(g, &e.from, &e.to);
        let (ss, ts) = if inner { (Side::Left, Side::Left) } else { choose_sides(e.kind, &box_of(p, &e.from), &box_of(p, &e.to)) };
        ports.get_mut(&e.from).unwrap().get_mut(ss).push(PortRef { edge: e.id, end: End::Src, inner, align: None });
        ports.get_mut(&e.to).unwrap().get_mut(ts).push(PortRef { edge: e.id, end: End::Dst, inner: false, align: None });
    }
    // 4+ data outs on the right: keep 3 there; the overflow peels off whichever end of the clockwise
    // order lies farther from the node's row (tie: the lower end) — targets below leave from the bottom,
    // targets above from the top, targets level with the node stay right the longest
    for n in &g.nodes {
        let row = p.cells[&n.id].row as f64;
        let far = |r: &PortRef| far_cell(g, p, r);
        let s = ports.get_mut(&n.id).unwrap();
        let mut outs: Vec<PortRef> = s.right.iter().filter(|r| r.end == End::Src && g.edges[r.edge].kind == Kind::Data).copied().collect();
        outs.sort_by(|a, b| {
            let (fa, fb) = (far(a), far(b));
            fa.1.partial_cmp(&fb.1).unwrap().then(fb.0.partial_cmp(&fa.0).unwrap_or(Ordering::Equal)).then(a.edge.cmp(&b.edge))
        });
        while outs.len() > MAX_RIGHT_OUTS {
            let hi = (far(&outs[0]).1 - row).abs();
            let lo = (far(outs.last().unwrap()).1 - row).abs();
            let r = if hi > lo { outs.remove(0) } else { outs.pop().unwrap() };
            s.right.retain(|q| *q != r);
            if far(&r).1 < row { s.top.push(r) } else { s.bottom.push(r) }
        }
    }
    for s in ports.values_mut() {
        sort_sides(g, p, s);
    }
    ports
}

pub fn end_info(ports: &Ports, node: &str, edge: usize, end: End) -> EndInfo {
    let s = &ports[node];
    for side in SIDES {
        let list = s.get(side);
        if let Some(index) = list.iter().position(|r| r.edge == edge && r.end == end) {
            return EndInfo { side, index, count: list.len(), inner: list[index].inner, align: list[index].align };
        }
    }
    panic!("no port for edge {edge} ({end:?}) on {node}");
}

/// node side S = most ports on any side of any node; `skip` = ids that are not nodes (groups)
pub fn side_count(ports: &Ports, skip: Option<&IndexSet<String>>) -> usize {
    ports
        .iter()
        .filter(|(id, _)| !skip.is_some_and(|s| s.contains(*id)))
        .flat_map(|(_, s)| SIDES.map(|side| s.get(side).len()))
        .fold(1, usize::max)
}

/// current position of a port on its side, as a fraction of the side's length
fn slot_fraction(list: &[PortRef], i: usize) -> f64 {
    let (a, n) = list[i].align.unwrap_or((i, list.len()));
    slot_offset(1.0, a, n)
}

/// a port may take another position along its side only between its neighbours, half a slot away from each
fn fits_at(list: &[PortRef], i: usize, at: f64) -> bool {
    let gap = 0.5 / list.len() as f64;
    let before = i.checked_sub(1).map_or(0.0, |j| slot_fraction(list, j));
    let after = list.get(i + 1).map_or(1.0, |_| slot_fraction(list, i + 1));
    let (lo, hi) = (if i == 0 { 0.0 } else { before + gap }, if i + 1 == list.len() { 1.0 } else { after - gap });
    lo <= at && at <= hi && 0.0 < at && at < 1.0
}

/**
 * A connection between two nodes in one row with nothing in between would run straight, but only when its
 * ports are at the same height; evenly spread slots of sides with different port counts rarely are, which
 * costs a detour around the row. Such a connection moves one of its ports along its side to the other
 * port's height, when that keeps the order and half a slot to the neighbours. The other ports stay put.
 * The even spread is the rule for every port that was not moved this way.
 */
pub fn snap_ports(g: &Graph, p: &Placement, ports: &Ports) -> Ports {
    let mut ports = ports.clone();
    let taken: IndexSet<String> = p.cells.values().map(cell_key).collect();
    for e in &g.edges {
        let (Some(sc), Some(tc)) = (p.cells.get(&e.from), p.cells.get(&e.to)) else { continue };
        let free = sc.row == tc.row && tc.col > sc.col && !(sc.col + 1..tc.col).any(|c| taken.contains(&cell_key(&crate::model::Cell { col: c, row: sc.row })));
        let at = |side: &[PortRef], end: End| side.iter().position(|r| r.edge == e.id && r.end == end);
        let (Some(i), Some(j)) = (at(&ports[&e.from].right, End::Src), at(&ports[&e.to].left, End::Dst)) else { continue };
        if !free || ports[&e.from].right[i].inner {
            continue;
        }
        let (src, dst) = (slot_fraction(&ports[&e.from].right, i), slot_fraction(&ports[&e.to].left, j));
        if (src - dst).abs() < 1e-12 {
            continue;
        }
        if fits_at(&ports[&e.to].left, j, src) {
            let a = ports[&e.from].right[i].align.unwrap_or((i, ports[&e.from].right.len()));
            ports.get_mut(&e.to).unwrap().left[j].align = Some(a);
        } else if fits_at(&ports[&e.from].right, i, dst) {
            let a = ports[&e.to].left[j].align.unwrap_or((j, ports[&e.to].left.len()));
            ports.get_mut(&e.from).unwrap().right[i].align = Some(a);
        }
    }
    ports
}
