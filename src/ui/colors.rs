//! The app's colour palette, as raw hex values for `gpui::rgb`. No theme
//! abstraction — just the shared consts every view uses.

pub const SIDEBAR_BACKGROUND: u32 = 0x252526;

// Translucent (0xRRGGBBAA) so the window blur shows through behind them.
pub const SIDEBAR_BLUR_BACKGROUND: u32 = 0x252526a6;
pub const CONTENT_BLUR_BACKGROUND: u32 = 0x1e1e1ed9;
pub const BORDER: u32 = 0x3c3c3c;
pub const ROW_SELECTED_BACKGROUND: u32 = 0x37373d;
pub const ROW_HOVER_BACKGROUND: u32 = 0x2a2d2e;
pub const PRIMARY_TEXT: u32 = 0xffffff;
pub const SECONDARY_TEXT: u32 = 0xcccccc;
pub const SUCCESS: u32 = 0x4ec9b0;
pub const DANGER: u32 = 0xf14c4c;
pub const WARNING: u32 = 0xcca700;
pub const NEUTRAL: u32 = 0x8c8c8c;
pub const CHART_SERIES: u32 = 0x569cd6;
