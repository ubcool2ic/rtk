//! Smoke test for the 1.0 gallery example.
//!
//! This test shells out to `cargo run --example gallery` to render headless
//! screenshots and checks that the PNGs exist and are non-empty. It only runs
//! when the `RTK_GALLERY_TEST=1` environment variable is set, so it is skipped
//! by default (the example build is slow).

use std::path::PathBuf;
use std::process::Command;

/// Returns a unique temporary directory for the screenshot output.
fn temp_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    let unique = format!(
        "rtk_gallery_smoke_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    dir.push(unique);
    dir
}

/// Asserts the file is a PNG with non-zero dimensions and returns its width.
fn png_width(path: &std::path::Path) -> u32 {
    let bytes = std::fs::read(path)
        .unwrap_or_else(|_| panic!("missing screenshot {}", path.display()));
    assert!(bytes.len() > 33, "truncated screenshot {}", path.display());
    assert_eq!(
        &bytes[..8],
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
        "not a PNG: {}",
        path.display()
    );
    assert_eq!(&bytes[12..16], b"IHDR", "missing IHDR in {}", path.display());
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    assert!(width > 0 && height > 0, "zero-sized screenshot {}", path.display());
    width
}

/// Runs the gallery example headlessly and asserts the PNGs are valid and non-empty.
#[test]
fn gallery_screenshots() {
    if std::env::var("RTK_GALLERY_TEST").ok().as_deref() != Some("1") {
        eprintln!("gallery_screenshots skipped: set RTK_GALLERY_TEST=1 to run");
        return;
    }

    let dir = temp_dir();
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let status = Command::new(env!("CARGO"))
        .args([
            "run",
            "--example",
            "gallery",
            "--quiet",
            "--",
            "--screenshot-dir",
        ])
        .arg(&dir)
        .status()
        .expect("run gallery example");
    assert!(status.success(), "gallery example failed");

    let themes = [
        "light",
        "dark",
        "highcontrast",
        "fluent",
        "material",
        "moderndark",
        "modernlight",
    ];
    let pages = ["basics", "inputs", "data", "charts", "theme"];
    for theme in themes {
        for page in pages {
            let path = dir.join(format!("gallery_{theme}_{page}.png"));
            png_width(&path);
        }
    }

    let hidpi = dir.join("gallery_light_basics_2x.png");
    let base = png_width(&dir.join("gallery_light_basics.png"));
    assert!(
        png_width(&hidpi) > base,
        "2x screenshot is not larger than 1x: {}",
        hidpi.display()
    );

    let _ = std::fs::remove_dir_all(&dir);
}
