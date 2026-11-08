//! The Estragon document schema: the contract shared by the web studio and Godot.
//!
//! Deliberately constrained (React Native style): a handful of primitives and a
//! style vocabulary limited to what Godot can express natively.

use serde::{Deserialize, Serialize};

/// A `.ui.json` document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// Design viewport, e.g. 1280x720.
    pub width: f32,
    pub height: f32,
    pub root: Node,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    #[serde(default)]
    pub kind: Kind,
    #[serde(default)]
    pub style: NodeStyle,
    /// Spike-only presentation: a flat fill colour (hex) so layouts are visible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// A layout container (Godot: `EstragonGrid`).
    Grid,
    /// A plain leaf box (Godot: `Panel`/`ColorRect` for now).
    #[default]
    Box,
}

/// Container + item style. Strings use CSS syntax and are parsed by Taffy.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeStyle {
    // --- container (grid) ---
    /// `grid-template-columns`, e.g. `"240px 1fr 240px"` or `"repeat(4, 64px)"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<String>,
    /// `grid-template-rows`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<String>,
    /// `grid-template-areas`, one string per row, e.g. `["map . hp", "chat bar bar"]`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub areas: Vec<String>,
    /// `grid-auto-rows` / `grid-auto-columns`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_rows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_columns: Option<String>,
    /// Gap in px, applied to both axes unless overridden.
    pub gap: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_gap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_gap: Option<f32>,
    /// Uniform padding in px.
    pub padding: f32,

    // --- item ---
    /// `grid-area: <name>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    /// `grid-column`, e.g. `"1 / 3"` or `"span 2"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    /// `grid-row`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_self: Option<Align>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<Align>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Align {
    Start,
    End,
    Center,
    Stretch,
}
