//! The sizes a `BFlex` is written in (`BDim`): a control's height, a tile's
//! square, a panel's width, each resolved from the theme's dimensions. Split
//! from `token.rs` for the file cap; it is one of the element's tokens.

use super::super::ctx::BRenderCtx;

/// A named dimension from the theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BDim {
    HeaderHeight,
    /// A card's heading strip: the button height, so a panel heading and
    /// the rows and buttons under it are one module.
    CardHead,
    FooterHeight,
    ButtonHeight,
    /// An icon tile: the button less its inset (`tile_size`).
    Tile,
    /// A glyph's tile inside a row: an icon with a small pad round it. A
    /// menu row's icon, a source list's and a checkbox or radio mark are this
    /// one square, so every row's glyph stands on the same tile.
    GlyphTile,
    RowHeight,
    /// A tree's row (`tree_row_height`).
    TreeRow,
    /// The compact row, one of the two heights every row is: a menu's, a
    /// source list's, a tree's, a list's, a table's (`tree_row_height`). The
    /// other is the module (`CardHead`): a property's, a value's.
    CompactRow,
    /// A menu-popover row: compact, below the module (`menu_row_height`).
    MenuRow,
    ScrubbingHeight,
    AssetShelfHeight,
    BottomStripHeight,
    BottomPanelHeight,
    StatusLogHeight,
    ToolbarWidth,
    /// Every rail's collapsed width (`BResolvedUiStyle::rail_width`).
    RailWidth,
    NavbarWidth,
    SidebarMinWidth,
    PanelWidth,
    MenuWidth,
    ChannelsWidth,
    ToolPropsWidth,
    SideZoneDefaultWidth,
}

impl BDim {
    pub(super) fn px(self, ctx: &BRenderCtx) -> f32 {
        let d = &ctx.style.dimensions;
        match self {
            Self::HeaderHeight => d.header_height,
            Self::CardHead => d.card_head_height(),
            Self::FooterHeight => d.footer_height,
            Self::ButtonHeight => d.button_height,
            Self::Tile => d.tile_size(),
            Self::GlyphTile => d.icon_size + 2.0 * ctx.style.spacing.pad_xs,
            Self::RowHeight => d.row_height,
            Self::TreeRow | Self::CompactRow => d.tree_row_height,
            Self::MenuRow => d.menu_row_height,
            Self::ScrubbingHeight => d.scrubbing_height,
            Self::AssetShelfHeight => d.asset_shelf_height,
            Self::BottomStripHeight => d.bottom_strip_height,
            Self::BottomPanelHeight => d.bottom_panel_height,
            Self::StatusLogHeight => d.status_log_height,
            Self::ToolbarWidth => d.toolbar_width,
            Self::RailWidth => ctx.style.rail_width(),
            Self::NavbarWidth => d.navbar_width,
            Self::SidebarMinWidth => d.sidebar_min_width,
            Self::PanelWidth => d.panel_width,
            Self::MenuWidth => d.menu_width,
            Self::ChannelsWidth => d.channels_width,
            Self::ToolPropsWidth => d.tool_props_width,
            Self::SideZoneDefaultWidth => d.side_zone_default_width,
        }
    }
}
