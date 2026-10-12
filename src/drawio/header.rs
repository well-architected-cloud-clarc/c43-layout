//! The title/description band above the grid.
use super::text::{wrap_in, Face, HELVETICA, HELVETICA_BOLD, LINE_H, PAD, TITLE_PX};
use crate::model::{Rect, TextBlock};

/// title/description band above the grid (ends at y = 0)
pub struct Header {
    pub rect: Rect,
    pub title: Option<TextBlock>,
    pub description: Option<TextBlock>,
}

/**
 * title then description, stacked in a band `grid_w` wide (widened only if one word cannot fit);
 * heights come from an estimated line count, the renderer does the actual wrapping; band rounded up to whole units
 */
pub fn header(title: Option<&str>, description: Option<&str>, grid_w: f64, unit_px: f64) -> Option<Header> {
    header_in(title, description, grid_w, unit_px, (&HELVETICA_BOLD, &HELVETICA))
}

/// as `header`, with the faces of the title and of the description given
pub fn header_in(title: Option<&str>, description: Option<&str>, grid_w: f64, unit_px: f64, (title_face, desc_face): (&Face, &Face)) -> Option<Header> {
    if title.is_none() && description.is_none() {
        return None;
    }
    let mut w = grid_w;
    loop {
        let inner = w * unit_px - 2.0 * PAD;
        let lines = |t: Option<&str>, face: &Face| t.map_or(Some(vec![]), |t| wrap_in(t, inner, TITLE_PX, face));
        let (Some(tl), Some(dl)) = (lines(title, title_face), lines(description, desc_face)) else {
            w += 1.0;
            continue;
        };
        let lh = (TITLE_PX * LINE_H) / unit_px;
        let pad = PAD / unit_px;
        let h = ((2.0 * PAD + (tl.len() + dl.len()) as f64 * TITLE_PX * LINE_H) / unit_px).ceil();
        let (x, bw, ty) = (pad, w - 2.0 * pad, -h + pad);
        let dy = ty + tl.len() as f64 * lh;
        let block = |lines: &[String], y: f64| TextBlock { text: lines.join("\n"), x, y, w: bw, h: lines.len() as f64 * lh };
        return Some(Header {
            rect: Rect { x: 0.0, y: -h, w, h },
            title: title.map(|_| block(&tl, ty)),
            description: description.map(|_| block(&dl, dy)),
        });
    }
}
