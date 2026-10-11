use c43_layout::{layout, Hints, InputGraph};
use serde_json::json;

fn run(title_bottom: bool, title_height: f64) -> c43_layout::model::Layout {
    let g: InputGraph = serde_json::from_value(json!({
        "groups":[{"id":"G","label":"Group"}],
        "nodes":[{"id":"u"},{"id":"a","group":"G"},{"id":"b","group":"G"}],
        "edges":[{"from":"u","to":"a"},{"from":"a","to":"b"}]
    }))
    .unwrap();
    let mut h = Hints::default();
    h.sizes.title_height = title_height;
    h.sizes.title_bottom = title_bottom;
    layout(&g, &h, &Default::default()).unwrap().layout
}

#[test]
fn title_band_at_the_bottom_sits_inside_the_bottom_border_below_the_members() {
    let l = run(true, 2.0);
    let g = &l.groups[0];
    let t = &g.title;
    assert!(t.y + t.h <= g.y + g.h, "title ends inside the group");
    assert!(t.y > g.y + g.h / 2.0, "title is in the bottom half");
    l.nodes.iter().filter(|n| n.group.as_deref() == Some("G")).for_each(|n| assert!(n.y + n.size <= t.y, "{} clear of the title", n.id));
}

#[test]
fn title_band_at_the_top_is_the_default() {
    let l = run(false, 1.0);
    let g = &l.groups[0];
    assert!(g.title.y < g.y + 1.0);
    l.nodes.iter().filter(|n| n.group.as_deref() == Some("G")).for_each(|n| assert!(n.y >= g.title.y + g.title.h, "{} below the title", n.id));
}

#[test]
fn bottom_band_leaves_the_top_gap_of_the_default() {
    // the room for the title moves from the top border to the bottom border, the group is not taller than with a top band
    let (top, bottom) = (run(false, 2.0), run(true, 2.0));
    assert_eq!(top.groups[0].h, bottom.groups[0].h);
}
