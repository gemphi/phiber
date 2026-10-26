//! Phiber — flat, singular, one folder per hierarchy level.
//!
//! Three view traits describe the coordinator layer:
//!   - `view::BViewItem` — emits exactly one `BView::<level>` variant.
//!   - `view::BViewList` — iterates a store collection into `Vec<BView>`.
//!
//! One store trait describes the static side:
//!   - `spec::BSpec`     — every static spec at every level.
//!
//! Static data lives ONLY in `set.rs` files. No inline arrays/tuples in any
//! `list.rs` or `item.rs`.
//!
//! Phiber library root. The `.blend` reader, scene model and asset
//! import/export live in the `phiphx` crate (`crates/phiphx`); it is
//! re-exported here as `crate::phiphx` (and the legacy alias `crate::phx`) so
//! existing `crate::phiphx::scene::…` paths keep resolving.
//! Binary entry point lives in `src/main.rs`.

pub use ::phiphx;
pub mod phx {
    pub use phiphx::*;
}
pub use phiphx::{
    AssetImportResult, AssetImporter, ImportedCollection, ImportedMaterial, ImportedObject, Mesh,
    PointCloud, PointCloudError, SampledPoint, Vertex, VertexAttributes,
};
pub use phiphx::Result as PhxResult;

pub mod cli;
pub mod level;
pub mod spec;
pub mod state;
pub mod theme;
pub mod view;

pub mod area;
pub mod host;
/// The manual, declared as data rather than written as loose files.
pub mod manual;
pub mod modal;
pub mod pane;
pub mod part;
pub mod region;
pub mod scene;
pub mod space;
/// Spatial — the depth view's frame budget (`spatial/mod.rs`).
pub mod spatial;
pub mod zone;

pub mod app;

pub use level::BLevelType;
pub use spec::{BSpec, BSpecKind};
pub use state::{BPath, BState};
pub use theme::{
    BDimensions, BEffects, BMetric, BPalette, BPaletteName, BPreferenceControl,
    BPreferenceMetadata, BRadiusScale, BRadiusToken, BResizeStyle, BResolvedColors,
    BResolvedUiStyle, BRoleMap, BShadowStyle, BSpacing, BStrokes, BStyleChoice, BStyleDescriptor,
    BStyleParseError, BStyleParser, BStyleRole, BStyleValue, BTypography, BUiStyleDefaults,
    METRICS, UI_STYLE_DESCRIPTORS, palette_for, resolve_ui_style, ui_style_defaults,
};
pub use view::{BView, BViewItem, BViewList};

/// The browser entry point (`#[wasm_bindgen(start)]`). The desktop binary
/// starts from `src/main.rs` instead.
#[cfg(target_family = "wasm")]
pub mod wasm;
// The v1 view layer (`bui`, `state`, `store`, `services` and the app shell that
// drove them) was retired to `_unused/phiber-v1/` once v2 drew every view;
// git still has it. v2 then became the crate itself: what was `crate::v2::x`
// is `crate::x`.
