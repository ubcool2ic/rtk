//! Fuzz-style tests for the pack and grid layout engines, checking invariants under random and extreme inputs.

use rtk::layout::{grid, grid_request, pack, pack_request, GridItem, PackItem, Rect, Size};
use rtk::{Anchor, Fill, GridOpts, PackOpts, Side, Sticky};

struct Rng(u64);

impl Rng {
    /// Advances the xorshift random number generator and returns the next 32-bit value.
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 as u32
    }

    /// Produces a random dimension that occasionally includes NaN, infinities, and negative values for fuzzing.
    fn dimension(&mut self) -> f32 {
        match self.next() % 16 {
            0 => f32::NAN,
            1 => f32::INFINITY,
            2 => f32::NEG_INFINITY,
            3 => -100.0,
            _ => (self.next() % 800) as f32 / 3.0,
        }
    }
}

/// Asserts that each rect is finite, non-negative in size, and contained within the parent bounds.
fn check(parent: Rect, rects: &[Rect]) {
    for rect in rects {
        for number in [rect.x, rect.y, rect.width, rect.height] {
            assert!(number.is_finite(), "{rect:?}");
        }
        assert!(rect.width >= 0.0 && rect.height >= 0.0);
        assert!(rect.x >= parent.x - 0.001 && rect.y >= parent.y - 0.001);
        assert!(rect.x + rect.width <= parent.x + parent.width + 0.001, "{rect:?}");
        assert!(rect.y + rect.height <= parent.y + parent.height + 0.001, "{rect:?}");
    }
}

/// Runs 500 randomized pack and grid layout configurations and validates the resulting geometry.
#[test]
fn five_hundred_random_pack_and_grid_configurations() {
    let mut rng = Rng(0x0123_4567_89ab_cdef);
    for _ in 0..500 {
        let parent = Rect::new(7.0, 11.0, rng.dimension(), rng.dimension());
        let mut packed = Vec::new();
        let mut gridded = Vec::new();
        for _ in 0..(rng.next() % 24) {
            let requested = Size { width: rng.dimension(), height: rng.dimension() };
            let side = match rng.next() % 4 {
                0 => Side::Top,
                1 => Side::Bottom,
                2 => Side::Left,
                _ => Side::Right,
            };
            let fill = match rng.next() % 4 {
                0 => Fill::None,
                1 => Fill::X,
                2 => Fill::Y,
                _ => Fill::Both,
            };
            let anchor = match rng.next() % 9 {
                0 => Anchor::Center,
                1 => Anchor::N,
                2 => Anchor::S,
                3 => Anchor::E,
                4 => Anchor::W,
                5 => Anchor::Ne,
                6 => Anchor::Nw,
                7 => Anchor::Se,
                _ => Anchor::Sw,
            };
            packed.push(PackItem {
                requested,
                opts: PackOpts {
                    side, fill, anchor,
                    expand: rng.next() % 2 == 0,
                    padx: rng.dimension(),
                    pady: rng.dimension(),
                },
            });
            let sticky = match rng.next() % 5 {
                0 => Sticky::NONE,
                1 => Sticky::EW,
                2 => Sticky::NS,
                3 => Sticky::N | Sticky::E,
                _ => Sticky::NSEW,
            };
            gridded.push(GridItem {
                requested,
                opts: GridOpts {
                    row: (rng.next() % 12) as usize,
                    column: (rng.next() % 12) as usize,
                    rowspan: (rng.next() % 5) as usize,
                    columnspan: (rng.next() % 5) as usize,
                    padx: rng.dimension(),
                    pady: rng.dimension(),
                    sticky,
                },
            });
        }
        let weights: Vec<_> = (0..16).map(|index| (index, rng.next() % 8)).collect();
        check(parent, &pack(parent, &packed));
        check(parent, &grid(parent, &gridded, &weights, &weights));
        for size in [pack_request(&packed), grid_request(&gridded)] {
            assert!(size.width.is_finite() && size.width >= 0.0);
            assert!(size.height.is_finite() && size.height >= 0.0);
        }
    }
}

/// Ensures pathological grid indices and spans are bounded and do not overflow or panic.
#[test]
fn pathological_grid_indices_and_spans_are_bounded() {
    let parent = Rect::new(0.0, 0.0, 10.0, 10.0);
    let items = [GridItem {
        requested: Size::new(100.0, 100.0),
        opts: GridOpts {
            row: usize::MAX,
            column: usize::MAX,
            rowspan: usize::MAX,
            columnspan: usize::MAX,
            sticky: Sticky::NSEW,
            ..GridOpts::default()
        },
    }];
    check(parent, &grid(parent, &items, &[(usize::MAX, u32::MAX)], &[]));
}
