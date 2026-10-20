//! Pure, bounded geometry, independent of fonts and windows.
//!
//! Nonfinite dimensions and padding become zero. Grid indices are capped at
//! 4095, spans at the remaining track count, and undersized grids shrink.
//! Results are contained in the normalized parent rectangle.

use std::ops::{BitOr, BitOrAssign};

pub(crate) const MAX_TRACKS: usize = 4096;

/// Clamps a length to a finite non-negative range, mapping nonfinite values to zero.
pub(crate) fn length(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 10_000_000.0)
    } else {
        0.0
    }
}

/// Clamps a position to a finite range, mapping nonfinite values to zero.
pub(crate) fn position(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1_000_000_000.0, 1_000_000_000.0)
    } else {
        0.0
    }
}

/// A width/height pair in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    /// Width in logical px (nonfinite values become 0).
    pub width: f32,
    /// Height in logical px (nonfinite values become 0).
    pub height: f32,
}

impl Size {
    /// Creates a size, normalizing nonfinite or negative dimensions to 0.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width: length(width), height: length(height) }
    }
}

/// An axis-aligned rectangle in logical pixels (origin at the top-left).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Left edge in logical px.
    pub x: f32,
    /// Top edge in logical px.
    pub y: f32,
    /// Width in logical px.
    pub width: f32,
    /// Height in logical px.
    pub height: f32,
}

impl Rect {
    /// Creates a rectangle, normalizing nonfinite values (positions clamp to
    /// ±1e9, dimensions to `0..=1e7`).
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x: position(x),
            y: position(y),
            width: length(width),
            height: length(height),
        }
    }

    /// Shrinks the rectangle by the given horizontal and vertical insets, clamped so it never inverts.
    fn normalized(self) -> Self {
        Self::new(self.x, self.y, self.width, self.height)
    }

    /// Shrinks the rectangle by `x` on each horizontal side and `y` on each
    /// vertical side (clamped to half the extent so it never inverts).
    pub fn inset(self, x: f32, y: f32) -> Self {
        let rect = self.normalized();
        let x = length(x).min(rect.width / 2.0);
        let y = length(y).min(rect.height / 2.0);
        Self::new(
            rect.x + x,
            rect.y + y,
            rect.width - 2.0 * x,
            rect.height - 2.0 * y,
        )
    }

    /// Returns the overlapping area of two rectangles (zero-sized if disjoint).
    pub fn intersection(self, other: Self) -> Self {
        let a = self.normalized();
        let b = other.normalized();
        let x = a.x.max(b.x);
        let y = a.y.max(b.y);
        Self::new(
            x,
            y,
            (a.x + a.width).min(b.x + b.width) - x,
            (a.y + a.height).min(b.y + b.height) - y,
        )
    }

    /// Clamps even empty rectangles' origins into the parent.
    pub fn within(self, parent: Self) -> Self {
        let parent = parent.normalized();
        let rect = self.normalized();
        let right = parent.x + parent.width;
        let bottom = parent.y + parent.height;
        let x = rect.x.clamp(parent.x, right);
        let y = rect.y.clamp(parent.y, bottom);
        Self::new(
            x,
            y,
            ((rect.x + rect.width).min(right) - x).max(0.0),
            ((rect.y + rect.height).min(bottom) - y).max(0.0),
        )
    }

    /// Returns true if the point lies inside the rectangle (right/bottom edges
    /// exclusive).
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width
            && y < self.y + self.height
    }

    /// Returns a copy of the rectangle scaled by the given factor.
    pub(crate) fn scaled(self, scale: f32) -> Self {
        Self::new(
            self.x * scale,
            self.y * scale,
            self.width * scale,
            self.height * scale,
        )
    }
}

/// The side of the cavity a packed widget is placed against (tkinter `side`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    /// Stack from the top (default).
    #[default]
    Top,
    /// Stack from the bottom.
    Bottom,
    /// Stack from the left.
    Left,
    /// Stack from the right.
    Right,
}

/// Which axes a packed widget stretches to fill (tkinter `fill`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fill {
    /// Keep the requested size (default).
    #[default]
    None,
    /// Stretch horizontally.
    X,
    /// Stretch vertically.
    Y,
    /// Stretch both axes.
    Both,
}

/// Where a widget sits inside its allocated parcel (tkinter `anchor`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Anchor {
    /// Centered (default).
    #[default]
    Center,
    /// Top-center.
    N,
    /// Bottom-center.
    S,
    /// Middle-right.
    E,
    /// Middle-left.
    W,
    /// Top-right.
    Ne,
    /// Top-left.
    Nw,
    /// Bottom-right.
    Se,
    /// Bottom-left.
    Sw,
}

impl Anchor {
    /// Returns the `(x, y)` fractions in `0.0..=1.0` of the parcel used to
    /// position the widget for this anchor.
    pub fn fractions(self) -> (f32, f32) {
        match self {
            Self::Center => (0.5, 0.5),
            Self::N => (0.5, 0.0),
            Self::S => (0.5, 1.0),
            Self::E => (1.0, 0.5),
            Self::W => (0.0, 0.5),
            Self::Ne => (1.0, 0.0),
            Self::Nw => (0.0, 0.0),
            Self::Se => (1.0, 1.0),
            Self::Sw => (0.0, 1.0),
        }
    }
}

/// Options for the place geometry manager (tkinter `place`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlaceOpts {
    /// Absolute x offset in logical px from the container's left edge.
    pub x: f32,
    /// Absolute y offset in logical px from the container's top edge.
    pub y: f32,
    /// Relative x offset as a fraction of the container width.
    pub relx: f32,
    /// Relative y offset as a fraction of the container height.
    pub rely: f32,
    /// Fixed width in logical px; overrides `relwidth` and the requested size.
    pub width: Option<f32>,
    /// Fixed height in logical px; overrides `relheight` and the requested size.
    pub height: Option<f32>,
    /// Width as a fraction of the container width.
    pub relwidth: Option<f32>,
    /// Height as a fraction of the container height.
    pub relheight: Option<f32>,
    /// Which point of the widget the coordinates refer to.
    pub anchor: Anchor,
}

/// Computes the rectangle for a placed widget, clamped inside `container`.
pub fn place_rect(container: Rect, requested: Size, opts: PlaceOpts) -> Rect {
    let container = container.normalized();
    let relx = if opts.relx.is_finite() { opts.relx } else { 0.0 };
    let rely = if opts.rely.is_finite() { opts.rely } else { 0.0 };
    let anchor_x = container.x + position(opts.x) + relx * container.width;
    let anchor_y = container.y + position(opts.y) + rely * container.height;

    let w = if let Some(w) = opts.width {
        length(w)
    } else if let Some(rw) = opts.relwidth {
        if rw.is_finite() {
            (rw * container.width).max(0.0)
        } else {
            0.0
        }
    } else {
        requested.width
    };

    let h = if let Some(h) = opts.height {
        length(h)
    } else if let Some(rh) = opts.relheight {
        if rh.is_finite() {
            (rh * container.height).max(0.0)
        } else {
            0.0
        }
    } else {
        requested.height
    };

    let (frac_x, frac_y) = opts.anchor.fractions();
    let left = anchor_x - frac_x * w;
    let top = anchor_y - frac_y * h;

    Rect::new(left, top, w, h).within(container)
}

/// Options for the pack geometry manager (tkinter `pack`).
#[derive(Clone, Copy, Debug, Default)]
pub struct PackOpts {
    /// Side of the cavity to pack against.
    pub side: Side,
    /// Axes the widget stretches along.
    pub fill: Fill,
    /// Whether the widget claims a share of leftover space.
    pub expand: bool,
    /// Horizontal padding in logical px on each side.
    pub padx: f32,
    /// Vertical padding in logical px on each side.
    pub pady: f32,
    /// Alignment inside the parcel when not filling.
    pub anchor: Anchor,
}

/// A bit set of grid cell edges a widget sticks to (tkinter `sticky`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sticky(u8);

impl Sticky {
    /// Sticks to no edges (centered in the cell).
    pub const NONE: Self = Self(0);
    /// Sticks to the top edge.
    pub const N: Self = Self(1);
    /// Sticks to the bottom edge.
    pub const S: Self = Self(2);
    /// Sticks to the right edge.
    pub const E: Self = Self(4);
    /// Sticks to the left edge.
    pub const W: Self = Self(8);
    /// Sticks to the top and bottom edges (stretches vertically).
    pub const NS: Self = Self(3);
    /// Sticks to the left and right edges (stretches horizontally).
    pub const EW: Self = Self(12);
    /// Sticks to all four edges (fills the cell).
    pub const NSEW: Self = Self(15);

    /// Returns true if every edge in `other` is also set in `self`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for Sticky {
    type Output = Self;

    /// Combines two sticky flags with bitwise OR.
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Sticky {
    /// Adds the sticky flags of another value in place.
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Options for the grid geometry manager (tkinter `grid`).
#[derive(Clone, Copy, Debug)]
pub struct GridOpts {
    /// Zero-based row index (capped at 4095).
    pub row: usize,
    /// Zero-based column index (capped at 4095).
    pub column: usize,
    /// Number of rows the widget spans (at least 1).
    pub rowspan: usize,
    /// Number of columns the widget spans (at least 1).
    pub columnspan: usize,
    /// Cell edges the widget sticks to.
    pub sticky: Sticky,
    /// Horizontal padding in logical px on each side.
    pub padx: f32,
    /// Vertical padding in logical px on each side.
    pub pady: f32,
}

impl Default for GridOpts {
    /// Cell `(0, 0)`, span 1, no sticky, no padding.
    fn default() -> Self {
        Self {
            row: 0,
            column: 0,
            rowspan: 1,
            columnspan: 1,
            sticky: Sticky::NONE,
            padx: 0.0,
            pady: 0.0,
        }
    }
}

/// One widget's requested size and pack options, as consumed by [`pack`].
#[derive(Clone, Copy, Debug)]
pub struct PackItem {
    /// The widget's natural size in logical px.
    pub requested: Size,
    /// Its pack options.
    pub opts: PackOpts,
}

/// One widget's requested size and grid options, as consumed by [`grid`].
#[derive(Clone, Copy, Debug)]
pub struct GridItem {
    /// The widget's natural size in logical px.
    pub requested: Size,
    /// Its grid options.
    pub opts: GridOpts,
}

/// Returns the size grown by the given padding on each axis.
fn padded(size: Size, x: f32, y: f32) -> Size {
    Size::new(
        length(size.width) + 2.0 * length(x),
        length(size.height) + 2.0 * length(y),
    )
}

/// Returns whether the side is horizontal (left or right).
fn horizontal(side: Side) -> bool {
    matches!(side, Side::Left | Side::Right)
}

/// Returns the minimum container size needed to fit all packed items.
pub fn pack_request(items: &[PackItem]) -> Size {
    let (mut used_w, mut used_h) = (0.0_f32, 0.0_f32);
    let (mut width, mut height) = (0.0_f32, 0.0_f32);
    for item in items {
        let size = padded(item.requested, item.opts.padx, item.opts.pady);
        if horizontal(item.opts.side) {
            height = length(height.max(used_h + size.height));
            used_w = length(used_w + size.width);
        } else {
            width = length(width.max(used_w + size.width));
            used_h = length(used_h + size.height);
        }
    }
    Size::new(width.max(used_w), height.max(used_h))
}

/// Computes the extra space distributed to expanding items along one axis.
fn expansion(items: &[PackItem], axis_x: bool, mut available: f32) -> f32 {
    let mut count = 0_usize;
    let mut limit = available;
    for item in items {
        let size = padded(item.requested, item.opts.padx, item.opts.pady);
        let needed = if axis_x { size.width } else { size.height };
        if horizontal(item.opts.side) == axis_x {
            available -= needed;
            count += usize::from(item.opts.expand);
        } else if count > 0 {
            limit = limit.min((available - needed) / count as f32);
        }
    }
    if count > 0 {
        limit = limit.min(available / count as f32);
    }
    length(limit)
}

/// Lays out packed items inside `container`, returning one rectangle per item
/// in input order (all clamped inside the container).
pub fn pack(container: Rect, items: &[PackItem]) -> Vec<Rect> {
    let container = container.normalized();
    let mut cavity = container;
    let mut result = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let opts = item.opts;
        let size = padded(item.requested, opts.padx, opts.pady);
        let axis_x = horizontal(opts.side);
        let available = if axis_x { cavity.width } else { cavity.height };
        let needed = if axis_x { size.width } else { size.height };
        let extra = if opts.expand {
            expansion(items.get(index..).unwrap_or_default(), axis_x, available)
        } else {
            0.0
        };
        let amount = (needed + extra).min(available);
        let parcel = match opts.side {
            Side::Top => {
                let parcel = Rect::new(cavity.x, cavity.y, cavity.width, amount);
                cavity.y += amount;
                cavity.height = (cavity.height - amount).max(0.0);
                parcel
            }
            Side::Bottom => {
                cavity.height = (cavity.height - amount).max(0.0);
                Rect::new(cavity.x, cavity.y + cavity.height, cavity.width, amount)
            }
            Side::Left => {
                let parcel = Rect::new(cavity.x, cavity.y, amount, cavity.height);
                cavity.x += amount;
                cavity.width = (cavity.width - amount).max(0.0);
                parcel
            }
            Side::Right => {
                cavity.width = (cavity.width - amount).max(0.0);
                Rect::new(cavity.x + cavity.width, cavity.y, amount, cavity.height)
            }
        };
        let inner = parcel.within(container).inset(opts.padx, opts.pady);
        let width = if matches!(opts.fill, Fill::X | Fill::Both) {
            inner.width
        } else {
            length(item.requested.width).min(inner.width)
        };
        let height = if matches!(opts.fill, Fill::Y | Fill::Both) {
            inner.height
        } else {
            length(item.requested.height).min(inner.height)
        };
        let (ax, ay) = opts.anchor.fractions();
        result.push(
            Rect::new(
                inner.x + (inner.width - width) * ax,
                inner.y + (inner.height - height) * ay,
                width,
                height,
            )
            .within(container),
        );
    }
    result
}

/// Clamps a grid span to the valid track range, ensuring at least one track.
fn span(start: usize, count: usize) -> (usize, usize) {
    let start = start.min(MAX_TRACKS - 1);
    (start, count.max(1).min(MAX_TRACKS - start))
}

/// Returns the start track, span count, and needed size for a grid item along one axis.
fn specification(item: &GridItem, axis_x: bool) -> (usize, usize, f32) {
    let opts = item.opts;
    let size = padded(item.requested, opts.padx, opts.pady);
    let (start, count) = if axis_x {
        span(opts.column, opts.columnspan)
    } else {
        span(opts.row, opts.rowspan)
    };
    (start, count, if axis_x { size.width } else { size.height })
}

/// Computes the requested size of each grid track along one axis from the items.
fn tracks(items: &[GridItem], axis_x: bool) -> Vec<f32> {
    let count = items
        .iter()
        .map(|item| {
            let (start, count, _) = specification(item, axis_x);
            start + count
        })
        .max()
        .unwrap_or(0);
    let mut tracks = vec![0.0_f32; count];
    for item in items {
        let (start, count, needed) = specification(item, axis_x);
        if count == 1 {
            if let Some(track) = tracks.get_mut(start) {
                *track = track.max(needed);
            }
        }
    }
    for item in items {
        let (start, count, needed) = specification(item, axis_x);
        if count > 1 {
            if let Some(range) = tracks.get_mut(start..start + count) {
                let deficit = (needed - range.iter().sum::<f32>()).max(0.0);
                for track in range {
                    *track += deficit / count as f32;
                }
            }
        }
    }
    tracks
}

/// Returns the minimum container size needed to fit all gridded items.
pub fn grid_request(items: &[GridItem]) -> Size {
    Size::new(
        tracks(items, true).iter().sum(),
        tracks(items, false).iter().sum(),
    )
}

/// Distributes available space across tracks according to weights, shrinking proportionally when over budget.
fn weighted(mut tracks: Vec<f32>, available: f32, weights: &[(usize, u32)]) -> Vec<f32> {
    let last = weights.iter().filter(|(i, _)| *i < MAX_TRACKS).map(|(i, _)| *i).max();
    if let Some(last) = last {
        tracks.resize(tracks.len().max(last + 1), 0.0);
    }
    let requested: f32 = tracks.iter().sum();
    if requested > available && requested > 0.0 {
        let ratio = available / requested;
        for track in &mut tracks {
            *track *= ratio;
        }
    } else {
        let extra = (available - requested).max(0.0);
        let total: f64 = weights
            .iter()
            .filter(|(i, _)| *i < MAX_TRACKS)
            .map(|(_, weight)| f64::from(*weight))
            .sum();
        if total > 0.0 {
            for &(index, weight) in weights {
                if let Some(track) = tracks.get_mut(index) {
                    *track += (f64::from(extra) * f64::from(weight) / total) as f32;
                }
            }
        }
    }
    tracks
}

/// Returns the offset and size of a track range, clamped to the available space.
fn track_range(tracks: &[f32], start: usize, count: usize, available: f32) -> (f32, f32) {
    let offset = tracks.get(..start).unwrap_or_default().iter().sum::<f32>().min(available);
    let size = tracks
        .get(start..start + count)
        .unwrap_or_default()
        .iter()
        .sum::<f32>()
        .min((available - offset).max(0.0));
    (offset, size)
}

/// Computes the offset and size of a widget within a cell according to sticky flags.
fn sticky_axis(start: f32, available: f32, requested: f32, low: bool, high: bool) -> (f32, f32) {
    let size = if low && high { available } else { length(requested).min(available) };
    let offset = if low {
        0.0
    } else if high {
        available - size
    } else {
        (available - size) / 2.0
    };
    (start + offset, size)
}

/// Lays out gridded items inside `container`, returning one rectangle per item
/// in input order. `column_weights`/`row_weights` are `(index, weight)` pairs
/// distributing leftover space.
pub fn grid(
    container: Rect,
    items: &[GridItem],
    column_weights: &[(usize, u32)],
    row_weights: &[(usize, u32)],
) -> Vec<Rect> {
    let container = container.normalized();
    let columns = weighted(tracks(items, true), container.width, column_weights);
    let rows = weighted(tracks(items, false), container.height, row_weights);
    items
        .iter()
        .map(|item| {
            let opts = item.opts;
            let (column, columnspan) = span(opts.column, opts.columnspan);
            let (row, rowspan) = span(opts.row, opts.rowspan);
            let (dx, width) = track_range(&columns, column, columnspan, container.width);
            let (dy, height) = track_range(&rows, row, rowspan, container.height);
            let cell = Rect::new(container.x + dx, container.y + dy, width, height)
                .within(container)
                .inset(opts.padx, opts.pady);
            let (x, width) = sticky_axis(
                cell.x, cell.width, item.requested.width,
                opts.sticky.contains(Sticky::W), opts.sticky.contains(Sticky::E),
            );
            let (y, height) = sticky_axis(
                cell.y, cell.height, item.requested.height,
                opts.sticky.contains(Sticky::N), opts.sticky.contains(Sticky::S),
            );
            Rect::new(x, y, width, height).within(container)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that place anchors position a widget correctly within its container.
    #[test]
    fn place_anchors_test() {
        let container = Rect::new(0.0, 0.0, 100.0, 100.0);
        let requested = Size::new(20.0, 10.0);
        let opts = PlaceOpts {
            x: 50.0,
            y: 50.0,
            anchor: Anchor::Center,
            ..Default::default()
        };
        let r = place_rect(container, requested, opts);
        assert_eq!(r, Rect::new(40.0, 45.0, 20.0, 10.0));

        let opts_nw = PlaceOpts {
            x: 10.0,
            y: 20.0,
            anchor: Anchor::Nw,
            ..Default::default()
        };
        let r_nw = place_rect(container, requested, opts_nw);
        assert_eq!(r_nw, Rect::new(10.0, 20.0, 20.0, 10.0));
    }
}
#[cfg(test)]
mod legacy_tests {
    use super::*;

    /// Builds a pack item with the given requested size and options.
    fn slave(width: f32, height: f32, opts: PackOpts) -> PackItem {
        PackItem { requested: Size::new(width, height), opts }
    }

    /// Builds a grid item with the given width at the specified row and column.
    fn cell(width: f32, row: usize, column: usize) -> GridItem {
        GridItem {
            requested: Size::new(width, 10.0),
            opts: GridOpts { row, column, ..GridOpts::default() },
        }
    }

    /// Verifies that top-packed items stack vertically and are centered horizontally.
    #[test]
    fn pack_top_stacking() {
        let items = [
            slave(20.0, 10.0, PackOpts::default()),
            slave(30.0, 15.0, PackOpts::default()),
        ];
        let result = pack(Rect::new(0.0, 0.0, 100.0, 80.0), &items);
        assert_eq!(result[0], Rect::new(40.0, 0.0, 20.0, 10.0));
        assert_eq!(result[1], Rect::new(35.0, 10.0, 30.0, 15.0));
        assert_eq!(pack_request(&items), Size::new(30.0, 25.0));
    }

    /// Verifies that a left-packed item with vertical fill stretches to the container height.
    #[test]
    fn pack_left_fill_y() {
        let item = slave(20.0, 10.0, PackOpts {
            side: Side::Left, fill: Fill::Y, ..PackOpts::default()
        });
        assert_eq!(
            pack(Rect::new(0.0, 0.0, 100.0, 80.0), &[item])[0],
            Rect::new(0.0, 0.0, 20.0, 80.0)
        );
    }

    /// Verifies that two expanding slaves with fill Both split the container evenly along the pack axis.
    #[test]
    fn pack_expand_sharing() {
        let opts = PackOpts { fill: Fill::Both, expand: true, ..PackOpts::default() };
        let result = pack(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            &[slave(10.0, 10.0, opts), slave(10.0, 10.0, opts)],
        );
        assert_eq!(result[0], Rect::new(0.0, 0.0, 100.0, 50.0));
        assert_eq!(result[1], Rect::new(0.0, 50.0, 100.0, 50.0));
    }

    /// Checks that padx and pady shrink the allocated slot and are included in the pack request size.
    #[test]
    fn pack_padding() {
        let opts = PackOpts { fill: Fill::X, padx: 3.0, pady: 4.0, ..PackOpts::default() };
        let items = [slave(20.0, 10.0, opts), slave(20.0, 10.0, opts)];
        let result = pack(Rect::new(0.0, 0.0, 100.0, 100.0), &items);
        assert_eq!(result[0], Rect::new(3.0, 4.0, 94.0, 10.0));
        assert_eq!(result[1].y, 22.0);
        assert_eq!(pack_request(&items), Size::new(26.0, 36.0));
    }

    /// Ensures an expanding item reserves space so a perpendicular side item keeps its requested extent.
    #[test]
    fn expansion_reserves_orthogonal_requests() {
        let items = [
            slave(10.0, 10.0, PackOpts {
                expand: true, fill: Fill::Both, ..PackOpts::default()
            }),
            slave(20.0, 40.0, PackOpts {
                side: Side::Left, fill: Fill::Y, ..PackOpts::default()
            }),
        ];
        let result = pack(Rect::new(0.0, 0.0, 100.0, 80.0), &items);
        assert_eq!(result[0].height, 40.0);
        assert_eq!(result[1].height, 40.0);
    }

    /// Confirms grid column widths are the maximum cell width per column and positions follow those widths.
    #[test]
    fn grid_column_widths_are_maxima() {
        let items = [cell(10.0, 0, 0), cell(30.0, 1, 0), cell(20.0, 0, 1)];
        assert_eq!(grid_request(&items), Size::new(50.0, 20.0));
        let result = grid(Rect::new(0.0, 0.0, 50.0, 20.0), &items, &[], &[]);
        assert_eq!(result[2].x, 30.0);
    }

    /// Verifies that sticky E|W stretches a grid cell to fill its column width.
    #[test]
    fn grid_sticky_ew_stretches() {
        let mut item = cell(20.0, 0, 0);
        item.opts.sticky = Sticky::E | Sticky::W;
        let result = grid(Rect::new(0.0, 0.0, 100.0, 20.0), &[item], &[(0, 1)], &[]);
        assert_eq!(result[0].width, 100.0);
    }

    /// Checks that column and row weights distribute extra space proportionally across grid tracks.
    #[test]
    fn grid_weight_distribution() {
        let mut items = [cell(10.0, 0, 0), cell(10.0, 0, 1)];
        for item in &mut items {
            item.opts.sticky = Sticky::NSEW;
        }
        let result = grid(
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &items, &[(0, 1), (1, 3)], &[(0, 1)],
        );
        assert_eq!(result[0], Rect::new(0.0, 0.0, 30.0, 30.0));
        assert_eq!(result[1], Rect::new(30.0, 0.0, 70.0, 30.0));
    }

    /// Validates that columnspan, rowspan, and padding affect both the grid request and the placed rectangle.
    #[test]
    fn grid_spans_and_padding() {
        let mut item = cell(40.0, 0, 0);
        item.opts.columnspan = 2;
        item.opts.rowspan = 2;
        item.opts.padx = 5.0;
        item.opts.pady = 3.0;
        item.opts.sticky = Sticky::NSEW;
        assert_eq!(grid_request(&[item]), Size::new(50.0, 16.0));
        assert_eq!(
            grid(Rect::new(0.0, 0.0, 50.0, 16.0), &[item], &[], &[])[0],
            Rect::new(5.0, 3.0, 40.0, 10.0)
        );
    }
}
