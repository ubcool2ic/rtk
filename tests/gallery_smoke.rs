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

/// Runs the gallery example headlessly and asserts the PNGs are non-empty.
#[test]
fn gallery_screenshots() {
    if std::env::var("RTK_GALLERY_TEST").ok().as_deref() != Some("1") {
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
            let meta = std::fs::metadata(&path)
                .unwrap_or_else(|_| panic!("missing screenshot {}", path.display()));
            assert!(meta.len() > 0, "empty screenshot {}", path.display());
        }
    }

    let hidpi = dir.join("gallery_light_basics_2x.png");
    let meta = std::fs::metadata(&hidpi)
        .unwrap_or_else(|_| panic!("missing screenshot {}", hidpi.display()));
    assert!(meta.len() > 0, "empty screenshot {}", hidpi.display());

    let _ = std::fs::remove_dir_all(&dir);
}
