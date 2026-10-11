//! drawio emission of a laid-out diagram: text measuring, header band, exporter with a caller-supplied look.
pub mod export;
pub mod header;
pub mod text;

use crate::model::{Layout, Rect};
use export::{to_drawio, Drawing, Styler};
use header::header;
use text::{label_lines, TITLE_PX, UNIT_PX};

/// as the plain layout export, with the node, group and edge drawing given; label lines, header band and frame are added here, at 20 px per grid unit
pub fn render_styled(l: &Layout, title: Option<&str>, description: Option<&str>, style: Styler) -> String {
    render_with(l, title, description, style, "rounded=0;fillColor=none;")
}

/// as `render_styled`, with the style of the outer diagram border given
pub fn render_with(l: &Layout, title: Option<&str>, description: Option<&str>, style: Styler, frame_style: &str) -> String {
    let head = header(title, description, l.frame.w, UNIT_PX);
    let frame = head.as_ref().map_or(l.frame, |h| Rect { x: 0.0, y: h.rect.y, w: h.rect.w, h: h.rect.h + l.frame.h });
    let node_lines = l.nodes.iter().map(|n| label_lines(&n.label, n.size * UNIT_PX).unwrap_or_else(|| vec![n.label.clone()])).collect();
    to_drawio(&Drawing { layout: l, unit_px: UNIT_PX, node_lines, header: head, frame, style, title_font: TITLE_PX, frame_style })
}
