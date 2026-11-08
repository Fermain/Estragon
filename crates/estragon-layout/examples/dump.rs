//! Print the layout of a `.ui.json` document as JSON (for parity checks).
//! Usage: cargo run -p estragon-layout --example dump -- <file.ui.json>
fn main() {
    let path = std::env::args().nth(1).expect("usage: dump <file.ui.json>");
    let text = std::fs::read_to_string(&path).expect("read document");
    let doc: estragon_layout::schema::Document = serde_json::from_str(&text).expect("parse document");
    let placed = estragon_layout::layout_document(&doc).expect("layout");
    println!("{}", serde_json::to_string_pretty(&placed).unwrap());
}
