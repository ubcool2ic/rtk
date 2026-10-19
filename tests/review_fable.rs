//! Regression tests for fixes from the Fable 5.1 review (v0.8.1).
use rtk::colormap::{sample, Colormap};

/// Viridis must end in yellow, not the bogus teal from the v0.8.0 table (rev3 F1).
#[test]
fn viridis_endpoint_is_yellow() {
    let c = sample(Colormap::Viridis, 1.0);
    assert!(c.r > 240 && c.g > 220 && c.b < 50, "{:?}", (c.r, c.g, c.b));
}

/// Inferno must be monotonic in brightness (no fold-back to black).
#[test]
fn inferno_brightens() {
    let lum = |t| { let c = sample(Colormap::Inferno, t); c.r as u32 + c.g as u32 + c.b as u32 };
    assert!(lum(1.0) > lum(0.75) && lum(0.75) > lum(0.25));
}
