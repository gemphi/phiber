//! The fonts in `assets/phi-fonts`, as this machine's text system sees them.
//!
//! `cargo run -p phiber --example fonts` registers every bundled face the way
//! the app does (`theme::vendor::install`) and prints each family: whether the
//! text system knows it by name, and the advance of an `M` at every weight it
//! ships. Run it after dropping a folder in. A family marked UNKNOWN would
//! draw in the fallback face, and an advance equal to Segoe UI's is that
//! fallback; a family's two widths (Phi-Soft, Phi-Soft Condensed) differ.

use gpui::{App, Font, FontFeatures, FontStyle, FontWeight, px};
use phiber::theme::style::bundle;

fn advance(cx: &App, family: &str, weight: u16) -> f32 {
    let font = Font {
        family: family.to_string().into(),
        features: FontFeatures::default(),
        weight: FontWeight(weight as f32),
        style: FontStyle::Normal,
        fallbacks: None,
    };
    let id = cx.text_system().resolve_font(&font);
    cx.text_system().advance(id, px(100.0), 'M').map_or(f32::NAN, |size| size.width.as_f32())
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        phiber::theme::vendor::install(cx);
        let known = cx.text_system().all_font_names();
        println!("Segoe UI 'M' at 100 px: {:.1}", advance(cx, "Segoe UI", 400));
        println!("{} bundled families:", bundle::bundled().len());
        for family in bundle::bundled() {
            let named = if known.iter().any(|name| name == family.name) { "known  " } else { "UNKNOWN" };
            let widths: Vec<String> = family.weights.iter().map(|&weight| format!("{weight}: {:.1}", advance(cx, family.name, weight))).collect();
            println!("  {named} {:<26} {:<24} M = {}", family.name, family.slug, widths.join("  "));
        }
        cx.quit();
    });
}
