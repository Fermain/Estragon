//! Estragon layout core.
//!
//! Godot lays out one `Container` at a time: each container positions its direct
//! children inside its own rect. This crate exposes exactly that operation
//! ([`layout_container`]) plus intrinsic sizing ([`measure_container`]), and the
//! web studio walks its tree the same way. Same engine, same algorithm, same
//! numbers on both sides.

pub mod schema;

use schema::{Align, NodeStyle};
use serde::Serialize;
use taffy::prelude::*;
use taffy::{
    GridPlacement, GridTemplateArea, GridTemplateAreas, GridTemplateComponent, GridTemplateTracks, GridAutoTracks,
    Line,
};

pub use taffy;

#[derive(Debug)]
pub enum LayoutError {
    Parse { field: &'static str, value: String, message: String },
    Areas(String),
    Taffy(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { field, value, message } => write!(f, "invalid `{field}` value {value:?}: {message}"),
            Self::Areas(m) => write!(f, "invalid grid areas: {m}"),
            Self::Taffy(m) => write!(f, "layout failed: {m}"),
        }
    }
}
impl std::error::Error for LayoutError {}

impl From<taffy::TaffyError> for LayoutError {
    fn from(e: taffy::TaffyError) -> Self {
        Self::Taffy(e.to_string())
    }
}

/// One child of a container, as seen by the layout pass.
#[derive(Debug, Clone)]
pub struct Item {
    pub style: NodeStyle,
    /// The child's own minimum size (Godot: `get_combined_minimum_size()`).
    /// Godot clamps a Control's size to this, so Taffy must never go below it.
    pub min_size: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ItemRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Lay out `items` inside a container of exactly `size`. Returns one rect per
/// item, relative to the container's top-left, in input order.
pub fn layout_container(container: &NodeStyle, items: &[Item], size: (f32, f32)) -> Result<Vec<ItemRect>, LayoutError> {
    let (mut tree, root, children) = build(container, items)?;
    let mut root_style = tree.style(root)?.clone();
    root_style.size = Size { width: length(size.0), height: length(size.1) };
    tree.set_style(root, root_style)?;
    compute(&mut tree, root, Size { width: AvailableSpace::Definite(size.0), height: AvailableSpace::Definite(size.1) })?;
    children
        .iter()
        .map(|&id| {
            let l = tree.layout(id)?;
            Ok(ItemRect { x: l.location.x, y: l.location.y, width: l.size.width, height: l.size.height })
        })
        .collect()
}

/// The container's min-content size (Godot: `_get_minimum_size()`).
pub fn measure_container(container: &NodeStyle, items: &[Item]) -> Result<(f32, f32), LayoutError> {
    let (mut tree, root, _) = build(container, items)?;
    compute(&mut tree, root, Size::MIN_CONTENT)?;
    let l = tree.layout(root)?;
    Ok((l.size.width, l.size.height))
}

/// A node's final rect in document (viewport) coordinates.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Placed {
    pub id: String,
    pub depth: u32,
    #[serde(flatten)]
    pub rect: ItemRect,
}

/// Lay out a whole document by walking it container-by-container, exactly as
/// Godot's Container system does. The root fills the document viewport.
pub fn layout_document(doc: &schema::Document) -> Result<Vec<Placed>, LayoutError> {
    let mut out = Vec::new();
    let root = ItemRect { x: 0.0, y: 0.0, width: doc.width, height: doc.height };
    place(&doc.root, root, 0, &mut out)?;
    Ok(out)
}

fn place(node: &schema::Node, rect: ItemRect, depth: u32, out: &mut Vec<Placed>) -> Result<(), LayoutError> {
    out.push(Placed { id: node.id.clone(), depth, rect });
    if node.kind != schema::Kind::Grid || node.children.is_empty() {
        return Ok(());
    }
    let items = node
        .children
        .iter()
        .map(|c| Ok(Item { style: c.style.clone(), min_size: min_size_of(c)? }))
        .collect::<Result<Vec<_>, LayoutError>>()?;
    let rects = layout_container(&node.style, &items, (rect.width, rect.height))?;
    for (child, r) in node.children.iter().zip(rects) {
        let abs = ItemRect { x: rect.x + r.x, y: rect.y + r.y, ..r };
        place(child, abs, depth + 1, out)?;
    }
    Ok(())
}

/// Equivalent of Godot's `get_combined_minimum_size()` for a schema node.
pub fn min_size_of(node: &schema::Node) -> Result<(f32, f32), LayoutError> {
    let own = (node.style.min_width.unwrap_or(0.0), node.style.min_height.unwrap_or(0.0));
    if node.kind != schema::Kind::Grid {
        return Ok(own);
    }
    let items = node
        .children
        .iter()
        .map(|c| Ok(Item { style: c.style.clone(), min_size: min_size_of(c)? }))
        .collect::<Result<Vec<_>, LayoutError>>()?;
    let m = measure_container(&node.style, &items)?;
    Ok((own.0.max(m.0), own.1.max(m.1)))
}

type Tree = TaffyTree<(f32, f32)>;

fn build(container: &NodeStyle, items: &[Item]) -> Result<(Tree, NodeId, Vec<NodeId>), LayoutError> {
    let mut tree: Tree = TaffyTree::new();
    let children = items
        .iter()
        .map(|item| Ok(tree.new_leaf_with_context(item_style(&item.style, item.min_size)?, item.min_size)?))
        .collect::<Result<Vec<_>, LayoutError>>()?;
    let root = tree.new_with_children(container_style(container)?, &children)?;
    Ok((tree, root, children))
}

fn compute(tree: &mut Tree, root: NodeId, available: Size<AvailableSpace>) -> Result<(), LayoutError> {
    tree.compute_layout_with_measure(root, available, |inputs, _id, ctx, style| {
        let min = ctx.map(|c| *c).unwrap_or((0.0, 0.0));
        taffy::compute_leaf_layout(inputs, style, |_, _| 0.0, |known, _available| Size {
            width: known.width.unwrap_or(min.0),
            height: known.height.unwrap_or(min.1),
        })
    })?;
    Ok(())
}

/// Translate a container's [`NodeStyle`] into a Taffy grid style.
pub fn container_style(s: &NodeStyle) -> Result<Style, LayoutError> {
    let mut style = Style { display: Display::Grid, ..Default::default() };

    if let Some(cols) = &s.columns {
        let t = parse_tracks("columns", cols)?;
        style.grid_template_columns = t.tracks;
        style.grid_template_column_names = t.line_names;
    }
    if let Some(rows) = &s.rows {
        let t = parse_tracks("rows", rows)?;
        style.grid_template_rows = t.tracks;
        style.grid_template_row_names = t.line_names;
    }
    if let Some(v) = &s.auto_rows {
        style.grid_auto_rows = parse::<GridAutoTracks>("autoRows", v)?.0;
    }
    if let Some(v) = &s.auto_columns {
        style.grid_auto_columns = parse::<GridAutoTracks>("autoColumns", v)?.0;
    }
    if !s.areas.is_empty() {
        style.grid_template_areas = Some(parse_areas(&s.areas)?);
    }
    style.gap = Size {
        width: length(s.column_gap.unwrap_or(s.gap)),
        height: length(s.row_gap.unwrap_or(s.gap)),
    };
    let p = length(s.padding);
    style.padding = Rect { left: p, right: p, top: p, bottom: p };
    Ok(style)
}

/// Translate an item's [`NodeStyle`] into a Taffy leaf style.
pub fn item_style(s: &NodeStyle, measured_min: (f32, f32)) -> Result<Style, LayoutError> {
    let mut style = Style::default();
    if let Some(area) = &s.area {
        // CSS `grid-area: foo` == row/column `foo / foo`, resolved via the implicit
        // `foo-start` / `foo-end` line names that `grid-template-areas` creates.
        let named = || GridPlacement::NamedLine(area.clone(), 0);
        style.grid_row = Line { start: named(), end: named() };
        style.grid_column = Line { start: named(), end: named() };
    }
    if let Some(v) = &s.column {
        style.grid_column = parse_line("column", v)?;
    }
    if let Some(v) = &s.row {
        style.grid_row = parse_line("row", v)?;
    }
    let min_w = s.min_width.unwrap_or(0.0).max(measured_min.0);
    let min_h = s.min_height.unwrap_or(0.0).max(measured_min.1);
    style.min_size = Size { width: length(min_w), height: length(min_h) };
    style.justify_self = s.justify_self.map(align);
    style.align_self = s.align_self.map(align);
    Ok(style)
}

fn align(a: Align) -> AlignSelf {
    match a {
        Align::Start => AlignSelf::START,
        Align::End => AlignSelf::END,
        Align::Center => AlignSelf::CENTER,
        Align::Stretch => AlignSelf::STRETCH,
    }
}

fn parse<T: std::str::FromStr>(field: &'static str, value: &str) -> Result<T, LayoutError>
where
    T::Err: std::fmt::Debug,
{
    value.parse::<T>().map_err(|e| LayoutError::Parse { field, value: value.into(), message: format!("{e:?}") })
}

fn parse_tracks(field: &'static str, v: &str) -> Result<GridTemplateTracks<String, GridTemplateComponent<String>>, LayoutError> {
    parse(field, v)
}

/// `"1 / 3"`, `"2"`, `"span 2"`, `"1 / span 2"`.
fn parse_line(field: &'static str, v: &str) -> Result<Line<GridPlacement<String>>, LayoutError> {
    let mut parts = v.splitn(2, '/');
    let start: GridPlacement<String> = parse(field, parts.next().unwrap_or("").trim())?;
    let end = match parts.next() {
        Some(e) => parse(field, e.trim())?,
        None => GridPlacement::Auto,
    };
    Ok(Line { start, end })
}

/// Parse `grid-template-areas` rows into Taffy's area list (1-indexed lines).
fn parse_areas(rows: &[String]) -> Result<GridTemplateAreas<String>, LayoutError> {
    let grid: Vec<Vec<&str>> = rows.iter().map(|r| r.split_whitespace().collect()).collect();
    let cols = grid[0].len();
    if grid.iter().any(|r| r.len() != cols) {
        return Err(LayoutError::Areas("every row must have the same number of cells".into()));
    }
    let mut areas: Vec<GridTemplateArea<String>> = Vec::new();
    for (r, row) in grid.iter().enumerate() {
        for (c, &name) in row.iter().enumerate() {
            if name.chars().all(|ch| ch == '.') {
                continue;
            }
            let (r, c) = (r as u16 + 1, c as u16 + 1);
            match areas.iter_mut().find(|a| a.name == name) {
                Some(a) => {
                    a.row_start = a.row_start.min(r);
                    a.row_end = a.row_end.max(r + 1);
                    a.column_start = a.column_start.min(c);
                    a.column_end = a.column_end.max(c + 1);
                }
                None => areas.push(GridTemplateArea {
                    name: name.to_string(),
                    row_start: r,
                    row_end: r + 1,
                    column_start: c,
                    column_end: c + 1,
                }),
            }
        }
    }
    // Areas must be rectangles: every cell in the bounding box must carry the name.
    for a in &areas {
        for r in a.row_start..a.row_end {
            for c in a.column_start..a.column_end {
                if grid[(r - 1) as usize][(c - 1) as usize] != a.name {
                    return Err(LayoutError::Areas(format!("area `{}` is not a rectangle", a.name)));
                }
            }
        }
    }
    Ok(GridTemplateAreas { areas, row_count: grid.len() as u16, column_count: cols as u16 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(style: NodeStyle) -> Item {
        Item { style, min_size: (0.0, 0.0) }
    }

    #[test]
    fn hud_areas() {
        let container = NodeStyle {
            columns: Some("200px 1fr 200px".into()),
            rows: Some("100px 1fr 80px".into()),
            areas: vec!["map . hp".into(), ". . .".into(), "chat bar bar".into()],
            gap: 10.0,
            padding: 20.0,
            ..Default::default()
        };
        let items = ["map", "hp", "chat", "bar"]
            .map(|a| item(NodeStyle { area: Some(a.into()), ..Default::default() }));
        let r = layout_container(&container, &items, (1280.0, 720.0)).unwrap();
        // 1280 - 2*20 padding - 2*10 gap - 400 fixed = 820 for the 1fr column
        assert_eq!(r[0], ItemRect { x: 20.0, y: 20.0, width: 200.0, height: 100.0 });
        assert_eq!(r[1], ItemRect { x: 1060.0, y: 20.0, width: 200.0, height: 100.0 });
        assert_eq!(r[2], ItemRect { x: 20.0, y: 620.0, width: 200.0, height: 80.0 });
        assert_eq!(r[3], ItemRect { x: 230.0, y: 620.0, width: 1030.0, height: 80.0 });
    }

    #[test]
    fn min_size_is_respected() {
        let container = NodeStyle { columns: Some("auto 1fr".into()), ..Default::default() };
        let items = [Item { style: NodeStyle::default(), min_size: (120.0, 40.0) }, item(NodeStyle::default())];
        let r = layout_container(&container, &items, (400.0, 100.0)).unwrap();
        assert_eq!(r[0].width, 120.0);
        assert_eq!(r[1].x, 120.0);
        assert_eq!(measure_container(&container, &items).unwrap(), (120.0, 40.0));
    }

    #[test]
    fn rejects_non_rectangular_area() {
        let s = NodeStyle { areas: vec!["a a".into(), "a b".into()], ..Default::default() };
        assert!(matches!(container_style(&s), Err(LayoutError::Areas(_))));
    }
}
