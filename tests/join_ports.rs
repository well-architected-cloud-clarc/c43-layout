use c43_layout::model::{JoinPorts, Layout, Side};
use c43_layout::{layout, Hints, InputGraph};
use serde_json::json;
use std::collections::HashSet;

/// hub H: 4 sources s0..s3 in, 3 targets t0..t2 out (all leave on the right)
fn run_full(join: JoinPorts) -> c43_layout::model::LayoutResult {
    let ids = |p: &str, n: usize| (0..n).map(|i| json!({ "id": format!("{p}{i}") })).collect::<Vec<_>>();
    let nodes = [ids("s", 4), vec![json!({"id":"H"})], ids("t", 3)].concat();
    let edges = [
        (0..4).map(|i| json!({"from": format!("s{i}"), "to": "H"})).collect::<Vec<_>>(),
        (0..3).map(|i| json!({"from": "H", "to": format!("t{i}")})).collect::<Vec<_>>(),
    ]
    .concat();
    let g: InputGraph = serde_json::from_value(json!({ "nodes": nodes, "edges": edges })).unwrap();
    layout(&g, &Hints { join_ports: join, ..Default::default() }, &Default::default()).unwrap()
}

fn run(join: JoinPorts) -> Layout {
    run_full(join).layout
}

/// distinct attachment points on H of the edges ending (`inbound`) or starting there
fn points(l: &Layout, inbound: bool) -> HashSet<String> {
    l.edges
        .iter()
        .filter(|e| if inbound { e.to == "H" } else { e.from == "H" })
        .map(|e| if inbound { &e.dst } else { &e.src })
        .map(|p| format!("{:?}@{:.3},{:.3}", p.side, p.x, p.y))
        .collect()
}

#[test]
fn off_by_default_keeps_every_port() {
    let l = run(JoinPorts::default());
    assert_eq!(points(&l, true).len(), 4);
    assert_eq!(points(&l, false).len(), 3);
    assert!(l.edges.iter().all(|e| !e.src.join && !e.dst.join));
}

#[test]
fn crowded_inbound_joins_only_inbound() {
    let l = run(JoinPorts { inbound: true, ..Default::default() });
    assert_eq!(points(&l, true).len(), 1);
    assert_eq!(points(&l, false).len(), 3);
}

#[test]
fn crowded_outbound_joins_only_outbound() {
    let l = run(JoinPorts { outbound: true, ..Default::default() });
    assert_eq!(points(&l, true).len(), 4);
    assert_eq!(points(&l, false).len(), 1);
}

#[test]
fn all_joins_both_directions() {
    let l = run(JoinPorts { all: true, ..Default::default() });
    assert_eq!(points(&l, true).len(), 1);
    assert_eq!(points(&l, false).len(), 1);
}

#[test]
fn joined_slot_is_in_the_middle_of_its_side() {
    let l = run(JoinPorts { inbound: true, ..Default::default() });
    let h = l.nodes.iter().find(|n| n.id == "H").unwrap();
    let e = l.edges.iter().find(|e| e.to == "H").unwrap();
    assert_eq!(e.dst.side, Side::Left);
    assert!(e.dst.join && e.dst.count == 2 || e.dst.count == 1);
    assert!(e.dst.y > h.y && e.dst.y < h.y + h.size);
}

#[test]
fn up_to_two_ports_are_not_crowded() {
    let g: InputGraph = serde_json::from_value(json!({
        "nodes": [{"id":"a"},{"id":"b"},{"id":"H"}],
        "edges": [{"from":"a","to":"H"},{"from":"b","to":"H"}]
    }))
    .unwrap();
    let l = layout(&g, &Hints { join_ports: JoinPorts { inbound: true, outbound: true, ..Default::default() }, ..Default::default() }, &Default::default()).unwrap().layout;
    assert!(l.edges.iter().all(|e| !e.dst.join));
}

#[test]
fn joining_leaves_no_rule_violation() {
    let r = run_full(JoinPorts { all: true, ..Default::default() });
    assert!(r.metrics.violations.is_empty(), "{:?}", r.metrics.violations);
}
