use c43_layout::drawio::export::{Font, Fonts, Styler};
use c43_layout::drawio::render_fonts;
use c43_layout::drawio::text::HELVETICA;
use c43_layout::model::{Layout, Lanes, Rect};

fn empty() -> Layout {
    Layout { s: 0.0, cols: 0, rows: 0, lanes: Lanes { v: vec![], h: vec![] }, groups: vec![], nodes: vec![], edges: vec![], frame: Rect { x: 0.0, y: 0.0, w: 20.0, h: 10.0 }, title: None, description: None }
}

fn styler() -> Styler<'static> {
    Styler { node: Box::new(|_, _, _| vec![]), group: Box::new(|_, _| vec![]), edge: Box::new(|_| unreachable!()), label_font: 12.0 }
}

fn fonts() -> Fonts<'static> {
    Fonts {
        regular: Font { family: "Reg", source: "data%3Areg", face: HELVETICA },
        name: Font { family: "Name", source: "data%3Aname", face: HELVETICA },
    }
}

fn cell<'a>(xml: &'a str, id: &str) -> &'a str {
    xml.lines().find(|l| l.contains(&format!("<mxCell id=\"{id}\""))).unwrap()
}

#[test]
fn fonts_are_declared_once_per_family_and_set_the_header_text() {
    let xml = render_fonts(&empty(), Some("Title"), Some("About"), styler(), "", Some(&fonts()));
    assert_eq!(xml.matches("fontSource=data%3Areg;").count(), 1);
    assert_eq!(xml.matches("fontSource=data%3Aname;").count(), 1);
    assert!(cell(&xml, "title").contains("fontFamily=Name;") && !cell(&xml, "title").contains("fontStyle"));
    assert!(cell(&xml, "description").contains("fontFamily=Reg;") && !cell(&xml, "description").contains("fontStyle"));
}

#[test]
fn no_fonts_keeps_the_default_look_of_the_header() {
    let xml = render_fonts(&empty(), Some("Title"), Some("About"), styler(), "", None);
    assert!(!xml.contains("fontFamily") && !xml.contains("fontSource"));
    assert!(cell(&xml, "title").contains("fontStyle=1;"));
}
