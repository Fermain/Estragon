//! Estragon GDExtension.
//!
//! - `EstragonGrid`: a `Container` that lays out its children with CSS Grid via
//!   Taffy (the same `estragon-layout` crate the web studio runs as WASM).
//! - `Estragon`: a helper singleton-style class that builds a node tree from a
//!   `.ui.json` document.
//!
//! Children stay ordinary Controls, so focus, Theme styling and signals keep working.
//! Per-child grid placement lives in metadata (`estragon_area`, `estragon_column`, …),
//! mirroring how Godot keeps size flags on the child.

use estragon_layout::schema::{Align, Document, Kind, Node, NodeStyle};
use estragon_layout::{Item, layout_container, measure_container};
use godot::classes::notify::ContainerNotification;
use godot::classes::{ColorRect, Container, Control, FileAccess, IContainer, file_access::ModeFlags};
use godot::prelude::*;

struct EstragonExtension;

#[gdextension]
unsafe impl ExtensionLibrary for EstragonExtension {}

/// A CSS Grid container powered by Taffy.
#[derive(GodotClass)]
#[class(tool, init, base = Container)]
pub struct EstragonGrid {
    /// `grid-template-columns`, e.g. `240px 1fr 240px`.
    #[export]
    #[var(set = set_columns)]
    columns: GString,
    /// `grid-template-rows`.
    #[export]
    #[var(set = set_rows)]
    rows: GString,
    /// `grid-template-areas`, one row per entry, e.g. `["map . hp", "chat bar bar"]`.
    #[export]
    #[var(set = set_areas)]
    areas: PackedStringArray,
    /// `grid-auto-rows`, sizing for implicitly-created rows (e.g. `44px`).
    #[export]
    #[var(set = set_auto_rows)]
    auto_rows: GString,
    /// `grid-auto-columns`.
    #[export]
    #[var(set = set_auto_columns)]
    auto_columns: GString,
    #[export]
    #[var(set = set_gap)]
    gap: f32,
    /// Overrides `gap` for rows when >= 0.
    #[export]
    #[var(set = set_row_gap)]
    #[init(val = -1.0)]
    row_gap: f32,
    /// Overrides `gap` for columns when >= 0.
    #[export]
    #[var(set = set_column_gap)]
    #[init(val = -1.0)]
    column_gap: f32,
    #[export]
    #[var(set = set_padding)]
    padding: f32,

    base: Base<Container>,
}

#[godot_api]
impl EstragonGrid {
    #[func]
    fn set_columns(&mut self, v: GString) {
        self.columns = v;
        self.relayout();
    }
    #[func]
    fn set_rows(&mut self, v: GString) {
        self.rows = v;
        self.relayout();
    }
    #[func]
    fn set_areas(&mut self, v: PackedStringArray) {
        self.areas = v;
        self.relayout();
    }
    #[func]
    fn set_auto_rows(&mut self, v: GString) {
        self.auto_rows = v;
        self.relayout();
    }
    #[func]
    fn set_auto_columns(&mut self, v: GString) {
        self.auto_columns = v;
        self.relayout();
    }
    #[func]
    fn set_row_gap(&mut self, v: f32) {
        self.row_gap = v;
        self.relayout();
    }
    #[func]
    fn set_column_gap(&mut self, v: f32) {
        self.column_gap = v;
        self.relayout();
    }
    #[func]
    fn set_gap(&mut self, v: f32) {
        self.gap = v;
        self.relayout();
    }
    #[func]
    fn set_padding(&mut self, v: f32) {
        self.padding = v;
        self.relayout();
    }

    fn relayout(&mut self) {
        self.base_mut().update_minimum_size();
        self.base_mut().queue_sort();
    }

    fn container_style(&self) -> NodeStyle {
        let non_empty = |s: &GString| (!s.is_empty()).then(|| s.to_string());
        NodeStyle {
            columns: non_empty(&self.columns),
            rows: non_empty(&self.rows),
            areas: self.areas.as_slice().iter().map(|s| s.to_string()).collect(),
            auto_rows: non_empty(&self.auto_rows),
            auto_columns: non_empty(&self.auto_columns),
            gap: self.gap,
            row_gap: (self.row_gap >= 0.0).then_some(self.row_gap),
            column_gap: (self.column_gap >= 0.0).then_some(self.column_gap),
            padding: self.padding,
            ..Default::default()
        }
    }

    /// Visible, layout-participating Control children (Godot's own rule for containers).
    fn layout_children(&self) -> Vec<Gd<Control>> {
        self.base()
            .get_children()
            .iter_shared()
            .filter_map(|n| n.try_cast::<Control>().ok())
            .filter(|c| c.is_visible() && !c.is_set_as_top_level())
            .collect()
    }

    fn items(children: &[Gd<Control>]) -> Vec<Item> {
        children
            .iter()
            .map(|c| {
                let min = c.get_combined_minimum_size();
                Item { style: item_style_from_meta(c), min_size: (min.x, min.y) }
            })
            .collect()
    }
}

#[godot_api]
impl IContainer for EstragonGrid {
    fn get_minimum_size(&self) -> Vector2 {
        let children = self.layout_children();
        match measure_container(&self.container_style(), &Self::items(&children)) {
            Ok((w, h)) => Vector2::new(w, h),
            Err(_) => Vector2::ZERO,
        }
    }

    fn on_notification(&mut self, what: ContainerNotification) {
        if what != ContainerNotification::SORT_CHILDREN {
            return;
        }
        let children = self.layout_children();
        let size = self.base().get_size();
        match layout_container(&self.container_style(), &Self::items(&children), (size.x, size.y)) {
            Ok(rects) => {
                for (child, r) in children.into_iter().zip(rects) {
                    self.base_mut().fit_child_in_rect(&child, Rect2::new(Vector2::new(r.x, r.y), Vector2::new(r.width, r.height)));
                }
            }
            Err(e) => { godot_warn!("EstragonGrid {}: {e}", self.base().get_name()); }
        }
    }
}

const META: [&str; 5] = ["estragon_area", "estragon_column", "estragon_row", "estragon_justify_self", "estragon_align_self"];

fn meta_str(c: &Gd<Control>, key: &str) -> Option<String> {
    let key = StringName::from(key);
    if !c.has_meta(&key) {
        return None;
    }
    let s = c.get_meta(&key).to_string();
    (!s.is_empty()).then_some(s)
}

fn parse_align(s: Option<String>) -> Option<Align> {
    match s?.as_str() {
        "start" => Some(Align::Start),
        "end" => Some(Align::End),
        "center" => Some(Align::Center),
        "stretch" => Some(Align::Stretch),
        _ => None,
    }
}

fn item_style_from_meta(c: &Gd<Control>) -> NodeStyle {
    NodeStyle {
        area: meta_str(c, META[0]),
        column: meta_str(c, META[1]),
        row: meta_str(c, META[2]),
        justify_self: parse_align(meta_str(c, META[3])),
        align_self: parse_align(meta_str(c, META[4])),
        ..Default::default()
    }
}

fn set_item_meta(c: &mut Gd<Control>, s: &NodeStyle) {
    let align = |a: Align| match a {
        Align::Start => "start",
        Align::End => "end",
        Align::Center => "center",
        Align::Stretch => "stretch",
    };
    let pairs = [
        (META[0], s.area.clone()),
        (META[1], s.column.clone()),
        (META[2], s.row.clone()),
        (META[3], s.justify_self.map(|a| align(a).to_string())),
        (META[4], s.align_self.map(|a| align(a).to_string())),
    ];
    for (k, v) in pairs {
        if let Some(v) = v {
            c.set_meta(k, &v.to_variant());
        }
    }
}

/// Utilities for loading Estragon documents.
#[derive(GodotClass)]
#[class(init, base = RefCounted)]
pub struct Estragon {
    base: Base<RefCounted>,
}

#[godot_api]
impl Estragon {
    /// Build a Control tree from a `.ui.json` file. Returns `null` (and logs) on error.
    #[func]
    fn load_document(path: GString) -> Option<Gd<Control>> {
        let Some(file) = FileAccess::open(&path, ModeFlags::READ) else {
            godot_error!("Estragon: cannot open {path}");
            return None;
        };
        let text = file.get_as_text().to_string();
        match serde_json::from_str::<Document>(&text) {
            Ok(doc) => {
                let mut root = build_node(&doc.root);
                root.set_size(Vector2::new(doc.width, doc.height));
                Some(root)
            }
            Err(e) => {
                godot_error!("Estragon: {path}: {e}");
                None
            }
        }
    }
}

fn build_node(node: &Node) -> Gd<Control> {
    let mut control: Gd<Control> = match node.kind {
        Kind::Grid => {
            let mut grid = EstragonGrid::new_alloc();
            {
                let mut g = grid.bind_mut();
                let s = &node.style;
                g.columns = GString::from(s.columns.as_deref().unwrap_or(""));
                g.rows = GString::from(s.rows.as_deref().unwrap_or(""));
                g.areas = s.areas.iter().map(|a| GString::from(a.as_str())).collect();
                g.auto_rows = GString::from(s.auto_rows.as_deref().unwrap_or(""));
                g.auto_columns = GString::from(s.auto_columns.as_deref().unwrap_or(""));
                g.gap = s.gap;
                g.row_gap = s.row_gap.unwrap_or(-1.0);
                g.column_gap = s.column_gap.unwrap_or(-1.0);
                g.padding = s.padding;
            }
            grid.upcast()
        }
        Kind::Box => {
            let mut rect = ColorRect::new_alloc();
            if let Some(fill) = &node.fill {
                rect.set_color(Color::from_html(fill).unwrap_or(Color::MAGENTA));
            }
            rect.upcast()
        }
    };
    control.set_name(&node.id);
    control.set_custom_minimum_size(Vector2::new(
        node.style.min_width.unwrap_or(0.0),
        node.style.min_height.unwrap_or(0.0),
    ));
    set_item_meta(&mut control, &node.style);
    for child in &node.children {
        control.add_child(&build_node(child));
    }
    control
}
